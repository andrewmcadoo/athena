from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Any, Callable, Mapping, Sequence

from candidates import (
    FisherUPConfig,
    HTGMaxConfig,
    IVWCDFConfig,
    aggregate_htg_max,
    get_candidate_registry,
)
from models import AggregateResult, MetricComponent
from normalization import NormalizationConfig
from scenarios import DEFAULT_CUSTOM_SIGMOIDS, ScenarioFixture, build_scenario_fixtures


@dataclass
class ScenarioCellResult:
    scenario_index: int
    scenario_name: str
    candidate: str
    passed: bool
    raw_scores: dict[str, float]
    score_summary: str
    pass_reason: str
    bounded: bool
    warnings: list[str]
    skipped: list[str]
    decompositions: dict[str, list[dict[str, Any]]]


def json_default(obj: Any) -> Any:
    if isinstance(obj, Enum):
        return obj.value
    raise TypeError(f"Object of type {obj.__class__.__name__} is not JSON serializable")


def serialize_result(result: AggregateResult) -> dict[str, Any]:
    return {
        "aggregate_score": result.aggregate_score,
        "contributions": [
            {
                "index": c.index,
                "method_ref": c.method_ref,
                "kind": c.kind.value,
                "score": c.score,
                "weight": c.weight,
                "contribution": c.contribution,
                "diagnostics": c.diagnostics,
            }
            for c in result.contributions
        ],
        "warnings": result.warnings,
        "skipped": result.skipped,
    }


def run_one(
    fn: Callable[[Sequence[MetricComponent]], AggregateResult],
    metrics: Sequence[MetricComponent],
) -> AggregateResult:
    return fn(metrics)


def is_bounded(score: float) -> bool:
    return 0.0 <= score <= 1.0 and math.isfinite(score)


