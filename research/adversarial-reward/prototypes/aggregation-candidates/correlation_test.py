from __future__ import annotations

import json
import math
import random
from dataclasses import replace
from datetime import datetime, timezone
from pathlib import Path
from statistics import mean
from typing import Any, Sequence

from candidates import FisherUPConfig, aggregate_fisher_up, chi_square_cdf_even_df
from models import MetricComponent, SigmoidParams
from normalization import (
    NormalizationConfig,
    extract_uncertainty_snapshot,
    normalize_component,
    sigmoid,
)
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


def load_best_fisher_config(sweep_payload: dict[str, Any]) -> FisherUPConfig:
    records = [record for record in sweep_payload["stage2"]["records"] if record["sweep_family"] == "fisher_main"]
    records.sort(
        key=lambda record: (
            int(record["evaluation"]["pass_count"]),
            float(record["evaluation"]["avg_pass_score"]),
            float(record["evaluation"]["avg_all_score"]),
        ),
        reverse=True,
    )
    if not records:
        raise ValueError("No fisher_main sweep records available")
    best = records[0]
    config = dict(best["config"])
    normalization = build_normalization(dict(best["normalization"]))
    return FisherUPConfig(
        n_ref=float(config["n_ref"]),
        r_floor=float(config["r_floor"]),
        p_eps=float(config["p_eps"]),
        se_reliability_enabled=bool(config["se_reliability_enabled"]),
        se_reliability_k=float(config["se_reliability_k"]),
        se_reliability_x0=float(config["se_reliability_x0"]),
        normalization=normalization,
    )


def correlation_matrix(k: int, rho: float) -> list[list[float]]:
    matrix: list[list[float]] = []
    for i in range(k):
        row: list[float] = []
        for j in range(k):
            row.append(1.0 if i == j else rho)
        matrix.append(row)
    return matrix


def cholesky_decompose(matrix: Sequence[Sequence[float]]) -> list[list[float]]:
    n = len(matrix)
    lower = [[0.0 for _ in range(n)] for _ in range(n)]
    for i in range(n):
        for j in range(i + 1):
            summation = sum(lower[i][k] * lower[j][k] for k in range(j))
            if i == j:
                value = matrix[i][i] - summation
                lower[i][j] = math.sqrt(max(value, 0.0))
            else:
                denom = lower[j][j]
                lower[i][j] = 0.0 if denom == 0.0 else (matrix[i][j] - summation) / denom
    return lower


def sample_correlated(lower: Sequence[Sequence[float]]) -> list[float]:
    n = len(lower)
    independent = [random.gauss(0.0, 1.0) for _ in range(n)]
    correlated = [0.0 for _ in range(n)]
    for i in range(n):
        correlated[i] = sum(lower[i][j] * independent[j] for j in range(i + 1))
    return correlated


def perturb_component(component: MetricComponent, noise: float) -> MetricComponent:
    # Keep weak signals in the same regime while injecting correlation structure.
    scale = max(0.1, 1.0 + 0.15 * noise)
    return replace(component, value=component.value * scale)


def fisher_log_evidence_sum(
    components: Sequence[MetricComponent], cfg: FisherUPConfig
) -> float:
    total = 0.0
    for component in components:
        score, _, _ = normalize_component(component, cfg.normalization)
        if score is None:
            continue
        snapshot = extract_uncertainty_snapshot(component)
        p_value = min(1.0, max(cfg.p_eps, 1.0 - score))
        if snapshot.uncertainty_present:
            n_i = float(snapshot.sample_size or 0)
            reliability = min(1.0, n_i / cfg.n_ref)
        else:
            reliability = cfg.r_floor
        if (
            cfg.se_reliability_enabled
            and snapshot.standard_error is not None
            and snapshot.standard_error > 0
        ):
            snr = abs(component.value) / snapshot.standard_error
            reliability *= sigmoid(snr, cfg.se_reliability_k, cfg.se_reliability_x0)
        p_adj = p_value ** reliability if reliability > 0.0 else 1.0
        p_adj = min(1.0, max(cfg.p_eps, p_adj))
        total += -2.0 * math.log(p_adj)
    return total


