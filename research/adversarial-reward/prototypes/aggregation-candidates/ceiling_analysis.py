from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from functools import partial
from pathlib import Path
from typing import Any, Callable, Mapping, Optional, Sequence

from candidates import chi_square_cdf_even_df, HybridConfig, aggregate_hybrid
from evaluate import evaluate_fixture, ScenarioCellResult
from models import (
    AggregateResult,
    ComponentContribution,
    DivergenceKind,
    MetricComponent,
)
from normalization import (
    bf_norm_log_scaled,
    bounded_unit_interval,
    direction_is_agreement,
    direction_is_none_variant,
    extract_uncertainty_snapshot,
    gate_precision,
    normalize_component,
    NormalizationConfig,
    se_dampen,
    sigmoid,
    _direction_value,
)
from perturbation_test import (
    BASELINE_HYBRID_CONFIG,
    S5_BF_VALUES,
    S6_BF_STRONG,
    S6_D_MID,
    build_s5_bf_variant,
    build_s6_compress_variant,
)
from scenarios import DEFAULT_CUSTOM_SIGMOIDS, _metric, build_scenario_fixtures


BF_MAX_TARGETS = [200, 500, 1000, 5000, 10000]
BF_CURVE_RANGE = range(1, 10001)
S5_SWEEP_BF = [80.0, 100.0, 120.0, 200.0, 500.0, 1000.0]
S6_FAILING_CELLS = [(3.0, 500.0), (3.0, 1000.0), (4.0, 100.0), (4.0, 500.0), (4.0, 1000.0)]
SCORE_AT_100_FLOOR = 0.3
SANITY_TOL = 1e-12
CEILING_THRESHOLD = 0.991


@dataclass(frozen=True)
class BFNormCandidate:
    name: str
    family: str
    bf_max_target: int
    free_param_name: str
    free_param_value: float
    norm_fn: Callable[[float], float]


@dataclass(frozen=True)
class CeilingProbeResult:
    candidate_name: str
    bf_max_target: int
    bf_ceiling: float
    score_at_100: float
    score_at_500: float
    score_at_1000: float
    passes_prefilter: bool


@dataclass(frozen=True)
class FullSuiteResult:
    candidate_name: str
    bf_max_target: int
    baseline_7_of_7: bool
    per_scenario: list[dict[str, Any]]
    s5_sweep: list[dict[str, Any]]


@dataclass(frozen=True)
class S6DecompositionCheck:
    d_mid: float
    bf_strong: float
    recon_error: float
    dominant_share: float
    failure_is_dominant_share: bool
    failure_is_recon_error: bool


def bf_norm_current(bf: float) -> float:
    return 1.0 - 1.0 / (1.0 + bf)


def bf_norm_power_law(bf: float, alpha: float) -> float:
    return 1.0 - 1.0 / (1.0 + bf) ** alpha


def bf_norm_exp_decay(bf: float, k: float) -> float:
    return 1.0 - math.exp(-bf / k)


def calibrate_log_scaled(bf_max_target: int) -> float:
    return math.log1p(float(bf_max_target)) * 0.009 / 0.991


def calibrate_power_law(bf_max_target: int) -> float:
    return math.log(0.009) / math.log(1.0 / (1.0 + float(bf_max_target)))


def calibrate_exp_decay(bf_max_target: int) -> float:
    return -float(bf_max_target) / math.log(0.009)


def build_bf_candidates() -> list[BFNormCandidate]:
    candidates: list[BFNormCandidate] = []
    for bf_max in BF_MAX_TARGETS:
        c_param = calibrate_log_scaled(bf_max)
        candidates.append(
            BFNormCandidate(
                name=f"log_scaled_bfmax_{bf_max}",
                family="log_scaled",
                bf_max_target=bf_max,
                free_param_name="c",
                free_param_value=c_param,
                norm_fn=partial(bf_norm_log_scaled, c=c_param),
            )
        )

        alpha_param = calibrate_power_law(bf_max)
        candidates.append(
            BFNormCandidate(
                name=f"power_law_bfmax_{bf_max}",
                family="power_law",
                bf_max_target=bf_max,
                free_param_name="alpha",
                free_param_value=alpha_param,
                norm_fn=partial(bf_norm_power_law, alpha=alpha_param),
            )
        )

        k_param = calibrate_exp_decay(bf_max)
        candidates.append(
            BFNormCandidate(
                name=f"exp_decay_bfmax_{bf_max}",
                family="exp_decay",
                bf_max_target=bf_max,
                free_param_name="k",
                free_param_value=k_param,
                norm_fn=partial(bf_norm_exp_decay, k=k_param),
            )
        )
    return candidates