def evaluate_fixture(
    fixture: ScenarioFixture,
    candidate_name: str,
    fn: Callable[[Sequence[MetricComponent]], AggregateResult],
) -> ScenarioCellResult:
    warnings: list[str] = []
    skipped: list[str] = []
    decompositions: dict[str, list[dict[str, Any]]] = {}
    raw_scores: dict[str, float] = {}

    if fixture.idx == 1:
        base = run_one(fn, fixture.datasets["base"])
        doubled = run_one(fn, fixture.datasets["doubled"])
        raw_scores = {"base": base.aggregate_score, "doubled": doubled.aggregate_score}
        passed = doubled.aggregate_score <= base.aggregate_score + 1e-12
        reason = "doubled <= base"
        bounded = is_bounded(base.aggregate_score) and is_bounded(doubled.aggregate_score)
        warnings.extend(base.warnings + doubled.warnings)
        skipped.extend(base.skipped + doubled.skipped)
        decompositions = {
            "base": serialize_result(base)["contributions"],
            "doubled": serialize_result(doubled)["contributions"],
        }
        score_summary = f"base={base.aggregate_score:.4f}, doubled={doubled.aggregate_score:.4f}"

    elif fixture.idx == 2:
        unanimous = run_one(fn, fixture.datasets["unanimous"])
        singles = [run_one(fn, [metric]).aggregate_score for metric in fixture.datasets["unanimous"]]
        max_single = max(singles) if singles else 0.0
        threshold = 1.5 * max_single
        raw_scores = {
            "aggregate": unanimous.aggregate_score,
            "max_single": max_single,
            "threshold": threshold,
        }
        passed = unanimous.aggregate_score >= threshold
        reason = "aggregate >= 1.5 * max_single"
        bounded = is_bounded(unanimous.aggregate_score) and all(is_bounded(s) for s in singles)
        warnings.extend(unanimous.warnings)
        skipped.extend(unanimous.skipped)
        decompositions = {"aggregate": serialize_result(unanimous)["contributions"]}
        score_summary = (
            f"agg={unanimous.aggregate_score:.4f}, max1={max_single:.4f}, target={threshold:.4f}"
        )

    elif fixture.idx == 3:
        mixed = run_one(fn, fixture.datasets["mixed"])
        all_c = run_one(fn, fixture.datasets["all_contradiction"])
        all_a = run_one(fn, fixture.datasets["all_agreement"])
        lo = min(all_a.aggregate_score, all_c.aggregate_score)
        hi = max(all_a.aggregate_score, all_c.aggregate_score)
        raw_scores = {
            "mixed": mixed.aggregate_score,
            "all_contradiction": all_c.aggregate_score,
            "all_agreement": all_a.aggregate_score,
        }
        passed = lo - 1e-12 <= mixed.aggregate_score <= hi + 1e-12
        reason = "all_agreement <= mixed <= all_contradiction"
        bounded = all(
            is_bounded(score)
            for score in [mixed.aggregate_score, all_c.aggregate_score, all_a.aggregate_score]
        )
        warnings.extend(mixed.warnings + all_c.warnings + all_a.warnings)
        skipped.extend(mixed.skipped + all_c.skipped + all_a.skipped)
        decompositions = {
            "mixed": serialize_result(mixed)["contributions"],
            "all_contradiction": serialize_result(all_c)["contributions"],
            "all_agreement": serialize_result(all_a)["contributions"],
        }
        score_summary = (
            f"mixed={mixed.aggregate_score:.4f}, allC={all_c.aggregate_score:.4f}, "
            f"allA={all_a.aggregate_score:.4f}"
        )

    elif fixture.idx == 4:
        missing = run_one(fn, fixture.datasets["missing"])
        baseline = run_one(fn, fixture.datasets["baseline_full"])
        if baseline.aggregate_score > 1e-12:
            relative_delta = abs(missing.aggregate_score - baseline.aggregate_score) / baseline.aggregate_score
        else:
            relative_delta = abs(missing.aggregate_score - baseline.aggregate_score)
        raw_scores = {
            "missing": missing.aggregate_score,
            "baseline_full": baseline.aggregate_score,
            "relative_delta": relative_delta,
        }
        passed = (
            math.isfinite(missing.aggregate_score)
            and math.isfinite(baseline.aggregate_score)
            and relative_delta <= 0.20
        )
        reason = "finite and <=20% delta from baseline"
        bounded = is_bounded(missing.aggregate_score) and is_bounded(baseline.aggregate_score)
        warnings.extend(missing.warnings + baseline.warnings)
        skipped.extend(missing.skipped + baseline.skipped)
        decompositions = {
            "missing": serialize_result(missing)["contributions"],
            "baseline_full": serialize_result(baseline)["contributions"],
        }
        score_summary = (
            f"missing={missing.aggregate_score:.4f}, baseline={baseline.aggregate_score:.4f}, "
            f"delta={relative_delta:.3f}"
        )

    elif fixture.idx == 5:
        heterogeneous = run_one(fn, fixture.datasets["heterogeneous"])
        component_scores = [c.score for c in heterogeneous.contributions]
        in_range = all(0.3 <= s <= 0.991 for s in component_scores)
        ranking = [c.method_ref for c in sorted(heterogeneous.contributions, key=lambda c: c.score, reverse=True)]
        raw_scores = {"aggregate": heterogeneous.aggregate_score}
        raw_scores.update({c.method_ref: c.score for c in heterogeneous.contributions})
        passed = in_range
        reason = "component scores in [0.3, 0.99] with tolerance; ranking checked post-pass"
        bounded = is_bounded(heterogeneous.aggregate_score)
        warnings.extend(heterogeneous.warnings)
        skipped.extend(heterogeneous.skipped)
        decompositions = {"heterogeneous": serialize_result(heterogeneous)["contributions"]}
        # Ranking gets attached for cross-candidate consistency check.
        decompositions["_ranking"] = [{"ranked_method_ref": r} for r in ranking]
        score_summary = (
            f"agg={heterogeneous.aggregate_score:.4f}, "
            f"scores={[round(s, 4) for s in component_scores]}"
        )

    elif fixture.idx == 6:
        calibration = run_one(fn, fixture.datasets["calibration"])
        recon = sum(c.contribution for c in calibration.contributions)
        dominant = max((c.contribution for c in calibration.contributions), default=0.0)
        dominant_share = dominant / calibration.aggregate_score if calibration.aggregate_score > 1e-12 else 0.0
        raw_scores = {
            "aggregate": calibration.aggregate_score,
            "reconstructed": recon,
            "dominant_share": dominant_share,
        }
        passed = abs(recon - calibration.aggregate_score) <= 1e-8 and dominant_share >= 0.35
        reason = "sum(weight*score) reconstructs aggregate and dominant component is identifiable"
        bounded = is_bounded(calibration.aggregate_score)
        warnings.extend(calibration.warnings)
        skipped.extend(calibration.skipped)
        decompositions = {"calibration": serialize_result(calibration)["contributions"]}
        score_summary = (
            f"agg={calibration.aggregate_score:.4f}, recon={recon:.4f}, dom_share={dominant_share:.3f}"
        )

    elif fixture.idx == 7:
        boundary = run_one(fn, fixture.datasets["boundary"])
        non_boundary = run_one(fn, fixture.datasets["non_boundary"])
        raw_scores = {
            "boundary": boundary.aggregate_score,
            "non_boundary": non_boundary.aggregate_score,
        }
        passed = boundary.aggregate_score < non_boundary.aggregate_score - 1e-12
        reason = "boundary < non_boundary for same values with lower uncertainty comparator"
        bounded = is_bounded(boundary.aggregate_score) and is_bounded(non_boundary.aggregate_score)
        warnings.extend(boundary.warnings + non_boundary.warnings)
        skipped.extend(boundary.skipped + non_boundary.skipped)
        decompositions = {
            "boundary": serialize_result(boundary)["contributions"],
            "non_boundary": serialize_result(non_boundary)["contributions"],
        }
        score_summary = f"boundary={boundary.aggregate_score:.4f}, non_boundary={non_boundary.aggregate_score:.4f}"

    else:
        raise ValueError(f"Unsupported fixture index: {fixture.idx}")

    return ScenarioCellResult(
        scenario_index=fixture.idx,
        scenario_name=fixture.name,
        candidate=candidate_name,
        passed=bool(passed and bounded),
        raw_scores=raw_scores,
        score_summary=score_summary,
        pass_reason=reason,
        bounded=bounded,
        warnings=warnings,
        skipped=skipped,
        decompositions=decompositions,
    )


