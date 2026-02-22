from __future__ import annotations

import json
import math
from dataclasses import replace
from datetime import datetime, timezone
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
from models import MetricComponent, SigmoidParams, Summary
from normalization import NormalizationConfig, extract_uncertainty_snapshot
from scenarios import DEFAULT_CUSTOM_SIGMOIDS, build_scenario_fixtures


def build_normalization(params: dict[str, Any]) -> NormalizationConfig:
    return NormalizationConfig(
        absolute_difference_sigmoid=SigmoidParams(
            k=float(params["abs_diff_k"]),
            x0=float(params["abs_diff_x0"]),
        ),
        custom_sigmoids=dict(DEFAULT_CUSTOM_SIGMOIDS),
        clip_eps=1e-12,
        se_dampen_enabled=bool(params["se_dampen_enabled"]),
        se_dampen_k=float(params["se_dampen_k"]),
        se_dampen_x0=float(params["se_dampen_x0"]),
    )


def best_record(records: list[dict[str, Any]], family: str) -> dict[str, Any]:
    subset = [record for record in records if record["sweep_family"] == family]
    subset.sort(
        key=lambda record: (
            int(record["evaluation"]["pass_count"]),
            float(record["evaluation"]["avg_pass_score"]),
            float(record["evaluation"]["avg_all_score"]),
        ),
        reverse=True,
    )
    if not subset:
        raise ValueError(f"No records found for family={family}")
    return subset[0]


def build_candidate_runner(record: dict[str, Any]) -> tuple[str, Callable[[Sequence[MetricComponent]], float], dict[str, Any]]:
    family = str(record["sweep_family"])
    normalization = build_normalization(dict(record["normalization"]))
    config = dict(record["config"])
    label = str(record["candidate"])

    if family == "ivw":
        cfg = IVWCDFConfig(
            eps=float(config["eps"]),
            w_default=float(config["w_default"]),
            multiplicity_bonus_enabled=bool(config["multiplicity_bonus_enabled"]),
            multiplicity_threshold=float(config["multiplicity_threshold"]),
            multiplicity_scale=float(config["multiplicity_scale"]),
            normalization=normalization,
        )
        return label, lambda comps, local_cfg=cfg: aggregate_ivw_cdf(comps, local_cfg).aggregate_score, config

    if family == "htg":
        cfg = HTGMaxConfig(
            alpha=float(config["alpha"]),
            tau=float(config["tau"]),
            c_floor=float(config["c_floor"]),
            eps=float(config["eps"]),
            lse_beta=float(config["lse_beta"]),
            mode=str(config["mode"]),
            soft_sum_boost=float(config["soft_sum_boost"]),
            normalization=normalization,
        )
        return label, lambda comps, local_cfg=cfg: aggregate_htg_max(comps, local_cfg).aggregate_score, config

    if family == "fisher_main":
        cfg = FisherUPConfig(
            n_ref=float(config["n_ref"]),
            r_floor=float(config["r_floor"]),
            p_eps=float(config["p_eps"]),
            se_reliability_enabled=bool(config["se_reliability_enabled"]),
            se_reliability_k=float(config["se_reliability_k"]),
            se_reliability_x0=float(config["se_reliability_x0"]),
            normalization=normalization,
        )
        return label, lambda comps, local_cfg=cfg: aggregate_fisher_up(comps, local_cfg).aggregate_score, config

    raise ValueError(f"Unsupported sweep family for calibration: {family}")


def with_value_scale(component: MetricComponent, scale: float) -> MetricComponent:
    return replace(component, value=component.value * scale)


def with_uncertainty_scale(component: MetricComponent, scale: float) -> MetricComponent:
    if component.uncertainty is None:
        return component
    point = component.uncertainty.point
    if isinstance(point, Summary) and point.standard_error is not None:
        updated_summary = replace(point, standard_error=point.standard_error * scale)
        updated_uncertainty = replace(component.uncertainty, point=updated_summary)
        return replace(component, uncertainty=updated_uncertainty)
    return component


