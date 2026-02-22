from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass, replace
from datetime import datetime, timezone
from pathlib import Path
from statistics import median
from typing import Iterable

from candidates import HybridConfig, aggregate_hybrid
from evaluate import ScenarioCellResult, evaluate_fixture
from models import (
    DivergenceKind,
    EffectDirection,
    MetricComponent,
    NoUncertainty,
    SigmoidParams,
    Summary,
    UncertaintySummary,
)
from normalization import NormalizationConfig
from scenarios import (
    DEFAULT_CUSTOM_SIGMOIDS,
    ScenarioFixture,
    _metric,
    _no_uncertainty,
    _summary,
    build_scenario_fixtures,
)


BASELINE_HYBRID_CONFIG = HybridConfig(
    normalization=NormalizationConfig(custom_sigmoids=dict(DEFAULT_CUSTOM_SIGMOIDS))
)

EXPECTED_S2_MARGIN = 0.07280374089261333
EXPECTED_S4_RELATIVE_DELTA = 0.0719926034986539
SANITY_ABS_TOL = 1e-6

S2_CUSTOM_K = [1.0, 1.5, 2.0, 2.2, 2.5, 3.0]
S2_CUSTOM_X0 = [-0.2, 0.0, 0.2, 0.5]
S2_SE_MULT = [0.5, 0.75, 1.0, 1.5, 2.0]
S5_BF_VALUES = [80.0, 90.0, 100.0, 110.0, 120.0, 150.0, 200.0, 500.0, 1000.0]
S7_BOUNDARY_SE = [0.25, 0.30, 0.40, 0.50, 0.70, 0.90, 1.20]
S6_D_MID = [0.9, 2.0, 3.0, 4.0]
S6_BF_STRONG = [12.0, 100.0, 500.0, 1000.0]
S4_MISSING_COUNTS = [1, 2, 3, 4]
S1_SE_MULT = [1.0, 1.5, 3.0, 5.0, 10.0]


@dataclass(frozen=True)
class PerturbationResult:
    scenario_idx: int
    axis: str
    label: str
    passed: bool
    margin: float
    margin_label: str
    raw_scores: dict[str, float]
    is_baseline: bool


def _fixture_by_idx() -> dict[int, ScenarioFixture]:
    return {fixture.idx: fixture for fixture in build_scenario_fixtures()}


def _direction_to_name(direction: EffectDirection | None) -> str | None:
    return direction.name if direction is not None else None


def _summary_fields(component: MetricComponent) -> tuple[int | None, float | None]:
    if component.uncertainty is not None and isinstance(component.uncertainty.point, Summary):
        point = component.uncertainty.point
        return point.sample_size, point.standard_error
    return None, None


def _rebuild_metric(
    component: MetricComponent,
    *,
    value: float | None = None,
    standard_error: float | None = None,
) -> MetricComponent:
    sample_size, existing_se = _summary_fields(component)
    target_se = existing_se if standard_error is None else standard_error
    return _metric(
        kind=component.kind,
        value=component.value if value is None else value,
        direction=_direction_to_name(component.direction),
        sample_size=sample_size,
        standard_error=target_se,
        method_ref=component.method_ref,
        units=component.units,
        component_sample_size=component.sample_size,
    )


def build_s2_custom_sigmoid_config(k: float, x0: float) -> HybridConfig:
    custom_sigmoids = dict(DEFAULT_CUSTOM_SIGMOIDS)
    custom_sigmoids["s2.custom.1"] = SigmoidParams(k=k, x0=x0)
    normalization = replace(BASELINE_HYBRID_CONFIG.normalization, custom_sigmoids=custom_sigmoids)
    return replace(BASELINE_HYBRID_CONFIG, normalization=normalization)


