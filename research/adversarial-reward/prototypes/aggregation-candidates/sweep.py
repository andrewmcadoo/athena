from __future__ import annotations

import json
from dataclasses import asdict
from datetime import datetime, timezone
from itertools import product
from pathlib import Path
from statistics import mean
from typing import Any, Callable, Sequence

from candidates import (
    FisherUPConfig,
    HTGMaxConfig,
    IVWCDFConfig,
    aggregate_fisher_up,
    aggregate_htg_max,
    aggregate_ivw_cdf,
)
from evaluate import ScenarioCellResult, evaluate_fixture
from models import AggregateResult, MetricComponent, SigmoidParams
from normalization import NormalizationConfig
from scenarios import DEFAULT_CUSTOM_SIGMOIDS, ScenarioFixture, build_scenario_fixtures


CandidateFn = Callable[[Sequence[MetricComponent]], AggregateResult]

S2_MULTIPLIERS = [round(1.0 + 0.1 * step, 1) for step in range(11)]


def scenario_primary_score(cell: ScenarioCellResult) -> float:
    raw = cell.raw_scores
    if cell.scenario_index == 1:
        return float(raw.get("base", 0.0))
    if cell.scenario_index == 2:
        return float(raw.get("aggregate", 0.0))
    if cell.scenario_index == 3:
        return float(raw.get("mixed", 0.0))
    if cell.scenario_index == 4:
        return float(raw.get("missing", 0.0))
    if cell.scenario_index == 5:
        return float(raw.get("aggregate", 0.0))
    if cell.scenario_index == 6:
        return float(raw.get("aggregate", 0.0))
    if cell.scenario_index == 7:
        return float(raw.get("boundary", 0.0))
    return 0.0


def evaluate_candidate_config(
    fixtures: Sequence[ScenarioFixture],
    candidate_name: str,
    fn: CandidateFn,
) -> dict[str, Any]:
    scenario_results: list[dict[str, Any]] = []
    pass_scores: list[float] = []
    all_scores: list[float] = []
    failed_scenarios: list[str] = []
    s2_ratio: float | None = None

    for fixture in fixtures:
        cell = evaluate_fixture(fixture, candidate_name, fn)
        scenario_results.append(asdict(cell))
        primary_score = scenario_primary_score(cell)
        all_scores.append(primary_score)
        if cell.passed:
            pass_scores.append(primary_score)
        else:
            failed_scenarios.append(f"S{cell.scenario_index}")
        if cell.scenario_index == 2:
            aggregate = float(cell.raw_scores.get("aggregate", 0.0))
            max_single = float(cell.raw_scores.get("max_single", 0.0))
            if max_single > 0.0:
                s2_ratio = aggregate / max_single

    pass_count = sum(1 for result in scenario_results if bool(result["passed"]))
    return {
        "pass_count": pass_count,
        "failed_scenarios": failed_scenarios,
        "avg_pass_score": mean(pass_scores) if pass_scores else 0.0,
        "avg_all_score": mean(all_scores) if all_scores else 0.0,
        "scenario_results": scenario_results,
        "s2_ratio": s2_ratio,
    }


def build_normalization(
    *,
    abs_diff_k: float,
    abs_diff_x0: float,
    se_dampen_enabled: bool,
    se_dampen_k: float,
    se_dampen_x0: float,
) -> NormalizationConfig:
    return NormalizationConfig(
        absolute_difference_sigmoid=SigmoidParams(k=abs_diff_k, x0=abs_diff_x0),
        custom_sigmoids=dict(DEFAULT_CUSTOM_SIGMOIDS),
        clip_eps=1e-12,
        se_dampen_enabled=se_dampen_enabled,
        se_dampen_k=se_dampen_k,
        se_dampen_x0=se_dampen_x0,
    )


def summarize_stage1_record(record: dict[str, Any]) -> tuple[int, float]:
    return int(record["total_passes"]), float(record["avg_pass_score"])