def rank_values(values: Sequence[float]) -> list[float]:
    indexed = sorted((value, idx) for idx, value in enumerate(values))
    ranks = [0.0 for _ in values]
    n = len(indexed)
    start = 0
    while start < n:
        end = start
        while end + 1 < n and indexed[end + 1][0] == indexed[start][0]:
            end += 1
        avg_rank = (start + end) / 2.0 + 1.0
        for pos in range(start, end + 1):
            ranks[indexed[pos][1]] = avg_rank
        start = end + 1
    return ranks


def pearson_r(x: Sequence[float], y: Sequence[float]) -> float:
    if len(x) != len(y) or not x:
        return 0.0
    x_mean = mean(x)
    y_mean = mean(y)
    x_dev = [value - x_mean for value in x]
    y_dev = [value - y_mean for value in y]
    numerator = sum(a * b for a, b in zip(x_dev, y_dev))
    denominator = math.sqrt(sum(a * a for a in x_dev) * sum(b * b for b in y_dev))
    if denominator <= 0.0:
        return 0.0
    return numerator / denominator


def spearman_rho(x: Sequence[float], y: Sequence[float]) -> float:
    return pearson_r(rank_values(x), rank_values(y))


def run_pattern_a(
    metrics: Sequence[MetricComponent],
    scorer: Callable[[Sequence[MetricComponent]], float],
) -> tuple[list[dict[str, float]], dict[str, Any]]:
    cycles: list[dict[str, float]] = []
    scores: list[float] = []
    times = [float(t) for t in range(50)]
    for t in range(50):
        scale = 1.0 - (float(t) / 50.0)
        transformed = [with_value_scale(component, scale) for component in metrics]
        score = scorer(transformed)
        scores.append(score)
        cycles.append({"t": float(t), "scale": scale, "score": score})
    rho = spearman_rho(times, scores)
    smoothness = max(
        abs(scores[idx + 1] - scores[idx]) for idx in range(len(scores) - 1)
    )
    return cycles, {
        "metric_name": "spearman_rho",
        "metric_value": rho,
        "threshold": -0.9,
        "metric_pass": rho < -0.9,
        "max_delta": smoothness,
        "smoothness_pass": smoothness < 0.3,
    }


def run_pattern_b(
    metrics: Sequence[MetricComponent],
    scorer: Callable[[Sequence[MetricComponent]], float],
) -> tuple[list[dict[str, float]], dict[str, Any]]:
    cycles: list[dict[str, float]] = []
    scores: list[float] = []
    for t in range(50):
        transformed: list[MetricComponent] = []
        for idx, component in enumerate(metrics):
            if t < 25:
                scale = 0.1
            else:
                scale = 5.0 if idx == 0 else 0.1
            transformed.append(with_value_scale(component, scale))
        score = scorer(transformed)
        scores.append(score)
        cycles.append({"t": float(t), "score": score})
    pre_mean = mean(scores[:25])
    post_mean = mean(scores[25:])
    step_ratio = post_mean / max(pre_mean, 1e-12)
    smoothness = max(
        abs(scores[idx + 1] - scores[idx]) for idx in range(len(scores) - 1)
    )
    return cycles, {
        "metric_name": "step_ratio",
        "metric_value": step_ratio,
        "threshold": 3.0,
        "metric_pass": step_ratio > 3.0,
        "max_delta": smoothness,
        "smoothness_pass": smoothness < 0.3,
    }


def run_pattern_c(
    metrics: Sequence[MetricComponent],
    scorer: Callable[[Sequence[MetricComponent]], float],
) -> tuple[list[dict[str, float]], dict[str, Any]]:
    cycles: list[dict[str, float]] = []
    scores: list[float] = []
    se_values: list[float] = []
    for t in range(50):
        se_scale = 0.55 + 0.45 * math.sin(2.0 * math.pi * float(t) / 50.0)
        transformed = [with_uncertainty_scale(component, se_scale) for component in metrics]
        score = scorer(transformed)
        score_ses = []
        for component in transformed:
            snapshot = extract_uncertainty_snapshot(component)
            if snapshot.standard_error is not None:
                score_ses.append(snapshot.standard_error)
        cycle_se = mean(score_ses) if score_ses else 0.0
        scores.append(score)
        se_values.append(cycle_se)
        cycles.append({"t": float(t), "se_scale": se_scale, "mean_se": cycle_se, "score": score})
    corr = pearson_r(se_values, scores)
    smoothness = max(
        abs(scores[idx + 1] - scores[idx]) for idx in range(len(scores) - 1)
    )
    return cycles, {
        "metric_name": "pearson_r",
        "metric_value": corr,
        "threshold": -0.5,
        "metric_pass": corr < -0.5,
        "max_delta": smoothness,
        "smoothness_pass": smoothness < 0.3,
    }