def build_s2_se_scaled_fixture(se_mult: float) -> ScenarioFixture:
    base = _fixture_by_idx()[2]
    components: list[MetricComponent] = []
    for component in base.datasets["unanimous"]:
        sample_size, se = _summary_fields(component)
        if se is None:
            raise ValueError(f"S2 component {component.method_ref} missing standard error")
        scaled_se = se if component.method_ref == "s2.custom.1" else se * se_mult
        components.append(
            _metric(
                kind=component.kind,
                value=component.value,
                direction=_direction_to_name(component.direction),
                sample_size=sample_size,
                standard_error=scaled_se,
                method_ref=component.method_ref,
                units=component.units,
                component_sample_size=component.sample_size,
            )
        )
    return ScenarioFixture(
        idx=base.idx,
        name=base.name,
        what_it_tests=base.what_it_tests,
        pass_criterion=base.pass_criterion,
        datasets={"unanimous": components},
    )


def build_s5_bf_variant(bf_value: float) -> ScenarioFixture:
    base = _fixture_by_idx()[5]
    components: list[MetricComponent] = []
    for component in base.datasets["heterogeneous"]:
        value = bf_value if component.method_ref == "s5.bf.1" else component.value
        components.append(_rebuild_metric(component, value=value))
    return ScenarioFixture(
        idx=base.idx,
        name=base.name,
        what_it_tests=base.what_it_tests,
        pass_criterion=base.pass_criterion,
        datasets={"heterogeneous": components},
    )


def build_s6_compress_variant(d_mid_value: float, bf_strong_value: float) -> ScenarioFixture:
    base = _fixture_by_idx()[6]
    components: list[MetricComponent] = []
    for component in base.datasets["calibration"]:
        value = component.value
        if component.method_ref == "s6.d.mid":
            value = d_mid_value
        elif component.method_ref == "s6.bf.strong":
            value = bf_strong_value
        components.append(_rebuild_metric(component, value=value))
    return ScenarioFixture(
        idx=base.idx,
        name=base.name,
        what_it_tests=base.what_it_tests,
        pass_criterion=base.pass_criterion,
        datasets={"calibration": components},
    )


def build_s7_boundary_se_variant(boundary_se: float) -> ScenarioFixture:
    base = _fixture_by_idx()[7]
    boundary_components: list[MetricComponent] = []
    for component in base.datasets["boundary"]:
        sample_size, se = _summary_fields(component)
        if se is None:
            raise ValueError(f"S7 boundary component {component.method_ref} missing standard error")
        target_se = boundary_se if component.method_ref == "s7.z.boundary" else se
        boundary_components.append(
            _metric(
                kind=component.kind,
                value=component.value,
                direction=_direction_to_name(component.direction),
                sample_size=sample_size,
                standard_error=target_se,
                method_ref=component.method_ref,
                units=component.units,
                component_sample_size=component.sample_size,
            )
        )
    non_boundary_components = [_rebuild_metric(component) for component in base.datasets["non_boundary"]]
    return ScenarioFixture(
        idx=base.idx,
        name=base.name,
        what_it_tests=base.what_it_tests,
        pass_criterion=base.pass_criterion,
        datasets={
            "boundary": boundary_components,
            "non_boundary": non_boundary_components,
        },
    )


def build_s4_missing_count_variant(n_missing: int) -> ScenarioFixture:
    if n_missing < 1 or n_missing > 4:
        raise ValueError("S4 missing count must be between 1 and 4")
    base = _fixture_by_idx()[4]
    baseline_full = list(base.datasets["baseline_full"])
    missing_components: list[MetricComponent] = []
    for idx, component in enumerate(baseline_full):
        sample_size, se = _summary_fields(component)
        if se is None:
            raise ValueError(f"S4 baseline component {component.method_ref} missing standard error")
        if idx >= n_missing:
            missing_components.append(_rebuild_metric(component))
            continue
        uncertainty: UncertaintySummary | None
        component_sample_size = component.sample_size
        if idx == 0:
            uncertainty = _no_uncertainty("simulator omitted SE")
            component_sample_size = sample_size
        elif idx == 1:
            uncertainty = _summary(sample_size=sample_size, standard_error=None, method_ref=f"{component.method_ref}.unc")
            component_sample_size = None
        elif idx == 2:
            uncertainty = None
            component_sample_size = sample_size
        else:
            uncertainty = _no_uncertainty("simulator omitted SE")
            component_sample_size = sample_size
        missing_components.append(
            MetricComponent(
                kind=component.kind,
                value=component.value,
                direction=component.direction,
                uncertainty=uncertainty,
                sample_size=component_sample_size,
                units=component.units,
                method_ref=component.method_ref,
            )
        )
    baseline_full_components = [_rebuild_metric(component) for component in baseline_full]
    return ScenarioFixture(
        idx=base.idx,
        name=base.name,
        what_it_tests=base.what_it_tests,
        pass_criterion=base.pass_criterion,
        datasets={
            "missing": missing_components,
            "baseline_full": baseline_full_components,
        },
    )