def sort_records(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(
        records,
        key=lambda item: (
            int(item["evaluation"]["pass_count"]),
            float(item["evaluation"]["avg_pass_score"]),
            float(item["evaluation"]["avg_all_score"]),
        ),
        reverse=True,
    )


def top_n(records: list[dict[str, Any]], n: int = 5) -> list[dict[str, Any]]:
    return sort_records(records)[:n]


def stage1_sweep(fixtures: Sequence[ScenarioFixture]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], bool]:
    stage1_records: list[dict[str, Any]] = []
    norm_index = 0
    for abs_diff_k, abs_diff_x0, se_dampen_k, se_dampen_x0 in product(
        [800.0, 1200.0, 2000.0],
        [5e-4, 7e-4, 1e-3],
        [3.0, 5.0, 8.0],
        [1.0, 2.0, 3.0],
    ):
        norm_index += 1
        normalization = build_normalization(
            abs_diff_k=abs_diff_k,
            abs_diff_x0=abs_diff_x0,
            se_dampen_enabled=True,
            se_dampen_k=se_dampen_k,
            se_dampen_x0=se_dampen_x0,
        )
        norm_params = {
            "abs_diff_k": abs_diff_k,
            "abs_diff_x0": abs_diff_x0,
            "se_dampen_enabled": True,
            "se_dampen_k": se_dampen_k,
            "se_dampen_x0": se_dampen_x0,
        }

        candidate_evaluations: dict[str, dict[str, Any]] = {}
        all_pass_scores: list[float] = []
        total_passes = 0
        for candidate_name, fn in {
            "IVW-CDF": lambda comps, cfg=IVWCDFConfig(normalization=normalization): aggregate_ivw_cdf(comps, cfg),
            "HTG-Max": lambda comps, cfg=HTGMaxConfig(normalization=normalization): aggregate_htg_max(comps, cfg),
            "Fisher-UP": lambda comps, cfg=FisherUPConfig(normalization=normalization): aggregate_fisher_up(comps, cfg),
        }.items():
            evaluation = evaluate_candidate_config(fixtures, candidate_name, fn)
            candidate_evaluations[candidate_name] = evaluation
            total_passes += int(evaluation["pass_count"])
            if evaluation["pass_count"] > 0:
                all_pass_scores.append(float(evaluation["avg_pass_score"]))

        stage1_records.append(
            {
                "normalization_id": f"N{norm_index:03d}",
                "normalization": norm_params,
                "candidate_evaluations": candidate_evaluations,
                "total_passes": total_passes,
                "avg_pass_score": mean(all_pass_scores) if all_pass_scores else 0.0,
            }
        )

    ranked = sorted(stage1_records, key=summarize_stage1_record, reverse=True)
    clear_winner = True
    if len(ranked) > 1:
        first = summarize_stage1_record(ranked[0])
        second = summarize_stage1_record(ranked[1])
        clear_winner = first != second

    selected = ranked[:1] if clear_winner else ranked[:3]
    return stage1_records, selected, clear_winner


def build_stage2_record(
    *,
    candidate: str,
    sweep_family: str,
    normalization_id: str,
    normalization_params: dict[str, Any],
    config_params: dict[str, Any],
    evaluation: dict[str, Any],
) -> dict[str, Any]:
    return {
        "candidate": candidate,
        "sweep_family": sweep_family,
        "normalization_id": normalization_id,
        "normalization": normalization_params,
        "config": config_params,
        "evaluation": evaluation,
    }


