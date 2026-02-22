from __future__ import annotations
import json
import random
from dataclasses import asdict
from datetime import datetime, timezone
from functools import partial
from pathlib import Path
from statistics import mean
from typing import Any, Callable, Sequence
from calibration_sim import (
    pearson_r,
    rank_values,
    run_pattern_a,
    run_pattern_b,
    run_pattern_c,
    spearman_rho,
    with_uncertainty_scale,
    with_value_scale,
)
from candidates import chi_square_cdf_even_df
from ceiling_analysis import aggregate_hybrid_patched, bf_norm_log_scaled, margin_from_cell
from correlation_test import correlation_matrix, cholesky_decompose, perturb_component, sample_correlated, variance
from evaluate import evaluate_fixture
from models import AggregateResult, MetricComponent
from perturbation_test import BASELINE_HYBRID_CONFIG
from scenarios import build_scenario_fixtures
BF_NORM_C = 0.083647
BF_MAX_TARGET = 10000
RHO_VALUES = [0.0, 0.3, 0.5, 0.7, 0.9]
SAMPLES_PER_RHO = 400
FLOOR_THRESHOLD = 2e-12
BASELINE_MARGIN_TOL = 1e-6
CEILING_RESULTS_FILE = "ceiling_analysis.json"
CEILING_CANDIDATE = "log_scaled_bfmax_10000"
S2_CALIBRATION = {
    "A": {
        "name": "Gradual convergence",
        "metric": "spearman_rho",
        "threshold": "< -0.9",
        "rows": {"IVW-CDF": (-0.8728, 0.1064, False), "HTG-Max": (-1.0000, 0.0010, True), "Fisher-UP": (-1.0000, 0.0386, True)},
    },
    "B": {
        "name": "Sudden regime change",
        "metric": "step_ratio",
        "threshold": "> 3.0",
        "rows": {"IVW-CDF": (2.9533, 0.5677, False), "HTG-Max": (1.0036, 0.0035, False), "Fisher-UP": (2553.2200, 0.9996, False)},
    },
    "C": {
        "name": "Oscillating uncertainty",
        "metric": "pearson_r",
        "threshold": "< -0.5",
        "rows": {"IVW-CDF": (0.0000, 0.0000, False), "HTG-Max": (-0.8784, 0.0000, True), "Fisher-UP": (-0.9635, 0.0017, True)},
    },
}
S2_CORRELATION = {
    0.0: (1.0, True),
    0.3: (1.0, True),
    0.5: (1.0, True),
    0.7: (1.002540134976042, True),
    0.9: (1.0, True),
}
def clamp(value: int, lo: int, hi: int) -> int:
    return max(lo, min(hi, value))
def _find_fixture(fixtures: Sequence[Any], idx: int) -> Any:
    for fixture in fixtures:
        if fixture.idx == idx:
            return fixture
    raise RuntimeError(f"Missing fixture idx={idx}")
def _to_float_map(values: dict[str, float]) -> dict[str, float]:
    return {k: float(v) for k, v in values.items()}
def _total_log_evidence(result: AggregateResult) -> float:
    total = 0.0
    for contribution in result.contributions:
        if "log_evidence" not in contribution.diagnostics:
            raise RuntimeError(f"Missing diagnostics['log_evidence'] for {contribution.method_ref}")
        total += float(contribution.diagnostics["log_evidence"])
    return total
def _load_ceiling_baseline(root: Path) -> dict[int, dict[str, Any]]:
    payload = json.loads((root / CEILING_RESULTS_FILE).read_text(encoding="utf-8"))
    target = next((r for r in payload.get("phase3_full_suite", []) if r.get("candidate_name") == CEILING_CANDIDATE), None)
    if target is None:
        raise RuntimeError(f"Missing {CEILING_CANDIDATE} in {CEILING_RESULTS_FILE}")
    return {
        int(r["scenario_idx"]): {
            "scenario_name": str(r["scenario_name"]),
            "margin": float(r["margin"]),
            "margin_label": str(r["margin_label"]),
        }
        for r in target["per_scenario"]
    }