def build_s1_se_variant(se_mult: float) -> ScenarioFixture:
    base = _fixture_by_idx()[1]

    def _scaled(component: MetricComponent) -> MetricComponent:
        sample_size, se = _summary_fields(component)
        if se is None:
            raise ValueError(f"S1 component {component.method_ref} missing standard error")
        return _metric(
            kind=component.kind,
            value=component.value,
            direction=_direction_to_name(component.direction),
            sample_size=sample_size,
            standard_error=se * se_mult,
            method_ref=component.method_ref,
            units=component.units,
            component_sample_size=component.sample_size,
        )

    return ScenarioFixture(
        idx=base.idx,
        name=base.name,
        what_it_tests=base.what_it_tests,
        pass_criterion=base.pass_criterion,
        datasets={
            "base": [_scaled(base.datasets["base"][0])],
            "doubled": [_scaled(base.datasets["doubled"][0])],
        },
    )


def _evaluate_hybrid(fixture: ScenarioFixture, config: HybridConfig) -> ScenarioCellResult:
    return evaluate_fixture(
        fixture,
        "Hybrid",
        lambda components, cfg=config: aggregate_hybrid(components, cfg),
    )


def _margin_from_cell(cell: ScenarioCellResult) -> tuple[float, str]:
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


def _perturbation_result(
    axis: str,
    label: str,
    fixture: ScenarioFixture,
    *,
    config: HybridConfig,
    is_baseline: bool,
) -> PerturbationResult:
    cell = _evaluate_hybrid(fixture, config)
    margin, margin_label = _margin_from_cell(cell)
    return PerturbationResult(
        scenario_idx=fixture.idx,
        axis=axis,
        label=label,
        passed=cell.passed,
        margin=margin,
        margin_label=margin_label,
        raw_scores={key: float(value) for key, value in cell.raw_scores.items()},
        is_baseline=is_baseline,
    )


def _run_baseline_sanity(fixtures: dict[int, ScenarioFixture]) -> dict[str, float]:
    s2_cell = _evaluate_hybrid(fixtures[2], BASELINE_HYBRID_CONFIG)
    s2_margin, _ = _margin_from_cell(s2_cell)
    s4_cell = _evaluate_hybrid(fixtures[4], BASELINE_HYBRID_CONFIG)
    s4_delta = float(s4_cell.raw_scores["relative_delta"])
    if not math.isclose(s2_margin, EXPECTED_S2_MARGIN, abs_tol=SANITY_ABS_TOL):
        raise AssertionError(
            f"S2 sanity check mismatch: expected {EXPECTED_S2_MARGIN:.12f}, got {s2_margin:.12f}"
        )
    if not math.isclose(s4_delta, EXPECTED_S4_RELATIVE_DELTA, abs_tol=SANITY_ABS_TOL):
        raise AssertionError(
            f"S4 sanity check mismatch: expected {EXPECTED_S4_RELATIVE_DELTA:.12f}, got {s4_delta:.12f}"
        )
    ratio = float(s2_cell.raw_scores["aggregate"]) / float(s2_cell.raw_scores["max_single"])
    return {
        "s2_margin": s2_margin,
        "s2_ratio": ratio,
        "s2_aggregate": float(s2_cell.raw_scores["aggregate"]),
        "s2_max_single": float(s2_cell.raw_scores["max_single"]),
        "s4_relative_delta": s4_delta,
    }