def stage2_sweep_for_normalization(
    fixtures: Sequence[ScenarioFixture],
    normalization_id: str,
    normalization_params: dict[str, Any],
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    normalization_main = build_normalization(
        abs_diff_k=float(normalization_params["abs_diff_k"]),
        abs_diff_x0=float(normalization_params["abs_diff_x0"]),
        se_dampen_enabled=True,
        se_dampen_k=float(normalization_params["se_dampen_k"]),
        se_dampen_x0=float(normalization_params["se_dampen_x0"]),
    )
    normalization_isolation = build_normalization(
        abs_diff_k=float(normalization_params["abs_diff_k"]),
        abs_diff_x0=float(normalization_params["abs_diff_x0"]),
        se_dampen_enabled=False,
        se_dampen_k=float(normalization_params["se_dampen_k"]),
        se_dampen_x0=float(normalization_params["se_dampen_x0"]),
    )

    # IVW-CDF sweep (~60)
    for multiplicity_scale, multiplicity_threshold, w_default in product(
        [0.3, 0.5, 0.8, 1.0, 1.5],
        [0.05, 0.1, 0.2],
        [0.1, 0.5, 1.0, 2.0],
    ):
        cfg = IVWCDFConfig(
            eps=1e-12,
            w_default=w_default,
            multiplicity_bonus_enabled=True,
            multiplicity_threshold=multiplicity_threshold,
            multiplicity_scale=multiplicity_scale,
            normalization=normalization_main,
        )
        evaluation = evaluate_candidate_config(
            fixtures,
            "IVW-CDF",
            lambda comps, local_cfg=cfg: aggregate_ivw_cdf(comps, local_cfg),
        )
        records.append(
            build_stage2_record(
                candidate="IVW-CDF",
                sweep_family="ivw",
                normalization_id=normalization_id,
                normalization_params=normalization_params,
                config_params={
                    "eps": 1e-12,
                    "w_default": w_default,
                    "multiplicity_bonus_enabled": True,
                    "multiplicity_threshold": multiplicity_threshold,
                    "multiplicity_scale": multiplicity_scale,
                },
                evaluation=evaluation,
            )
        )

    # HTG-Max sweep (~180)
    for alpha, tau, c_floor, mode in product(
        [1.0, 1.5, 2.0],
        [5.0, 7.8, 12.0],
        [0.15, 0.3, 0.5, 0.7],
        ["hard_max", "lse_rebound", "soft_sum"],
    ):
        if mode == "lse_rebound":
            lse_values = [0.5, 2.0, 8.0]
        else:
            lse_values = [8.0]
        for lse_beta in lse_values:
            cfg = HTGMaxConfig(
                alpha=alpha,
                tau=tau,
                c_floor=c_floor,
                eps=1e-12,
                lse_beta=lse_beta,
                mode=mode,
                soft_sum_boost=2.0,
                normalization=normalization_main,
            )
            evaluation = evaluate_candidate_config(
                fixtures,
                "HTG-Max",
                lambda comps, local_cfg=cfg: aggregate_htg_max(comps, local_cfg),
            )
            records.append(
                build_stage2_record(
                    candidate="HTG-Max",
                    sweep_family="htg",
                    normalization_id=normalization_id,
                    normalization_params=normalization_params,
                    config_params={
                        "alpha": alpha,
                        "tau": tau,
                        "c_floor": c_floor,
                        "eps": 1e-12,
                        "mode": mode,
                        "lse_beta": lse_beta,
                        "soft_sum_boost": 2.0,
                    },
                    evaluation=evaluation,
                )
            )

    # Fisher-UP main sweep (~120) and isolation sweep (~120)
    for n_ref, r_floor, se_reliability_enabled in product(
        [50.0, 100.0, 200.0],
        [0.1, 0.3, 0.5, 0.7],
        [True, False],
    ):
        if se_reliability_enabled:
            se_param_grid = list(product([2.0, 3.0, 5.0], [1.5, 2.0, 3.0]))
        else:
            se_param_grid = [(3.0, 2.0)]
        for se_reliability_k, se_reliability_x0 in se_param_grid:
            cfg_main = FisherUPConfig(
                n_ref=n_ref,
                r_floor=r_floor,
                p_eps=1e-12,
                se_reliability_enabled=se_reliability_enabled,
                se_reliability_k=se_reliability_k,
                se_reliability_x0=se_reliability_x0,
                normalization=normalization_main,
            )
            evaluation_main = evaluate_candidate_config(
                fixtures,
                "Fisher-UP",
                lambda comps, local_cfg=cfg_main: aggregate_fisher_up(comps, local_cfg),
            )
            records.append(
                build_stage2_record(
                    candidate="Fisher-UP",
                    sweep_family="fisher_main",
                    normalization_id=normalization_id,
                    normalization_params=normalization_params,
                    config_params={
                        "n_ref": n_ref,
                        "r_floor": r_floor,
                        "p_eps": 1e-12,
                        "se_reliability_enabled": se_reliability_enabled,
                        "se_reliability_k": se_reliability_k,
                        "se_reliability_x0": se_reliability_x0,
                        "se_dampen_enabled": True,
                    },
                    evaluation=evaluation_main,
                )
            )

            cfg_isolation = FisherUPConfig(
                n_ref=n_ref,
                r_floor=r_floor,
                p_eps=1e-12,
                se_reliability_enabled=se_reliability_enabled,
                se_reliability_k=se_reliability_k,
                se_reliability_x0=se_reliability_x0,
                normalization=normalization_isolation,
            )
            evaluation_isolation = evaluate_candidate_config(
                fixtures,
                "Fisher-UP",
                lambda comps, local_cfg=cfg_isolation: aggregate_fisher_up(comps, local_cfg),
            )
            records.append(
                build_stage2_record(
                    candidate="Fisher-UP",
                    sweep_family="fisher_isolation",
                    normalization_id=normalization_id,
                    normalization_params=normalization_params,
                    config_params={
                        "n_ref": n_ref,
                        "r_floor": r_floor,
                        "p_eps": 1e-12,
                        "se_reliability_enabled": se_reliability_enabled,
                        "se_reliability_k": se_reliability_k,
                        "se_reliability_x0": se_reliability_x0,
                        "se_dampen_enabled": False,
                    },
                    evaluation=evaluation_isolation,
                )
            )

    return records


def filter_by_family(records: list[dict[str, Any]], family: str) -> list[dict[str, Any]]:
    return [record for record in records if record["sweep_family"] == family]


def best_record(records: list[dict[str, Any]]) -> dict[str, Any] | None:
    if not records:
        return None
    return sort_records(records)[0]


def s2_frontier_for_family(records: list[dict[str, Any]]) -> dict[str, Any]:
    qualifying: list[dict[str, Any]] = []
    for record in records:
        evaluation = record["evaluation"]
        if evaluation["pass_count"] == 6 and evaluation["failed_scenarios"] == ["S2"]:
            ratio = evaluation.get("s2_ratio")
            if ratio is not None:
                qualifying.append(record)

    if not qualifying:
        return {
            "qualifying_configs": 0,
            "best_ratio": None,
            "max_multiplier_passed": None,
            "best_record": None,
            "frontier": {},
        }

    best = max(qualifying, key=lambda item: float(item["evaluation"]["s2_ratio"]))
    ratio = float(best["evaluation"]["s2_ratio"])
    frontier = {f"{multiplier:.1f}": ratio + 1e-12 >= multiplier for multiplier in S2_MULTIPLIERS}
    passed = [multiplier for multiplier, ok in frontier.items() if ok]
    max_multiplier_passed = float(max(passed)) if passed else None
    return {
        "qualifying_configs": len(qualifying),
        "best_ratio": ratio,
        "max_multiplier_passed": max_multiplier_passed,
        "best_record": best,
        "frontier": frontier,
    }


def format_failed_scenarios(failed: list[str]) -> str:
    if not failed:
        return "none"
    return ",".join(failed)


def markdown_table_top5(title: str, records: list[dict[str, Any]]) -> list[str]:
    lines = [f"### {title}\n\n"]
    lines.append(
        "| Rank | Norm | Passes | Avg pass score | Failed scenarios | Config |\n"
    )
    lines.append("| --- | --- | --- | --- | --- | --- |\n")
    for rank, record in enumerate(top_n(records, 5), start=1):
        evaluation = record["evaluation"]
        lines.append(
            "| "
            f"{rank} | {record['normalization_id']} | {evaluation['pass_count']}/7 | "
            f"{evaluation['avg_pass_score']:.4f} | {format_failed_scenarios(evaluation['failed_scenarios'])} | "
            f"`{json.dumps(record['config'], sort_keys=True)}` |\n"
        )
    lines.append("\n")
    return lines


def write_summary(
    path: Path,
    *,
    stage1_records: list[dict[str, Any]],
    selected_norms: list[dict[str, Any]],
    clear_winner: bool,
    stage2_records: list[dict[str, Any]],
) -> None:
    ivw_records = filter_by_family(stage2_records, "ivw")
    htg_records = filter_by_family(stage2_records, "htg")
    fisher_main_records = filter_by_family(stage2_records, "fisher_main")
    fisher_isolation_records = filter_by_family(stage2_records, "fisher_isolation")

    best_ivw = best_record(ivw_records)
    best_htg = best_record(htg_records)
    best_fisher_main = best_record(fisher_main_records)
    best_fisher_isolation = best_record(fisher_isolation_records)

    frontier_ivw = s2_frontier_for_family(ivw_records)
    frontier_htg = s2_frontier_for_family(htg_records)
    frontier_fisher = s2_frontier_for_family(fisher_main_records)

    seven_of_seven: list[dict[str, Any]] = []
    for candidate_records in [ivw_records, htg_records, fisher_main_records]:
        for record in candidate_records:
            if int(record["evaluation"]["pass_count"]) == 7:
                seven_of_seven.append(record)

    lines: list[str] = []
    lines.append("# Aggregation Candidate Sweep Summary (Session 2)\n\n")
    lines.append(
        f"Generated: {datetime.now(timezone.utc).isoformat()}\n\n"
    )
    lines.append("## Stage 1 Normalization Sweep\n\n")
    lines.append(
        f"- Configs evaluated: {len(stage1_records)} normalization configs x 3 candidates = {len(stage1_records) * 3} candidate-configs\n"
    )
    ranked_stage1 = sorted(stage1_records, key=summarize_stage1_record, reverse=True)
    top_stage1 = ranked_stage1[:3]
    lines.append("- Top normalization configs:\n")
    for record in top_stage1:
        lines.append(
            f"  - {record['normalization_id']}: total_passes={record['total_passes']}/21, "
            f"avg_pass_score={record['avg_pass_score']:.4f}, params={record['normalization']}\n"
        )
    if clear_winner:
        lines.append(
            f"- Winner: {selected_norms[0]['normalization_id']} (used for Stage 2)\n\n"
        )
    else:
        selected_ids = ", ".join(record["normalization_id"] for record in selected_norms)
        lines.append(
            f"- No clear winner (tie on Stage 1 score tuple). Stage 2 executed for top-3: {selected_ids}\n\n"
        )

    lines.append("## Stage 2 Candidate Sweeps\n\n")
    lines.append(
        f"- Candidate-configs evaluated: {len(stage2_records)} total "
        "(including Fisher isolation runs)\n\n"
    )
    lines.extend(markdown_table_top5("IVW-CDF Top 5", ivw_records))
    lines.extend(markdown_table_top5("HTG-Max Top 5", htg_records))
    lines.extend(markdown_table_top5("Fisher-UP Top 5 (main sweep)", fisher_main_records))

    lines.append("## 7/7 Configurations\n\n")
    if seven_of_seven:
        lines.append("| Candidate | Norm | Config |\n")
        lines.append("| --- | --- | --- |\n")
        for record in seven_of_seven:
            lines.append(
                f"| {record['candidate']} | {record['normalization_id']} | "
                f"`{json.dumps(record['config'], sort_keys=True)}` |\n"
            )
    else:
        lines.append("- No 7/7 configuration found in Stage 2.\n")
    lines.append("\n")

    lines.append("## S2 Criterion Sensitivity Frontier\n\n")
    lines.append(
        "Configs considered: those with 6/7 passes and only `S2` failing under the default multiplier `1.5`.\n\n"
    )
    lines.append("| Candidate | Qualifying configs | Best ratio agg/max_single | Max multiplier passed (1.0-2.0 grid) |\n")
    lines.append("| --- | --- | --- | --- |\n")
    lines.append(
        f"| IVW-CDF | {frontier_ivw['qualifying_configs']} | "
        f"{'n/a' if frontier_ivw['best_ratio'] is None else f'{frontier_ivw['best_ratio']:.4f}'} | "
        f"{'n/a' if frontier_ivw['max_multiplier_passed'] is None else f'{frontier_ivw['max_multiplier_passed']:.1f}'} |\n"
    )
    lines.append(
        f"| HTG-Max | {frontier_htg['qualifying_configs']} | "
        f"{'n/a' if frontier_htg['best_ratio'] is None else f'{frontier_htg['best_ratio']:.4f}'} | "
        f"{'n/a' if frontier_htg['max_multiplier_passed'] is None else f'{frontier_htg['max_multiplier_passed']:.1f}'} |\n"
    )
    lines.append(
        f"| Fisher-UP | {frontier_fisher['qualifying_configs']} | "
        f"{'n/a' if frontier_fisher['best_ratio'] is None else f'{frontier_fisher['best_ratio']:.4f}'} | "
        f"{'n/a' if frontier_fisher['max_multiplier_passed'] is None else f'{frontier_fisher['max_multiplier_passed']:.1f}'} |\n"
    )
    lines.append("\n")

    lines.append("## Fisher SE-Reliability Isolation (se_dampen=False)\n\n")
    if best_fisher_main is not None and best_fisher_isolation is not None:
        lines.append("| Sweep | Passes | Avg pass score | Failed scenarios | Norm | Config |\n")
        lines.append("| --- | --- | --- | --- | --- | --- |\n")
        lines.append(
            "| Main (se_dampen=True) | "
            f"{best_fisher_main['evaluation']['pass_count']}/7 | "
            f"{best_fisher_main['evaluation']['avg_pass_score']:.4f} | "
            f"{format_failed_scenarios(best_fisher_main['evaluation']['failed_scenarios'])} | "
            f"{best_fisher_main['normalization_id']} | "
            f"`{json.dumps(best_fisher_main['config'], sort_keys=True)}` |\n"
        )
        lines.append(
            "| Isolation (se_dampen=False) | "
            f"{best_fisher_isolation['evaluation']['pass_count']}/7 | "
            f"{best_fisher_isolation['evaluation']['avg_pass_score']:.4f} | "
            f"{format_failed_scenarios(best_fisher_isolation['evaluation']['failed_scenarios'])} | "
            f"{best_fisher_isolation['normalization_id']} | "
            f"`{json.dumps(best_fisher_isolation['config'], sort_keys=True)}` |\n"
        )
    else:
        lines.append("- Fisher comparison unavailable (missing records).\n")
    lines.append("\n")

    lines.append("## Best Per Candidate\n\n")
    lines.append("| Candidate | Passes | Failed scenarios | Norm | Config |\n")
    lines.append("| --- | --- | --- | --- | --- |\n")
    for label, record in [
        ("IVW-CDF", best_ivw),
        ("HTG-Max", best_htg),
        ("Fisher-UP", best_fisher_main),
    ]:
        if record is None:
            lines.append(f"| {label} | n/a | n/a | n/a | n/a |\n")
            continue
        lines.append(
            f"| {label} | {record['evaluation']['pass_count']}/7 | "
            f"{format_failed_scenarios(record['evaluation']['failed_scenarios'])} | "
            f"{record['normalization_id']} | "
            f"`{json.dumps(record['config'], sort_keys=True)}` |\n"
        )
    lines.append("\n")

    path.write_text("".join(lines), encoding="utf-8")


def main() -> None:
    here = Path(__file__).resolve().parent
    fixtures = build_scenario_fixtures()

    stage1_records, selected_norms, clear_winner = stage1_sweep(fixtures)

    stage2_records: list[dict[str, Any]] = []
    for selected in selected_norms:
        stage2_records.extend(
            stage2_sweep_for_normalization(
                fixtures,
                normalization_id=str(selected["normalization_id"]),
                normalization_params=dict(selected["normalization"]),
            )
        )

    payload = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage1": {
            "clear_winner": clear_winner,
            "selected_normalizations": selected_norms,
            "records": stage1_records,
        },
        "stage2": {
            "records": stage2_records,
        },
    }

    results_path = here / "sweep_results.json"
    summary_path = here / "sweep_summary.md"
    results_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    write_summary(
        summary_path,
        stage1_records=stage1_records,
        selected_norms=selected_norms,
        clear_winner=clear_winner,
        stage2_records=stage2_records,
    )

    print(f"Stage 1 records: {len(stage1_records)}")
    print(f"Selected normalizations for Stage 2: {[record['normalization_id'] for record in selected_norms]}")
    print(f"Stage 2 records: {len(stage2_records)}")
    print(f"Wrote {results_path}")
    print(f"Wrote {summary_path}")


if __name__ == "__main__":
    main()