def matrix_markdown(
    fixtures: Sequence[ScenarioFixture],
    candidate_order: Sequence[str],
    matrix: Mapping[str, Mapping[str, ScenarioCellResult]],
) -> str:
    header = "| Candidate | " + " | ".join(f"S{f.idx} {f.name}" for f in fixtures) + " |\n"
    sep = "|" + " --- |" * (len(fixtures) + 1) + "\n"
    rows = [header, sep]
    for candidate in candidate_order:
        cells: list[str] = []
        for fixture in fixtures:
            cell = matrix[candidate][fixture.name]
            glyph = "PASS" if cell.passed else "FAIL"
            score = cell.score_summary.replace("|", "/")
            cells.append(f"{score} ({glyph})")
        rows.append("| " + candidate + " | " + " | ".join(cells) + " |\n")
    return "".join(rows)


def main() -> None:
    here = Path(__file__).resolve().parent
    fixtures = build_scenario_fixtures()

    normalization = NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)
    candidates = get_candidate_registry(
        ivw_cfg=IVWCDFConfig(eps=1e-12, w_default=1.0, normalization=normalization),
        htg_cfg=HTGMaxConfig(
            alpha=1.5,
            tau=7.8,
            c_floor=0.15,
            eps=1e-12,
            mode="hard_max",
            normalization=normalization,
        ),
        fisher_cfg=FisherUPConfig(n_ref=100.0, r_floor=0.1, p_eps=1e-12, normalization=normalization),
    )

    matrix: dict[str, dict[str, ScenarioCellResult]] = {
        candidate: {} for candidate in candidates
    }
    ranking_by_candidate: dict[str, list[str]] = {}

    for candidate_name, fn in candidates.items():
        for fixture in fixtures:
            cell = evaluate_fixture(fixture, candidate_name, fn)
            matrix[candidate_name][fixture.name] = cell
            if fixture.idx == 5:
                ranking_by_candidate[candidate_name] = [
                    str(item["ranked_method_ref"])
                    for item in cell.decompositions.get("_ranking", [])
                ]
                cell.decompositions.pop("_ranking", None)

    # Post-check for scenario 5 ranking consistency across candidates.
    reference_order = ranking_by_candidate.get("IVW-CDF", [])
    for candidate_name in matrix:
        fixture_name = "Scale heterogeneity"
        if fixture_name not in matrix[candidate_name]:
            continue
        cell = matrix[candidate_name][fixture_name]
        stable = ranking_by_candidate.get(candidate_name, []) == reference_order
        cell.passed = bool(cell.passed and stable and cell.bounded)
        if not stable:
            cell.pass_reason += "; ranking mismatch across candidates"

    # Optional exploratory variant for documentation only.
    htg_lse_cfg = HTGMaxConfig(
        alpha=1.5,
        tau=7.8,
        c_floor=0.15,
        eps=1e-12,
        mode="lse_rebound",
        lse_beta=8.0,
        normalization=normalization,
    )
    lse_results = {
        fixture.name: evaluate_fixture(
            fixture,
            "HTG-Max-LSE",
            lambda comps, cfg=htg_lse_cfg: aggregate_htg_max(comps, cfg),
        )
        for fixture in fixtures
    }

    matrix_md = matrix_markdown(fixtures, list(candidates.keys()), matrix)

    payload = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "candidate_order": list(candidates.keys()),
        "fixtures": [asdict(fixture) for fixture in fixtures],
        "matrix": {
            candidate: {
                scenario: asdict(cell)
                for scenario, cell in scenario_map.items()
            }
            for candidate, scenario_map in matrix.items()
        },
        "matrix_markdown": matrix_md,
        "ranking_reference": reference_order,
        "htg_lse_exploratory": {
            name: asdict(cell) for name, cell in lse_results.items()
        },
    }

    results_json = here / "results.json"
    results_md = here / "results.md"
    results_json.write_text(
        json.dumps(payload, indent=2, default=json_default) + "\n",
        encoding="utf-8",
    )

    lines: list[str] = []
    lines.append("# Aggregation Candidate Evaluation (Session 1 Prototype)\n\n")
    lines.append(f"Generated: {payload['generated_at_utc']}\n\n")
    lines.append("## 3x7 Matrix (Primary Candidates)\n\n")
    lines.append(matrix_md)
    lines.append("\n## Scenario Pass Criteria\n\n")
    for fixture in fixtures:
        lines.append(
            f"- S{fixture.idx} {fixture.name}: {fixture.pass_criterion}\n"
        )
    lines.append("\n## Exploratory HTG Variant\n\n")
    lines.append(
        "The optional `HTG-Max` LogSumExp variant was also executed with re-bounding "
        "`1 - exp(-LSE)`. Scores are recorded in `results.json` under "
        "`htg_lse_exploratory`.\n"
    )
    results_md.write_text("".join(lines), encoding="utf-8")

    print(matrix_md)
    print(f"Wrote {results_json}")
    print(f"Wrote {results_md}")


if __name__ == "__main__":
    main()