def main() -> None:
    here = Path(__file__).resolve().parent
    sweep_path = here / "sweep_results.json"
    if not sweep_path.exists():
        raise FileNotFoundError(f"Missing sweep results: {sweep_path}")

    sweep_payload = json.loads(sweep_path.read_text(encoding="utf-8"))
    stage2_records = list(sweep_payload["stage2"]["records"])
    selected_records = {
        "IVW-CDF": best_record(stage2_records, "ivw"),
        "HTG-Max": best_record(stage2_records, "htg"),
        "Fisher-UP": best_record(stage2_records, "fisher_main"),
    }

    fixtures = build_scenario_fixtures()
    calibration_fixture = next(fixture for fixture in fixtures if fixture.idx == 6)
    base_metrics = list(calibration_fixture.datasets["calibration"])

    pattern_fns = {
        "A": ("Gradual convergence", run_pattern_a),
        "B": ("Sudden regime change", run_pattern_b),
        "C": ("Oscillating uncertainty", run_pattern_c),
    }

    payload: dict[str, Any] = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "source_fixture": f"S{calibration_fixture.idx} {calibration_fixture.name}",
        "best_configs": {},
        "patterns": {},
    }

    summary_lines: list[str] = []
    summary_lines.append("# Calibration Simulation Summary (Session 2 Stretch)\n\n")
    summary_lines.append(
        f"Generated: {payload['generated_at_utc']}\n\n"
    )
    summary_lines.append(
        "Base fixture: S6 calibration decomposability (6 metrics), 50 deterministic cycles per pattern.\n\n"
    )
    summary_lines.append(
        "| Pattern | Candidate | Metric | Value | Threshold | Metric pass | Max delta | Smoothness pass | Overall |\n"
    )
    summary_lines.append("| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n")

    for candidate_name, record in selected_records.items():
        label, scorer, config = build_candidate_runner(record)
        payload["best_configs"][candidate_name] = {
            "normalization_id": record["normalization_id"],
            "normalization": record["normalization"],
            "config": config,
        }
        for pattern_id, (pattern_name, pattern_fn) in pattern_fns.items():
            cycles, metrics = pattern_fn(base_metrics, scorer)
            overall_pass = bool(metrics["metric_pass"] and metrics["smoothness_pass"])
            if pattern_id not in payload["patterns"]:
                payload["patterns"][pattern_id] = {
                    "name": pattern_name,
                    "results": [],
                }
            payload["patterns"][pattern_id]["results"].append(
                {
                    "candidate": label,
                    "normalization_id": record["normalization_id"],
                    "metrics": metrics,
                    "overall_pass": overall_pass,
                    "cycles": cycles,
                }
            )
            threshold_repr = (
                "< -0.9" if pattern_id == "A" else "> 3.0" if pattern_id == "B" else "< -0.5"
            )
            summary_lines.append(
                "| "
                f"{pattern_id} {pattern_name} | {label} | {metrics['metric_name']} | "
                f"{metrics['metric_value']:.4f} | {threshold_repr} | "
                f"{'PASS' if metrics['metric_pass'] else 'FAIL'} | {metrics['max_delta']:.4f} | "
                f"{'PASS' if metrics['smoothness_pass'] else 'FAIL'} | "
                f"{'PASS' if overall_pass else 'FAIL'} |\n"
            )

    results_path = here / "calibration_results.json"
    summary_path = here / "calibration_summary.md"
    results_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    summary_path.write_text("".join(summary_lines), encoding="utf-8")

    print(f"Wrote {results_path}")
    print(f"Wrote {summary_path}")


if __name__ == "__main__":
    main()