def normalize_component_with_alt_bf(
    component: MetricComponent,
    config: NormalizationConfig,
    bf_norm_fn: Callable[[float], float],
) -> tuple[Optional[float], list[str], dict[str, float | str | None]]:
    if component.kind is not DivergenceKind.BayesFactor:
        return normalize_component(component, config)

    warnings: list[str] = []
    direction_value = _direction_value(component.direction)
    if component.direction is None or direction_is_none_variant(component.direction):
        transformed_value = abs(component.value)
        direction_mode = "unsigned"
    else:
        transformed_value = component.value
        direction_mode = direction_value or "unset"

    bf = max(transformed_value, 0.0)
    raw_score = bf_norm_fn(bf)

    bounded_raw = bounded_unit_interval(raw_score, config.clip_eps)
    if direction_is_agreement(component.direction):
        adjusted_score = 1.0 - bounded_raw
    else:
        adjusted_score = bounded_raw

    final_score = bounded_unit_interval(adjusted_score, config.clip_eps)
    if config.se_dampen_enabled:
        snapshot = extract_uncertainty_snapshot(component)
        if snapshot.standard_error is not None and snapshot.standard_error > 0:
            final_score = se_dampen(final_score, component.value, snapshot.standard_error, config)

    return final_score, warnings, {
        "raw_score": bounded_raw,
        "direction_mode": direction_mode,
        "transformed_value": transformed_value,
    }


def aggregate_hybrid_patched(
    components: Sequence[MetricComponent],
    config: HybridConfig | None,
    bf_norm_fn: Callable[[float], float],
) -> AggregateResult:
    cfg = config or HybridConfig()
    warnings: list[str] = []
    skipped: list[str] = []
    staged: list[dict[str, object]] = []

    for idx, component in enumerate(components):
        score, local_warnings, score_diag = normalize_component_with_alt_bf(
            component, cfg.normalization, bf_norm_fn
        )
        warnings.extend(local_warnings)
        if score is None:
            skipped.append(component.method_ref)
            continue

        precision = gate_precision(component, cfg.eps)
        if precision is None:
            confidence = cfg.c_missing
        else:
            confidence = max(cfg.c_floor, sigmoid(precision, cfg.alpha, cfg.tau))
        confidence = bounded_unit_interval(confidence, cfg.normalization.clip_eps)
        gated_score = score * confidence
        p_value = min(1.0, max(cfg.p_eps, 1.0 - gated_score))
        log_evidence = -2.0 * math.log(p_value)

        staged.append(
            {
                "idx": idx,
                "component": component,
                "score": score,
                "precision": precision,
                "confidence": confidence,
                "gated_score": gated_score,
                "p_value": p_value,
                "log_evidence": log_evidence,
                "raw_score": score_diag.get("raw_score"),
                "direction_mode": score_diag.get("direction_mode"),
            }
        )

    if not staged:
        return AggregateResult(
            candidate="Hybrid",
            aggregate_score=0.0,
            contributions=[],
            skipped=skipped,
            warnings=warnings,
        )

    total_log_evidence = sum(float(entry["log_evidence"]) for entry in staged)
    aggregate = chi_square_cdf_even_df(total_log_evidence, n_terms=1)
    aggregate = bounded_unit_interval(aggregate, cfg.normalization.clip_eps)

    denom = sum(float(entry["log_evidence"]) * float(entry["score"]) for entry in staged)
    if denom > cfg.eps:
        scale = aggregate / denom
        weights = [float(entry["log_evidence"]) * scale for entry in staged]
    else:
        weights = [0.0 for _ in staged]

    contributions: list[ComponentContribution] = []
    for weight, entry in zip(weights, staged):
        score = float(entry["score"])
        contribution = weight * score
        component = entry["component"]
        contributions.append(
            ComponentContribution(
                index=int(entry["idx"]),
                method_ref=component.method_ref,  # type: ignore[attr-defined]
                kind=component.kind,  # type: ignore[attr-defined]
                score=score,
                weight=weight,
                contribution=contribution,
                diagnostics={
                    "precision": entry["precision"],
                    "confidence": float(entry["confidence"]),
                    "gated_score": float(entry["gated_score"]),
                    "p_value": float(entry["p_value"]),
                    "log_evidence": float(entry["log_evidence"]),
                    "raw_score": entry["raw_score"],
                    "direction_mode": entry["direction_mode"],
                },
            )
        )

    return AggregateResult(
        candidate="Hybrid",
        aggregate_score=aggregate,
        contributions=contributions,
        skipped=skipped,
        warnings=warnings,
    )