# Phase 0: baseline gate (abort on any fail)
def phase0_baseline_gate(
    fixtures: Sequence[Any],
    hybrid_fn: Callable[[Sequence[MetricComponent]], AggregateResult],
    root: Path,
) -> dict[str, Any]:
    baseline = _load_ceiling_baseline(root)
    rows, failures = [], []
    for fixture in fixtures:
        cell = evaluate_fixture(fixture, "Hybrid-Stretch", hybrid_fn)
        margin, margin_label = margin_from_cell(cell)
        ref = baseline.get(fixture.idx)
        if ref is None:
            failures.append(f"S{fixture.idx} missing baseline reference")
            continue
        margin_delta = float(margin - ref["margin"])
        passed, label_match = bool(cell.passed), margin_label == ref["margin_label"]
        margin_match = abs(margin_delta) < BASELINE_MARGIN_TOL
        rows.append(
            {
                "scenario_idx": int(fixture.idx),
                "scenario_name": str(fixture.name),
                "passed": passed,
                "margin": float(margin),
                "margin_label": margin_label,
                "baseline_margin": float(ref["margin"]),
                "baseline_margin_label": str(ref["margin_label"]),
                "margin_delta": margin_delta,
                "label_match": label_match,
                "margin_match": margin_match,
                "raw_scores": _to_float_map(cell.raw_scores),
            }
        )
        if not passed:
            failures.append(f"S{fixture.idx} failed pass gate")
        if not label_match:
            failures.append(f"S{fixture.idx} margin label mismatch ({margin_label} != {ref['margin_label']})")
        if not margin_match:
            failures.append(f"S{fixture.idx} margin delta {margin_delta:.3e} exceeds {BASELINE_MARGIN_TOL:.1e}")
    if failures:
        raise RuntimeError("Phase 0 baseline gate failed:\n" + "\n".join(f"- {line}" for line in failures))
    return {"status": "pass", "scenario_count": len(rows), "rows": rows}
# Phase 1: calibration
def _pattern_pass(pattern_id: str, metric_value: float, max_delta: float) -> tuple[bool, bool]:
    if pattern_id == "A":
        return metric_value < -0.9, max_delta < 0.3
    if pattern_id == "B":
        return metric_value > 3.0, max_delta < 0.3
    if pattern_id == "C":
        return metric_value < -0.5, max_delta < 0.3
    raise RuntimeError(f"Unsupported pattern {pattern_id}")
def _pattern_b_class(step_ratio: float, max_delta: float) -> str:
    responsive, smooth = step_ratio > 3.0, max_delta < 0.3
    if responsive and smooth:
        return "responsive and smooth"
    if responsive and not smooth:
        return "responsive but non-smooth"
    if (not responsive) and smooth:
        return "non-responsive but smooth"
    return "non-responsive and non-smooth"
def _s2_rows_for_pattern(pattern_id: str) -> dict[str, dict[str, Any]]:
    rows = S2_CALIBRATION[pattern_id]["rows"]
    return {
        name: {"metric_value": float(v[0]), "max_delta": float(v[1]), "overall_pass": bool(v[2])}
        for name, v in rows.items()
    }