def variance(values: Sequence[float]) -> float:
    if len(values) < 2:
        return 0.0
    avg = mean(values)
    return sum((value - avg) ** 2 for value in values) / (len(values) - 1)


def main() -> None:
    here = Path(__file__).resolve().parent
    sweep_path = here / "sweep_results.json"
    if not sweep_path.exists():
        raise FileNotFoundError(f"Missing sweep results: {sweep_path}")

    payload = json.loads(sweep_path.read_text(encoding="utf-8"))
    fisher_cfg = load_best_fisher_config(payload)

    fixtures = build_scenario_fixtures()
    s2_fixture = next(fixture for fixture in fixtures if fixture.idx == 2)
    base_components = list(s2_fixture.datasets["unanimous"])
    k = len(base_components)

    random.seed(42)
    rho_values = [0.0, 0.3, 0.5, 0.7, 0.9]
    samples_per_rho = 400

    results: list[dict[str, Any]] = []
    for rho in rho_values:
        corr = correlation_matrix(k, rho)
        chol = cholesky_decompose(corr)
        uncorrected_scores: list[float] = []
        corrected_scores: list[float] = []
        t_values: list[float] = []

        sampled_components: list[list[MetricComponent]] = []
        for _ in range(samples_per_rho):
            noise = sample_correlated(chol)
            components = [perturb_component(component, draw) for component, draw in zip(base_components, noise)]
            sampled_components.append(components)
            t_values.append(fisher_log_evidence_sum(components, fisher_cfg))
            uncorrected_scores.append(aggregate_fisher_up(components, fisher_cfg).aggregate_score)

        var_t = variance(t_values)
        effective_df = (2.0 * (k**2) / var_t) if var_t > 0.0 else float(2 * k)
        corrected_terms = max(1, int(effective_df / 2.0))
        corrected_terms = min(corrected_terms, 1000)
        for t_value in t_values:
            corrected_scores.append(chi_square_cdf_even_df(t_value, n_terms=corrected_terms))

        mean_uncorrected = mean(uncorrected_scores) if uncorrected_scores else 0.0
        mean_corrected = mean(corrected_scores) if corrected_scores else 0.0
        inflation_ratio = mean_uncorrected / max(mean_corrected, 1e-12)
        flagged = bool(rho == 0.5 and inflation_ratio > 1.5)

        results.append(
            {
                "rho": rho,
                "samples": samples_per_rho,
                "var_t": var_t,
                "effective_df": effective_df,
                "corrected_n_terms": corrected_terms,
                "mean_uncorrected_aggregate": mean_uncorrected,
                "mean_corrected_aggregate": mean_corrected,
                "inflation_ratio": inflation_ratio,
                "flag_ratio_gt_1_5_at_rho_0_5": flagged,
            }
        )

    output = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "source_fixture": f"S{s2_fixture.idx} {s2_fixture.name}",
        "fisher_config": {
            "n_ref": fisher_cfg.n_ref,
            "r_floor": fisher_cfg.r_floor,
            "p_eps": fisher_cfg.p_eps,
            "se_reliability_enabled": fisher_cfg.se_reliability_enabled,
            "se_reliability_k": fisher_cfg.se_reliability_k,
            "se_reliability_x0": fisher_cfg.se_reliability_x0,
            "normalization": {
                "abs_diff_k": fisher_cfg.normalization.absolute_difference_sigmoid.k,
                "abs_diff_x0": fisher_cfg.normalization.absolute_difference_sigmoid.x0,
                "se_dampen_enabled": fisher_cfg.normalization.se_dampen_enabled,
                "se_dampen_k": fisher_cfg.normalization.se_dampen_k,
                "se_dampen_x0": fisher_cfg.normalization.se_dampen_x0,
            },
        },
        "results": results,
    }

    out_path = here / "correlation_results.json"
    out_path.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {out_path}")
    for row in results:
        print(
            "rho={rho:.1f} inflation={inflation:.4f} corrected={corr:.4f} uncorrected={uncorr:.4f}".format(
                rho=row["rho"],
                inflation=row["inflation_ratio"],
                corr=row["mean_corrected_aggregate"],
                uncorr=row["mean_uncorrected_aggregate"],
            )
        )


if __name__ == "__main__":
    main()