def margin_from_cell(cell: ScenarioCellResult) -> tuple[float, str]:
    scores = {key: float(value) for key, value in cell.raw_scores.items()}
    if cell.scenario_index == 1:
        return scores["base"] - scores["doubled"], "base-doubled"
    if cell.scenario_index == 2:
        ratio = scores["aggregate"] / scores["max_single"]
        return ratio / 1.5 - 1.0, "(aggregate/max_single)/1.5-1"
    if cell.scenario_index == 3:
        lo = min(scores["all_agreement"], scores["all_contradiction"])
        hi = max(scores["all_agreement"], scores["all_contradiction"])
        return min(scores["mixed"] - lo, hi - scores["mixed"]), "min(mixed-lo,hi-mixed)"
    if cell.scenario_index == 4:
        return 0.20 - scores["relative_delta"], "0.20-relative_delta"
    if cell.scenario_index == 5:
        component_scores = [value for key, value in scores.items() if key != "aggregate"]
        return (
            min(min(component_scores) - 0.3, 0.991 - max(component_scores)),
            "min(min(score)-0.3,0.991-max(score))",
        )
    if cell.scenario_index == 6:
        recon_error = abs(scores["reconstructed"] - scores["aggregate"])
        return (
            min(scores["dominant_share"] - 0.35, 1e-8 - recon_error),
            "min(dominant_share-0.35,1e-8-abs(recon-aggregate))",
        )
    if cell.scenario_index == 7:
        return scores["non_boundary"] - scores["boundary"], "non_boundary-boundary"
    raise ValueError(f"Unsupported scenario index: {cell.scenario_index}")


def confirm_s6_decomposition() -> list[S6DecompositionCheck]:
    checks: list[S6DecompositionCheck] = []
    for d_mid, bf_strong in S6_FAILING_CELLS:
        fixture = build_s6_compress_variant(d_mid, bf_strong)
        cell = evaluate_fixture(
            fixture,
            "Hybrid",
            lambda components: aggregate_hybrid(components, BASELINE_HYBRID_CONFIG),
        )
        recon_error = abs(float(cell.raw_scores["reconstructed"]) - float(cell.raw_scores["aggregate"]))
        dominant_share = float(cell.raw_scores["dominant_share"])
        checks.append(
            S6DecompositionCheck(
                d_mid=d_mid,
                bf_strong=bf_strong,
                recon_error=recon_error,
                dominant_share=dominant_share,
                failure_is_dominant_share=(dominant_share < 0.35),
                failure_is_recon_error=(recon_error > 1e-8),
            )
        )

    if not all(check.failure_is_dominant_share and not check.failure_is_recon_error for check in checks):
        raise RuntimeError("S6 decomposition check failed: expected all failing cells to be dominant-share failures only.")
    return checks


def compute_bf_curves(
    candidates: Sequence[BFNormCandidate],
) -> tuple[dict[str, list[float]], list[CeilingProbeResult]]:
    norm_fns: dict[str, Callable[[float], float]] = {
        "current_reference": bf_norm_current,
    }
    for candidate in candidates:
        norm_fns[candidate.name] = candidate.norm_fn

    curves: dict[str, list[float]] = {}
    probes: list[CeilingProbeResult] = []

    for name, norm_fn in norm_fns.items():
        curve = [float(norm_fn(float(bf))) for bf in BF_CURVE_RANGE]
        curves[name] = curve
        bf_ceiling = find_exact_bf_ceiling(curve)
        score_at_100 = float(norm_fn(100.0))
        score_at_500 = float(norm_fn(500.0))
        score_at_1000 = float(norm_fn(1000.0))
        passes_prefilter = score_at_100 >= SCORE_AT_100_FLOOR
        bf_max_target = 111 if name == "current_reference" else int(name.rsplit("_", 1)[-1])
        probes.append(
            CeilingProbeResult(
                candidate_name=name,
                bf_max_target=bf_max_target,
                bf_ceiling=bf_ceiling,
                score_at_100=score_at_100,
                score_at_500=score_at_500,
                score_at_1000=score_at_1000,
                passes_prefilter=passes_prefilter,
            )
        )

    def probe_sort_key(probe: CeilingProbeResult) -> tuple[int, str, int]:
        if probe.candidate_name == "current_reference":
            return (0, probe.candidate_name, probe.bf_max_target)
        return (1, probe.candidate_name, probe.bf_max_target)

    probes.sort(key=probe_sort_key)
    return curves, probes