def phase1_calibration(
    base_metrics: Sequence[MetricComponent],
    scorer: Callable[[Sequence[MetricComponent]], float],
) -> dict[str, Any]:
    patterns = {"A": ("Gradual convergence", run_pattern_a), "B": ("Sudden regime change", run_pattern_b), "C": ("Oscillating uncertainty", run_pattern_c)}
    probe = base_metrics[0]
    probe_info = {
        "probe_method_ref": probe.method_ref,
        "value_scaled_0_5": float(with_value_scale(probe, 0.5).value),
        "uncertainty_scaled_1_5_value": float(with_uncertainty_scale(probe, 1.5).value),
    }
    results, comparison = [], []
    for pattern_id, (pattern_name, runner) in patterns.items():
        cycles, metrics = runner(base_metrics, scorer)
        metric_value, max_delta = float(metrics["metric_value"]), float(metrics["max_delta"])
        metric_pass, smooth_pass = _pattern_pass(pattern_id, metric_value, max_delta)
        scores, tvals = [float(c["score"]) for c in cycles], [float(c["t"]) for c in cycles]
        helper_pearson = None
        if pattern_id == "C":
            helper_pearson = pearson_r([float(c["mean_se"]) for c in cycles], scores)
        row = {
            "pattern_id": pattern_id,
            "pattern_name": pattern_name,
            "metric_name": str(metrics["metric_name"]),
            "metric_value": metric_value,
            "threshold": float(metrics["threshold"]),
            "metric_pass": bool(metric_pass),
            "max_delta": max_delta,
            "smoothness_pass": bool(smooth_pass),
            "overall_pass": bool(metric_pass and smooth_pass),
            "cycles": cycles,
            "helper_checks": {
                "spearman_rho": float(spearman_rho(tvals, scores)),
                "pearson_r": None if helper_pearson is None else float(helper_pearson),
                "rank_head": [float(v) for v in rank_values(scores)[:5]],
            },
        }
        if pattern_id == "B":
            row["narrative"] = _pattern_b_class(metric_value, max_delta)
        results.append(row)
        comparison.append(
            {
                "pattern_id": pattern_id,
                "pattern_name": pattern_name,
                "hybrid": {
                    "metric_name": row["metric_name"],
                    "metric_value": metric_value,
                    "max_delta": max_delta,
                    "overall_pass": row["overall_pass"],
                },
                "session2_single_family": _s2_rows_for_pattern(pattern_id),
            }
        )
    s2_ref = {
        pid: {
            "name": info["name"],
            "metric": info["metric"],
            "threshold": info["threshold"],
            "single_family": _s2_rows_for_pattern(pid),
        }
        for pid, info in S2_CALIBRATION.items()
    }
    return {
        "status": "complete",
        "fixture": "S6 Calibration decomposability",
        "cycles_per_pattern": 50,
        "criteria": {
            "A": "spearman_rho < -0.9 and max_delta < 0.3",
            "B": "step_ratio > 3.0 and max_delta < 0.3",
            "C": "pearson_r < -0.5 and max_delta < 0.3",
        },
        "helper_probe": probe_info,
        "results": results,
        "session2_reference": s2_ref,
        "comparison": comparison,
    }
# Phase 2: correlation robustness
def phase2_correlation(
    base_metrics: Sequence[MetricComponent],
    hybrid_fn: Callable[[Sequence[MetricComponent]], AggregateResult],
) -> dict[str, Any]:
    k = len(base_metrics)
    random.seed(42)  # required determinism anchor
    rows = []
    for rho in RHO_VALUES:
        chol = cholesky_decompose(correlation_matrix(k, rho))
        uncorrected, tvals = [], []
        for _ in range(SAMPLES_PER_RHO):
            noise = sample_correlated(chol)
            perturbed = [perturb_component(c, n) for c, n in zip(base_metrics, noise)]
            result = hybrid_fn(perturbed)
            uncorrected.append(float(result.aggregate_score))
            tvals.append(_total_log_evidence(result))
        var_t = float(variance(tvals))
        effective_df = float((2.0 * (k**2)) / var_t) if var_t > 0.0 else float(2 * k)
        corrected_terms = clamp(int(effective_df / 2.0), 1, 1000)
        corrected = [float(chi_square_cdf_even_df(t, n_terms=corrected_terms)) for t in tvals]
        mean_unc, mean_cor = float(mean(uncorrected)), float(mean(corrected))
        inflation = mean_unc / max(mean_cor, 1e-12)
        floor_count = sum(v <= FLOOR_THRESHOLD for v in uncorrected)
        floor_fraction = floor_count / float(len(uncorrected))
        rows.append(
            {
                "rho": float(rho),
                "samples": SAMPLES_PER_RHO,
                "k_components": k,
                "uncorrected_n_terms": 1,
                "var_t": var_t,
                "effective_df": effective_df,
                "corrected_n_terms": corrected_terms,
                "mean_uncorrected_aggregate": mean_unc,
                "mean_corrected_aggregate": mean_cor,
                "inflation_ratio": float(inflation),
                "floor_threshold": FLOOR_THRESHOLD,
                "floor_count": int(floor_count),
                "floor_fraction": float(floor_fraction),
                "floor_saturated": bool(floor_fraction > 0.5),
            }
        )
    rho_05 = next(row for row in rows if abs(row["rho"] - 0.5) < 1e-12)
    pass_05 = bool(rho_05["inflation_ratio"] <= 1.5 and not rho_05["floor_saturated"])
    comparison = []
    for row in rows:
        s2_inflation, s2_floor = S2_CORRELATION[float(row["rho"])]
        comparison.append(
            {
                "rho": float(row["rho"]),
                "hybrid_inflation_ratio": float(row["inflation_ratio"]),
                "hybrid_floor_saturated": bool(row["floor_saturated"]),
                "session2_inflation_ratio": float(s2_inflation),
                "session2_floor_saturated_inferred": bool(s2_floor),
            }
        )
    return {
        "status": "complete",
        "fixture": "S6 Calibration decomposability",
        "seed": 42,
        "rho_values": list(RHO_VALUES),
        "samples_per_rho": SAMPLES_PER_RHO,
        "brown_correction_note": (
            "Hybrid uses n_terms=1. Brown-style effective_df/corrected_n_terms are diagnostic "
            "context, not an exact correction of the hybrid objective."
        ),
        "results": rows,
        "session2_reference": {
            str(k): {"inflation_ratio": float(v[0]), "floor_saturated_inferred": bool(v[1])}
            for k, v in S2_CORRELATION.items()
        },
        "comparison": comparison,
        "pass_criterion": "inflation_ratio <= 1.5 at rho=0.5 AND floor_saturated == False",
        "pass_at_rho_0_5": pass_05,
        "all_rho_floor_not_saturated": all(not row["floor_saturated"] for row in rows),
    }