def _axis_stats(results: Iterable[PerturbationResult]) -> dict[str, float | int]:
    rows = list(results)
    margins = [row.margin for row in rows]
    pass_count = sum(1 for row in rows if row.passed)
    total = len(rows)
    return {
        "total": total,
        "pass_count": pass_count,
        "fail_count": total - pass_count,
        "pass_rate": pass_count / total if total else 0.0,
        "min_margin": min(margins) if margins else 0.0,
        "median_margin": median(margins) if margins else 0.0,
        "max_margin": max(margins) if margins else 0.0,
    }


def _tipping_1d(points: list[tuple[float, PerturbationResult]], label: str) -> list[str]:
    lines: list[str] = []
    ordered = sorted(points, key=lambda item: item[0])
    for (left_x, left_res), (right_x, right_res) in zip(ordered, ordered[1:]):
        if left_res.passed != right_res.passed:
            lines.append(
                f"{label} {left_x:g}->{right_x:g}: {('PASS' if left_res.passed else 'FAIL')} -> "
                f"{('PASS' if right_res.passed else 'FAIL')}"
            )
    return lines


def _tipping_s2_custom(points: list[tuple[float, float, PerturbationResult]]) -> list[str]:
    by_x0: dict[float, list[tuple[float, PerturbationResult]]] = {}
    for k, x0, result in points:
        by_x0.setdefault(x0, []).append((k, result))
    lines: list[str] = []
    for x0 in sorted(by_x0):
        for tip in _tipping_1d(by_x0[x0], label=f"x0={x0:g}, k"):
            lines.append(tip)
    return lines


def _tipping_s6_joint(points: list[tuple[float, float, PerturbationResult]]) -> list[str]:
    table = {(d_mid, bf): result for d_mid, bf, result in points}
    lines: list[str] = []
    for d_mid in S6_D_MID:
        one_dim = [(bf, table[(d_mid, bf)]) for bf in S6_BF_STRONG]
        for tip in _tipping_1d(one_dim, label=f"d_mid={d_mid:g}, bf"):
            lines.append(tip)
    for bf in S6_BF_STRONG:
        one_dim = [(d_mid, table[(d_mid, bf)]) for d_mid in S6_D_MID]
        for tip in _tipping_1d(one_dim, label=f"bf={bf:g}, d_mid"):
            lines.append(tip)
    deduped: list[str] = []
    seen: set[str] = set()
    for line in lines:
        if line in seen:
            continue
        seen.add(line)
        deduped.append(line)
    return deduped