def find_exact_bf_ceiling(curve: Sequence[float]) -> float:
    ceiling = 0
    for bf, score in zip(BF_CURVE_RANGE, curve):
        if score < CEILING_THRESHOLD:
            ceiling = bf
    return float(ceiling)


def run_sanity_gate(fixtures: Sequence[Any]) -> list[dict[str, Any]]:
    sanity_rows: list[dict[str, Any]] = []
    patched_fn = lambda components: aggregate_hybrid_patched(
        components, BASELINE_HYBRID_CONFIG, bf_norm_current
    )
    baseline_fn = lambda components: aggregate_hybrid(components, BASELINE_HYBRID_CONFIG)

    for fixture in fixtures:
        patched_cell = evaluate_fixture(fixture, "Hybrid-Patched-Current", patched_fn)
        baseline_cell = evaluate_fixture(fixture, "Hybrid-Baseline", baseline_fn)

        patched_keys = set(patched_cell.raw_scores.keys())
        baseline_keys = set(baseline_cell.raw_scores.keys())
        if patched_keys != baseline_keys:
            raise RuntimeError(
                f"Sanity gate failed for S{fixture.idx} {fixture.name}: raw-score keys differ "
                f"patched={sorted(patched_keys)} baseline={sorted(baseline_keys)}"
            )

        max_abs_diff = 0.0
        worst_key = ""
        for key in baseline_keys:
            baseline_value = float(baseline_cell.raw_scores[key])
            patched_value = float(patched_cell.raw_scores[key])
            diff = abs(patched_value - baseline_value)
            if diff > max_abs_diff:
                max_abs_diff = diff
                worst_key = key
            if diff > SANITY_TOL:
                raise RuntimeError(
                    f"Sanity gate failed for S{fixture.idx} {fixture.name}, key '{key}': "
                    f"patched={patched_value:.16f}, baseline={baseline_value:.16f}, diff={diff:.3e}, tol={SANITY_TOL:.1e}"
                )

        sanity_rows.append(
            {
                "scenario_idx": fixture.idx,
                "scenario_name": fixture.name,
                "max_abs_diff": max_abs_diff,
                "worst_key": worst_key,
                "passed": max_abs_diff <= SANITY_TOL,
                "baseline_passed": baseline_cell.passed,
            }
        )

    return sanity_rows


def run_full_suite_evaluation(
    fixtures: Sequence[Any],
    candidates: Sequence[BFNormCandidate],
    probe_by_name: Mapping[str, CeilingProbeResult],
) -> tuple[list[dict[str, Any]], list[FullSuiteResult]]:
    sanity_rows = run_sanity_gate(fixtures)

    full_results: list[FullSuiteResult] = []
    for candidate in candidates:
        probe = probe_by_name[candidate.name]
        if not probe.passes_prefilter:
            continue

        patched_fn = lambda components, fn=candidate.norm_fn: aggregate_hybrid_patched(
            components, BASELINE_HYBRID_CONFIG, fn
        )

        per_scenario: list[dict[str, Any]] = []
        for fixture in fixtures:
            cell = evaluate_fixture(fixture, candidate.name, patched_fn)
            margin, margin_label = margin_from_cell(cell)
            per_scenario.append(
                {
                    "scenario_idx": fixture.idx,
                    "scenario_name": fixture.name,
                    "passed": cell.passed,
                    "margin": margin,
                    "margin_label": margin_label,
                    "raw_scores": {key: float(value) for key, value in cell.raw_scores.items()},
                }
            )

        s5_sweep: list[dict[str, Any]] = []
        for bf_value in S5_SWEEP_BF:
            fixture = build_s5_bf_variant(bf_value)
            cell = evaluate_fixture(fixture, candidate.name, patched_fn)
            margin, margin_label = margin_from_cell(cell)
            s5_sweep.append(
                {
                    "bf_value": bf_value,
                    "passed": cell.passed,
                    "margin": margin,
                    "margin_label": margin_label,
                    "raw_scores": {key: float(value) for key, value in cell.raw_scores.items()},
                }
            )

        full_results.append(
            FullSuiteResult(
                candidate_name=candidate.name,
                bf_max_target=candidate.bf_max_target,
                baseline_7_of_7=all(row["passed"] for row in per_scenario),
                per_scenario=per_scenario,
                s5_sweep=s5_sweep,
            )
        )

    return sanity_rows, full_results