# Phase 3: output generation
def _fmt(pass_flag: bool) -> str:
    return "PASS" if pass_flag else "FAIL"
def build_markdown(payload: dict[str, Any]) -> str:
    p0, p1, p2 = payload["phase0_baseline_gate"], payload["phase1_calibration"], payload["phase2_correlation"]
    b = next(row for row in p1["results"] if row["pattern_id"] == "B")
    out = [
        "# Session 5 Stretch Summary\n\n",
        f"Generated: {payload['generated_at_utc']}\n\n",
        f"Hybrid: HTG gating + Fisher product, log-scaled BF normalization (c={BF_NORM_C}, bf_max_target={BF_MAX_TARGET}).\n\n",
        "## Phase 0 — Baseline Verification\n\n",
        "| Scenario | Pass | Margin | Baseline margin | Delta |\n",
        "| :--- | :---: | ---: | ---: | ---: |\n",
    ]
    for row in p0["rows"]:
        out.append(
            f"| S{row['scenario_idx']} {row['scenario_name']} | {_fmt(row['passed'])} | {row['margin']:+.6f} | "
            f"{row['baseline_margin']:+.6f} | {row['margin_delta']:+.3e} |\n"
        )
    out += [
        "\n## Phase 1 — Calibration\n\n",
        "| Pattern | Metric | Value | Max delta | Metric pass | Smoothness pass | Overall |\n",
        "| :--- | :--- | ---: | ---: | :---: | :---: | :---: |\n",
    ]
    for row in p1["results"]:
        out.append(
            f"| {row['pattern_id']} {row['pattern_name']} | {row['metric_name']} | {row['metric_value']:.4f} | {row['max_delta']:.4f} | "
            f"{_fmt(row['metric_pass'])} | {_fmt(row['smoothness_pass'])} | {_fmt(row['overall_pass'])} |\n"
        )
    out += [
        "\n### Pattern B Narrative\n\n",
        "Key question: does the hybrid respond to sudden single-metric regime change where Session 2 single-family candidates failed?\n\n",
        f"Hybrid Pattern B: step_ratio={b['metric_value']:.4f}, max_delta={b['max_delta']:.4f}, classification={b.get('narrative', 'n/a')}.\n\n",
        "### Session 2 Calibration Comparison\n\n",
        "| Pattern | Hybrid (value, delta, overall) | IVW-CDF | HTG-Max | Fisher-UP |\n",
        "| :--- | :--- | :--- | :--- | :--- |\n",
    ]
    for row in p1["comparison"]:
        s2 = row["session2_single_family"]
        out.append(
            f"| {row['pattern_id']} {row['pattern_name']} | {row['hybrid']['metric_value']:.4f}, {row['hybrid']['max_delta']:.4f}, {_fmt(row['hybrid']['overall_pass'])} | "
            f"{s2['IVW-CDF']['metric_value']:.4f}, {s2['IVW-CDF']['max_delta']:.4f}, {_fmt(s2['IVW-CDF']['overall_pass'])} | "
            f"{s2['HTG-Max']['metric_value']:.4f}, {s2['HTG-Max']['max_delta']:.4f}, {_fmt(s2['HTG-Max']['overall_pass'])} | "
            f"{s2['Fisher-UP']['metric_value']:.4f}, {s2['Fisher-UP']['max_delta']:.4f}, {_fmt(s2['Fisher-UP']['overall_pass'])} |\n"
        )
    out += [
        "\n## Phase 2 — Correlation Robustness\n\n",
        "| rho | mean unc | mean cor | inflation | var(T) | eff_df | corr_terms | floor_count | floor_saturated |\n",
        "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | :---: |\n",
    ]
    for row in p2["results"]:
        out.append(
            f"| {row['rho']:.1f} | {row['mean_uncorrected_aggregate']:.6e} | {row['mean_corrected_aggregate']:.6e} | {row['inflation_ratio']:.4f} | "
            f"{row['var_t']:.6e} | {row['effective_df']:.2f} | {row['corrected_n_terms']} | {row['floor_count']} | "
            f"{'yes' if row['floor_saturated'] else 'no'} |\n"
        )
    out += [
        "\n### Session 2 Correlation Comparison\n\n",
        "| rho | Hybrid inflation | Hybrid floor-saturated | Session 2 inflation | Session 2 floor-saturated (inferred) |\n",
        "| ---: | ---: | :---: | ---: | :---: |\n",
    ]
    for row in p2["comparison"]:
        out.append(
            f"| {row['rho']:.1f} | {row['hybrid_inflation_ratio']:.4f} | {'yes' if row['hybrid_floor_saturated'] else 'no'} | "
            f"{row['session2_inflation_ratio']:.4f} | {'yes' if row['session2_floor_saturated_inferred'] else 'no'} |\n"
        )
    out += [
        "\n## Summary Verdict\n\n",
        f"- Phase 0 gate: {_fmt(p0['status'] == 'pass')}\n",
        f"- Pattern B classification: {b.get('narrative', 'n/a')}\n",
        f"- Correlation pass at rho=0.5: {_fmt(p2['pass_at_rho_0_5'])}\n",
        f"- Floor-saturation clear across all rho: {_fmt(p2['all_rho_floor_not_saturated'])}\n",
    ]
    return "".join(out)