def _build_summary_markdown(
    fixtures: dict[int, ScenarioFixture],
    baseline_sanity: dict[str, float],
    axis_results: dict[str, list[PerturbationResult]],
    s2_custom_points: list[tuple[float, float, PerturbationResult]],
    s5_bf_points: list[tuple[float, PerturbationResult]],
    tipping_points: dict[str, list[str]],
) -> str:
    lines: list[str] = []
    lines.append("# Hybrid Perturbation Robustness (Session 4)\n\n")
    lines.append(f"Generated: {datetime.now(timezone.utc).isoformat()}\n\n")
    lines.append("## Baseline Sanity Checks\n\n")
    lines.append(
        f"- S2 margin `((agg/max_single)/1.5)-1`: `{baseline_sanity['s2_margin']:+.6f}` "
        f"(ratio `{baseline_sanity['s2_ratio']:.6f}`)\n"
    )
    lines.append(
        f"- S4 relative delta: `{baseline_sanity['s4_relative_delta']:.6f}`\n"
    )
    lines.append("\n## Top-Level Verdict\n\n")
    lines.append("| Axis | Scenario | Pass rate | Pass/Total | Min margin | Median margin |\n")
    lines.append("| :--- | :--- | ---: | ---: | ---: | ---: |\n")
    for axis, results in axis_results.items():
        stats = _axis_stats(results)
        scenario_name = fixtures[results[0].scenario_idx].name
        lines.append(
            f"| `{axis}` | S{results[0].scenario_idx} {scenario_name} | "
            f"{100.0 * float(stats['pass_rate']):.1f}% | "
            f"{int(stats['pass_count'])}/{int(stats['total'])} | "
            f"{float(stats['min_margin']):+.6f} | {float(stats['median_margin']):+.6f} |\n"
        )

    lines.append("\n## Critical Axis: S2 Custom Sigmoid Margin Grid\n\n")
    lines.append("| k \\\\ x0 | -0.2 | 0.0 | 0.2 | 0.5 |\n")
    lines.append("| ---: | ---: | ---: | ---: | ---: |\n")
    grid = {(k, x0): result for k, x0, result in s2_custom_points}
    for k in S2_CUSTOM_K:
        cells: list[str] = []
        for x0 in S2_CUSTOM_X0:
            result = grid[(k, x0)]
            cells.append(f"{result.margin:+.5f} ({'PASS' if result.passed else 'FAIL'})")
        lines.append(f"| {k:.1f} | " + " | ".join(cells) + " |\n")

    lines.append("\n## Critical Axis: S5 BayesFactor Sweep\n\n")
    lines.append("| BF value | Margin | Pass | Max component score | Min component score |\n")
    lines.append("| ---: | ---: | :---: | ---: | ---: |\n")
    for bf_value, result in sorted(s5_bf_points, key=lambda item: item[0]):
        component_scores = [value for key, value in result.raw_scores.items() if key != "aggregate"]
        lines.append(
            f"| {bf_value:.0f} | {result.margin:+.6f} | {'PASS' if result.passed else 'FAIL'} | "
            f"{max(component_scores):.6f} | {min(component_scores):.6f} |\n"
        )

    lines.append("\n## Tipping Points\n\n")
    for axis in ["s2_custom_sigmoid", "s5_bayes_factor", "s6_joint_compress"]:
        lines.append(f"- `{axis}`\n")
        tips = tipping_points.get(axis, [])
        if tips:
            for tip in tips:
                lines.append(f"  - {tip}\n")
        else:
            lines.append("  - No pass/fail transitions observed within sampled points.\n")
    return "".join(lines)