def check_s6_side_benefit(
    full_results: Sequence[FullSuiteResult],
    candidate_by_name: Mapping[str, BFNormCandidate],
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for result in full_results:
        if not result.baseline_7_of_7:
            continue
        candidate = candidate_by_name[result.candidate_name]
        patched_fn = lambda components, fn=candidate.norm_fn: aggregate_hybrid_patched(
            components, BASELINE_HYBRID_CONFIG, fn
        )

        cells: list[dict[str, Any]] = []
        for d_mid, bf_strong in S6_FAILING_CELLS:
            fixture = build_s6_compress_variant(d_mid, bf_strong)
            cell = evaluate_fixture(fixture, candidate.name, patched_fn)
            dominant_share = float(cell.raw_scores["dominant_share"])
            recon_error = abs(float(cell.raw_scores["reconstructed"]) - float(cell.raw_scores["aggregate"]))
            cells.append(
                {
                    "d_mid": d_mid,
                    "bf_strong": bf_strong,
                    "dominant_share": dominant_share,
                    "recon_error": recon_error,
                    "passes_now": dominant_share >= 0.35,
                }
            )

        improved = [cell for cell in cells if cell["passes_now"]]
        rows.append(
            {
                "candidate_name": candidate.name,
                "family": candidate.family,
                "bf_max_target": candidate.bf_max_target,
                "improved_cells": improved,
                "cells": cells,
            }
        )
    return rows


def select_best_candidate(
    full_results: Sequence[FullSuiteResult],
    probe_by_name: Mapping[str, CeilingProbeResult],
) -> Optional[str]:
    if not full_results:
        return None

    scored_rows: list[tuple[bool, int, float, float, str]] = []
    for result in full_results:
        pass_count = sum(1 for row in result.per_scenario if row["passed"])
        probe = probe_by_name[result.candidate_name]
        scored_rows.append(
            (
                result.baseline_7_of_7,
                pass_count,
                probe.bf_ceiling,
                probe.score_at_100,
                result.candidate_name,
            )
        )

    scored_rows.sort(reverse=True)
    return scored_rows[0][-1]


def build_markdown(
    phase1_checks: Sequence[S6DecompositionCheck],
    probes: Sequence[CeilingProbeResult],
    full_results: Sequence[FullSuiteResult],
    side_benefits: Sequence[dict[str, Any]],
    candidate_by_name: Mapping[str, BFNormCandidate],
    best_candidate_name: Optional[str],
) -> str:
    full_by_name = {row.candidate_name: row for row in full_results}
    probe_by_name = {row.candidate_name: row for row in probes}

    lines: list[str] = []
    lines.append("# BF Normalization Ceiling Analysis\n\n")
    lines.append(f"Generated: {datetime.now(timezone.utc).isoformat()}\n\n")

    lines.append("## 1. S6 Decomposition\n\n")
    lines.append(
        "| d_mid | bf_strong | dominant_share | recon_error | failure_is_dominant_share | failure_is_recon_error |\n"
    )
    lines.append("| ---: | ---: | ---: | ---: | :---: | :---: |\n")
    for row in phase1_checks:
        lines.append(
            f"| {row.d_mid:.1f} | {row.bf_strong:.1f} | {row.dominant_share:.6f} | "
            f"{row.recon_error:.3e} | {'yes' if row.failure_is_dominant_share else 'no'} | "
            f"{'yes' if row.failure_is_recon_error else 'no'} |\n"
        )
    lines.append(
        "\nAll five failing S6 cells are dominant-share failures (`dominant_share < 0.35`) with reconstruction error within tolerance.\n\n"
    )

    lines.append("## 2. BF Normalization Comparison\n\n")
    lines.append(
        "| name | bf_ceiling | score@100 | score@500 | score@1000 | pre-filter | 7/7 pass? |\n"
    )
    lines.append("| :--- | ---: | ---: | ---: | ---: | :---: | :---: |\n")
    for probe in probes:
        full = full_by_name.get(probe.candidate_name)
        if probe.candidate_name == "current_reference":
            pass_status = "baseline"
        elif full is None:
            pass_status = "not-run"
        else:
            pass_status = "yes" if full.baseline_7_of_7 else "no"
        lines.append(
            f"| {probe.candidate_name} | {probe.bf_ceiling:.0f} | {probe.score_at_100:.6f} | "
            f"{probe.score_at_500:.6f} | {probe.score_at_1000:.6f} | "
            f"{'yes' if probe.passes_prefilter else 'no'} | {pass_status} |\n"
        )
    lines.append("\n")

    lines.append("## 3. Best Candidate Detail\n\n")
    if best_candidate_name is None or best_candidate_name not in full_by_name:
        lines.append("No candidate passed pre-filter and full-suite evaluation.\n\n")
    else:
        best_full = full_by_name[best_candidate_name]
        best_probe = probe_by_name[best_candidate_name]
        best_candidate = candidate_by_name[best_candidate_name]
        lines.append(
            f"Best candidate: `{best_candidate.name}` "
            f"(family=`{best_candidate.family}`, bf_max_target=`{best_candidate.bf_max_target}`, "
            f"{best_candidate.free_param_name}=`{best_candidate.free_param_value:.6f}`, "
            f"bf_ceiling=`{best_probe.bf_ceiling:.0f}`)\n\n"
        )
        lines.append("### Baseline 7-scenario margins\n\n")
        lines.append("| scenario | pass | margin | margin_label |\n")
        lines.append("| :--- | :---: | ---: | :--- |\n")
        for row in best_full.per_scenario:
            lines.append(
                f"| S{int(row['scenario_idx'])} {row['scenario_name']} | "
                f"{'PASS' if row['passed'] else 'FAIL'} | {float(row['margin']):+.6f} | "
                f"{row['margin_label']} |\n"
            )
        lines.append("\n### S5 BF sweep\n\n")
        lines.append("| BF | pass | margin | max_component | min_component |\n")
        lines.append("| ---: | :---: | ---: | ---: | ---: |\n")
        for row in best_full.s5_sweep:
            component_scores = [
                float(value)
                for key, value in row["raw_scores"].items()
                if key != "aggregate"
            ]
            lines.append(
                f"| {float(row['bf_value']):.0f} | {'PASS' if row['passed'] else 'FAIL'} | "
                f"{float(row['margin']):+.6f} | {max(component_scores):.6f} | {min(component_scores):.6f} |\n"
            )
        lines.append("\n")

    lines.append("## 4. S6 Side-Benefit\n\n")
    if not side_benefits:
        lines.append("No baseline 7/7 candidate was available for S6 side-benefit checks.\n\n")
    else:
        lines.append("| candidate | bf_max_target | improved failing S6 cells |\n")
        lines.append("| :--- | ---: | :--- |\n")
        for row in side_benefits:
            improved_cells = row["improved_cells"]
            if improved_cells:
                cell_text = ", ".join(
                    f"(d_mid={cell['d_mid']:.1f}, bf={cell['bf_strong']:.1f})"
                    for cell in improved_cells
                )
            else:
                cell_text = "none"
            lines.append(
                f"| {row['candidate_name']} | {int(row['bf_max_target'])} | {cell_text} |\n"
            )
        lines.append("\n")

    lines.append("## 5. Recommendation\n\n")
    if best_candidate_name is None or best_candidate_name not in full_by_name:
        lines.append(
            "No candidate satisfied the post-filter suite strongly enough for recommendation. "
            "Keep current normalization and revisit BF normalization families.\n"
        )
    else:
        best_full = full_by_name[best_candidate_name]
        best_probe = probe_by_name[best_candidate_name]
        best_candidate = candidate_by_name[best_candidate_name]
        resolves_s5 = any(
            row["passed"] and float(row["bf_value"]) >= 500.0 for row in best_full.s5_sweep
        )
        lines.append(
            f"- Decision: {'GO' if best_full.baseline_7_of_7 and resolves_s5 else 'NO-GO'} for athena-e2a adoption.\n"
        )
        lines.append(
            f"- Recommended normalization family: `{best_candidate.family}`\n"
        )
        lines.append(
            f"- Recommended bf_max_target: `{best_candidate.bf_max_target}`\n"
        )
        lines.append(
            f"- Evidence: baseline 7/7 = `{best_full.baseline_7_of_7}`, "
            f"bf_ceiling = `{best_probe.bf_ceiling:.0f}`, "
            f"S5 pass at BF>=500 = `{resolves_s5}`.\n"
        )
    return "".join(lines)


def main() -> None:
    _ = _metric
    if BASELINE_HYBRID_CONFIG.normalization.custom_sigmoids != dict(DEFAULT_CUSTOM_SIGMOIDS):
        raise RuntimeError("BASELINE_HYBRID_CONFIG custom sigmoid map diverges from DEFAULT_CUSTOM_SIGMOIDS.")

    fixtures = build_scenario_fixtures()
    candidates = build_bf_candidates()
    candidate_by_name = {candidate.name: candidate for candidate in candidates}

    phase1_checks = confirm_s6_decomposition()
    curves, probes = compute_bf_curves(candidates)
    probe_by_name = {probe.candidate_name: probe for probe in probes}

    sanity_rows, full_results = run_full_suite_evaluation(fixtures, candidates, probe_by_name)
    side_benefits = check_s6_side_benefit(full_results, candidate_by_name)
    best_candidate_name = select_best_candidate(full_results, probe_by_name)

    payload = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "constants": {
            "BF_MAX_TARGETS": BF_MAX_TARGETS,
            "BF_CURVE_RANGE": [BF_CURVE_RANGE.start, BF_CURVE_RANGE.stop - 1],
            "S5_SWEEP_BF": S5_SWEEP_BF,
            "S5_BF_VALUES_REFERENCE": S5_BF_VALUES,
            "S6_D_MID_REFERENCE": S6_D_MID,
            "S6_BF_STRONG_REFERENCE": S6_BF_STRONG,
            "S6_FAILING_CELLS": S6_FAILING_CELLS,
            "SCORE_AT_100_FLOOR": SCORE_AT_100_FLOOR,
            "SANITY_TOL": SANITY_TOL,
            "CEILING_THRESHOLD": CEILING_THRESHOLD,
        },
        "candidates": [
            {
                "name": candidate.name,
                "family": candidate.family,
                "bf_max_target": candidate.bf_max_target,
                "free_param_name": candidate.free_param_name,
                "free_param_value": candidate.free_param_value,
            }
            for candidate in candidates
        ],
        "phase1_s6_decomposition": [asdict(row) for row in phase1_checks],
        "phase2_curves": curves,
        "phase2_ceiling_probes": [asdict(row) for row in probes],
        "phase3_sanity_gate": sanity_rows,
        "phase3_full_suite": [asdict(row) for row in full_results],
        "phase4_s6_side_benefit": side_benefits,
        "best_candidate_name": best_candidate_name,
    }

    report_md = build_markdown(
        phase1_checks=phase1_checks,
        probes=probes,
        full_results=full_results,
        side_benefits=side_benefits,
        candidate_by_name=candidate_by_name,
        best_candidate_name=best_candidate_name,
    )

    here = Path(__file__).resolve().parent
    json_path = here / "ceiling_analysis.json"
    md_path = here / "ceiling_analysis.md"
    json_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    md_path.write_text(report_md, encoding="utf-8")

    prefilter_count = sum(1 for probe in probes if probe.candidate_name != "current_reference" and probe.passes_prefilter)
    baseline_7of7_count = sum(1 for row in full_results if row.baseline_7_of_7)
    print(f"Candidates built: {len(candidates)}")
    print(f"Candidates passing pre-filter: {prefilter_count}")
    print(f"Candidates with baseline 7/7: {baseline_7of7_count}")
    print(f"Best candidate: {best_candidate_name}")
    print(f"Wrote {json_path}")
    print(f"Wrote {md_path}")


if __name__ == "__main__":
    main()