def write_outputs(root: Path, payload: dict[str, Any]) -> None:
    (root / "stretch_results.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    (root / "stretch_summary.md").write_text(build_markdown(payload), encoding="utf-8")
def main() -> None:
    root = Path(__file__).resolve().parent
    fixtures = build_scenario_fixtures()
    base_metrics = list(_find_fixture(fixtures, 6).datasets["calibration"])
    hybrid_fn = lambda comps: aggregate_hybrid_patched(comps, BASELINE_HYBRID_CONFIG, partial(bf_norm_log_scaled, c=BF_NORM_C))
    scorer = lambda comps: hybrid_fn(comps).aggregate_score
    # 4-phase flow: 0->1->2->3. Any phase0 failure raises and aborts.
    phase0 = phase0_baseline_gate(fixtures, hybrid_fn, root)
    phase1 = phase1_calibration(base_metrics, scorer)
    phase2 = phase2_correlation(base_metrics, hybrid_fn)
    payload = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "hybrid_config": {
            "aggregation": "HTG gating + Fisher product",
            "aggregation_impl": "aggregate_hybrid_patched",
            "fisher_n_terms": 1,
            "bf_norm_family": "log_scaled",
            "bf_norm_c": BF_NORM_C,
            "bf_max_target": BF_MAX_TARGET,
            "base_config": asdict(BASELINE_HYBRID_CONFIG),
        },
        "phase0_baseline_gate": phase0,
        "phase1_calibration": phase1,
        "phase2_correlation": phase2,
        "determinism_note": "All fields except generated_at_utc are deterministic; phase 2 seeds random=42 at phase start.",
    }
    write_outputs(root, payload)
    print(f"Wrote {root / 'stretch_results.json'}")
    print(f"Wrote {root / 'stretch_summary.md'}")
    print(f"Phase 0 status: {phase0['status']} ({phase0['scenario_count']} scenarios)")
    print(f"rho=0.5 pass: {_fmt(phase2['pass_at_rho_0_5'])}; all-rho floor clear: {_fmt(phase2['all_rho_floor_not_saturated'])}")
if __name__ == "__main__":
    main()