def main() -> None:
    fixtures = _fixture_by_idx()
    baseline_sanity = _run_baseline_sanity(fixtures)

    axis_results: dict[str, list[PerturbationResult]] = {}
    s2_custom_points: list[tuple[float, float, PerturbationResult]] = []
    s2_se_points: list[tuple[float, PerturbationResult]] = []
    s5_bf_points: list[tuple[float, PerturbationResult]] = []
    s7_boundary_points: list[tuple[float, PerturbationResult]] = []
    s6_joint_points: list[tuple[float, float, PerturbationResult]] = []
    s4_missing_points: list[tuple[float, PerturbationResult]] = []
    s1_se_points: list[tuple[float, PerturbationResult]] = []

    axis = "s2_custom_sigmoid"
    axis_results[axis] = []
    for k in S2_CUSTOM_K:
        for x0 in S2_CUSTOM_X0:
            result = _perturbation_result(
                axis,
                label=f"k={k:.1f},x0={x0:.1f}",
                fixture=fixtures[2],
                config=build_s2_custom_sigmoid_config(k, x0),
                is_baseline=(math.isclose(k, 2.2) and math.isclose(x0, 0.0)),
            )
            axis_results[axis].append(result)
            s2_custom_points.append((k, x0, result))

    axis = "s2_non_custom_se_scale"
    axis_results[axis] = []
    for mult in S2_SE_MULT:
        result = _perturbation_result(
            axis,
            label=f"mult={mult:.2f}",
            fixture=build_s2_se_scaled_fixture(mult),
            config=BASELINE_HYBRID_CONFIG,
            is_baseline=math.isclose(mult, 1.0),
        )
        axis_results[axis].append(result)
        s2_se_points.append((mult, result))

    axis = "s5_bayes_factor"
    axis_results[axis] = []
    for bf_value in S5_BF_VALUES:
        result = _perturbation_result(
            axis,
            label=f"bf={bf_value:.0f}",
            fixture=build_s5_bf_variant(bf_value),
            config=BASELINE_HYBRID_CONFIG,
            is_baseline=math.isclose(bf_value, 100.0),
        )
        axis_results[axis].append(result)
        s5_bf_points.append((bf_value, result))

    axis = "s7_boundary_se"
    axis_results[axis] = []
    for boundary_se in S7_BOUNDARY_SE:
        result = _perturbation_result(
            axis,
            label=f"boundary_se={boundary_se:.2f}",
            fixture=build_s7_boundary_se_variant(boundary_se),
            config=BASELINE_HYBRID_CONFIG,
            is_baseline=math.isclose(boundary_se, 1.20),
        )
        axis_results[axis].append(result)
        s7_boundary_points.append((boundary_se, result))

    axis = "s6_joint_compress"
    axis_results[axis] = []
    for d_mid in S6_D_MID:
        for bf_strong in S6_BF_STRONG:
            result = _perturbation_result(
                axis,
                label=f"d_mid={d_mid:.1f},bf_strong={bf_strong:.1f}",
                fixture=build_s6_compress_variant(d_mid, bf_strong),
                config=BASELINE_HYBRID_CONFIG,
                is_baseline=(math.isclose(d_mid, 0.9) and math.isclose(bf_strong, 12.0)),
            )
            axis_results[axis].append(result)
            s6_joint_points.append((d_mid, bf_strong, result))

    axis = "s4_missing_count"
    axis_results[axis] = []
    for n_missing in S4_MISSING_COUNTS:
        result = _perturbation_result(
            axis,
            label=f"n_missing={n_missing}",
            fixture=build_s4_missing_count_variant(n_missing),
            config=BASELINE_HYBRID_CONFIG,
            is_baseline=(n_missing == 3),
        )
        axis_results[axis].append(result)
        s4_missing_points.append((float(n_missing), result))

    axis = "s1_se_mult"
    axis_results[axis] = []
    for mult in S1_SE_MULT:
        result = _perturbation_result(
            axis,
            label=f"mult={mult:.1f}",
            fixture=build_s1_se_variant(mult),
            config=BASELINE_HYBRID_CONFIG,
            is_baseline=math.isclose(mult, 1.0),
        )
        axis_results[axis].append(result)
        s1_se_points.append((mult, result))

    tipping_points = {
        "s2_custom_sigmoid": _tipping_s2_custom(s2_custom_points),
        "s5_bayes_factor": _tipping_1d(s5_bf_points, label="bf"),
        "s6_joint_compress": _tipping_s6_joint(s6_joint_points),
    }

    payload = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "baseline_hybrid_config": asdict(BASELINE_HYBRID_CONFIG),
        "sanity_checks": baseline_sanity,
        "axes": {
            axis_name: {
                "scenario_idx": results[0].scenario_idx,
                "scenario_name": fixtures[results[0].scenario_idx].name,
                "stats": _axis_stats(results),
                "results": [asdict(row) for row in results],
            }
            for axis_name, results in axis_results.items()
        },
        "tipping_points": tipping_points,
        "run_count": sum(len(results) for results in axis_results.values()),
    }

    summary_md = _build_summary_markdown(
        fixtures=fixtures,
        baseline_sanity=baseline_sanity,
        axis_results=axis_results,
        s2_custom_points=s2_custom_points,
        s5_bf_points=s5_bf_points,
        tipping_points=tipping_points,
    )

    here = Path(__file__).resolve().parent
    results_path = here / "perturbation_results.json"
    summary_path = here / "perturbation_summary.md"
    results_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    summary_path.write_text(summary_md, encoding="utf-8")

    print(f"S2 sanity margin: {baseline_sanity['s2_margin']:+.6f}")
    print(f"S4 sanity delta: {baseline_sanity['s4_relative_delta']:.6f}")
    print(f"Total perturbation runs: {payload['run_count']}")
    print(f"Wrote {results_path}")
    print(f"Wrote {summary_path}")


if __name__ == "__main__":
    main()
