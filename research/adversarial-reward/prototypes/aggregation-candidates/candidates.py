from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Callable, Sequence

from models import AggregateResult, ComponentContribution, DivergenceKind, MetricComponent
from normalization import (
    NormalizationConfig,
    bounded_unit_interval,
    extract_uncertainty_snapshot,
    gate_precision,
    inverse_variance_weight,
    normalize_component,
    sigmoid,
)


@dataclass(frozen=True)
class IVWCDFConfig:
    eps: float = 1e-12
    w_default: float = 1.0
    multiplicity_bonus_enabled: bool = False
    multiplicity_threshold: float = 0.1
    multiplicity_scale: float = 0.5
    normalization: NormalizationConfig = field(default_factory=NormalizationConfig)


@dataclass(frozen=True)
class HTGMaxConfig:
    alpha: float = 1.5
    tau: float = 7.8
    c_floor: float = 0.15
    eps: float = 1e-12
    lse_beta: float = 8.0
    mode: str = "hard_max"
    soft_sum_boost: float = 2.0
    normalization: NormalizationConfig = field(default_factory=NormalizationConfig)


@dataclass(frozen=True)
class FisherUPConfig:
    n_ref: float = 100.0
    r_floor: float = 0.1
    p_eps: float = 1e-12
    se_reliability_enabled: bool = False
    se_reliability_k: float = 3.0
    se_reliability_x0: float = 2.0
    normalization: NormalizationConfig = field(default_factory=NormalizationConfig)


def chi_square_cdf_even_df(x: float, n_terms: int) -> float:
    # For df = 2N, CDF(x;2N) = 1 - exp(-x/2) * sum_{k=0}^{N-1} (x/2)^k / k!
    if n_terms <= 0:
        return 0.0
    half_x = max(0.0, x) / 2.0
    term = 1.0
    series = term
    for k in range(1, n_terms):
        term *= half_x / k
        series += term
    cdf = 1.0 - math.exp(-half_x) * series
    return bounded_unit_interval(cdf, 1e-12)


def aggregate_ivw_cdf(
    components: Sequence[MetricComponent], config: IVWCDFConfig | None = None
) -> AggregateResult:
    cfg = config or IVWCDFConfig()
    warnings: list[str] = []
    skipped: list[str] = []
    staged: list[dict[str, object]] = []

    numerator = 0.0
    denominator = 0.0
    for idx, component in enumerate(components):
        score, local_warnings, score_diag = normalize_component(component, cfg.normalization)
        warnings.extend(local_warnings)
        if score is None:
            skipped.append(component.method_ref)
            continue

        weight, snapshot = inverse_variance_weight(component, cfg.eps, cfg.w_default)
        numerator += weight * score
        denominator += weight
        staged.append(
            {
                "idx": idx,
                "component": component,
                "score": score,
                "raw_weight": weight,
                "sample_size": snapshot.sample_size,
                "standard_error": snapshot.standard_error,
                "weight_source": snapshot.source,
                "raw_score": score_diag.get("raw_score"),
                "direction_mode": score_diag.get("direction_mode"),
            }
        )

    aggregate = numerator / denominator if denominator > 0.0 else 0.0
    if cfg.multiplicity_bonus_enabled and len(staged) > 1:
        concordant = sum(1 for entry in staged if float(entry["score"]) > cfg.multiplicity_threshold)
        concordance = concordant / len(staged)
        bonus = 1.0 + cfg.multiplicity_scale * concordance * math.log(len(staged))
        aggregate = aggregate * bonus
    aggregate = bounded_unit_interval(aggregate, cfg.normalization.clip_eps)
    contributions: list[ComponentContribution] = []
    for entry in staged:
        raw_weight = float(entry["raw_weight"])
        norm_weight = raw_weight / denominator if denominator > 0.0 else 0.0
        score = float(entry["score"])
        contribution = norm_weight * score
        component = entry["component"]
        contributions.append(
            ComponentContribution(
                index=int(entry["idx"]),
                method_ref=component.method_ref,
                kind=component.kind,
                score=score,
                weight=norm_weight,
                contribution=contribution,
                diagnostics={
                    "raw_weight": raw_weight,
                    "sample_size": entry["sample_size"],
                    "standard_error": entry["standard_error"],
                    "weight_source": entry["weight_source"],
                    "raw_score": entry["raw_score"],
                    "direction_mode": entry["direction_mode"],
                },
            )
        )

    return AggregateResult(
        candidate="IVW-CDF",
        aggregate_score=aggregate,
        contributions=contributions,
        skipped=skipped,
        warnings=warnings,
    )


def aggregate_htg_max(
    components: Sequence[MetricComponent], config: HTGMaxConfig | None = None
) -> AggregateResult:
    cfg = config or HTGMaxConfig()
    warnings: list[str] = []
    skipped: list[str] = []

    staged: list[dict[str, object]] = []
    for idx, component in enumerate(components):
        score, local_warnings, score_diag = normalize_component(component, cfg.normalization)
        warnings.extend(local_warnings)
        if score is None:
            skipped.append(component.method_ref)
            continue

        precision = gate_precision(component, cfg.eps)
        confidence = (
            cfg.c_floor
            if precision is None
            else sigmoid(precision, cfg.alpha, cfg.tau)
        )
        confidence = bounded_unit_interval(confidence, cfg.normalization.clip_eps)
        gated_score = score * confidence

        staged.append(
            {
                "idx": idx,
                "component": component,
                "score": score,
                "confidence": confidence,
                "gated_score": gated_score,
                "precision": precision,
                "score_diag": score_diag,
            }
        )

    if not staged:
        return AggregateResult(
            candidate="HTG-Max",
            aggregate_score=0.0,
            contributions=[],
            skipped=skipped,
            warnings=warnings,
        )

    group_winners: dict[DivergenceKind, dict[str, object]] = {}
    for entry in staged:
        kind = entry["component"].kind
        current = group_winners.get(kind)
        if current is None or float(entry["gated_score"]) > float(current["gated_score"]):
            group_winners[kind] = entry

    winners = list(group_winners.values())
    if cfg.mode == "lse_rebound":
        scaled = [math.exp(cfg.lse_beta * float(entry["gated_score"])) for entry in winners]
        lse = math.log(sum(scaled)) / cfg.lse_beta
        aggregate = 1.0 - math.exp(-lse)

        softmax_den = sum(scaled)
        winner_weight_map: dict[int, float] = {}
        for scaled_value, entry in zip(scaled, winners):
            comp_idx = int(entry["idx"])
            winner_weight_map[comp_idx] = scaled_value / softmax_den if softmax_den > 0.0 else 0.0
    elif cfg.mode == "soft_sum":
        raw_sum = sum(float(entry["gated_score"]) for entry in winners)
        aggregate = raw_sum / len(winners) * cfg.soft_sum_boost
        winner_weight_map = {int(entry["idx"]): 1.0 / len(winners) for entry in winners}
    else:
        # Primary variant for this session: hard max across type-level winners.
        overall_winner = max(winners, key=lambda entry: float(entry["gated_score"]))
        aggregate = float(overall_winner["gated_score"])
        winner_weight_map = {int(overall_winner["idx"]): 1.0}

    aggregate = bounded_unit_interval(aggregate, cfg.normalization.clip_eps)
    contributions: list[ComponentContribution] = []
    for entry in staged:
        idx = int(entry["idx"])
        score = float(entry["score"])
        confidence = float(entry["confidence"])
        winner_gate = winner_weight_map.get(idx, 0.0)
        # Decomposition guarantees sum_i(weight_i * score_i) equals aggregate.
        weight = winner_gate * confidence
        contribution = weight * score
        component = entry["component"]
        score_diag = entry["score_diag"]
        contributions.append(
            ComponentContribution(
                index=idx,
                method_ref=component.method_ref,
                kind=component.kind,
                score=score,
                weight=weight,
                contribution=contribution,
                diagnostics={
                    "confidence": confidence,
                    "gated_score": float(entry["gated_score"]),
                    "precision": entry["precision"],
                    "winner_gate": winner_gate,
                    "raw_score": score_diag.get("raw_score"),
                    "direction_mode": score_diag.get("direction_mode"),
                    "mode": cfg.mode,
                },
            )
        )

    if cfg.mode == "hard_max":
        candidate_name = "HTG-Max"
    elif cfg.mode == "lse_rebound":
        candidate_name = "HTG-Max-LSE"
    elif cfg.mode == "soft_sum":
        candidate_name = "HTG-Max-SoftSum"
    else:
        candidate_name = "HTG-Max"

    return AggregateResult(
        candidate=candidate_name,
        aggregate_score=aggregate,
        contributions=contributions,
        skipped=skipped,
        warnings=warnings,
    )


def aggregate_fisher_up(
    components: Sequence[MetricComponent], config: FisherUPConfig | None = None
) -> AggregateResult:
    cfg = config or FisherUPConfig()
    warnings: list[str] = []
    skipped: list[str] = []
    staged: list[dict[str, float | int | str | DivergenceKind | None]] = []

    for idx, component in enumerate(components):
        score, local_warnings, score_diag = normalize_component(component, cfg.normalization)
        warnings.extend(local_warnings)
        if score is None:
            skipped.append(component.method_ref)
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
            se_factor = sigmoid(snr, cfg.se_reliability_k, cfg.se_reliability_x0)
            reliability = reliability * se_factor

        p_adj = p_value ** reliability if reliability > 0.0 else 1.0
        p_adj = min(1.0, max(cfg.p_eps, p_adj))
        log_evidence = -2.0 * math.log(p_adj)

        staged.append(
            {
                "idx": idx,
                "method_ref": component.method_ref,
                "kind": component.kind,
                "score": score,
                "p_value": p_value,
                "p_adj": p_adj,
                "log_evidence": log_evidence,
                "reliability": reliability,
                "sample_size": snapshot.sample_size,
                "standard_error": snapshot.standard_error,
                "raw_score": score_diag.get("raw_score"),
                "direction_mode": score_diag.get("direction_mode"),
            }
        )

    if not staged:
        return AggregateResult(
            candidate="Fisher-UP",
            aggregate_score=0.0,
            contributions=[],
            skipped=skipped,
            warnings=warnings,
        )

    total_log_evidence = sum(float(entry["log_evidence"]) for entry in staged)
    n = len(staged)
    aggregate = chi_square_cdf_even_df(total_log_evidence, n_terms=n)
    aggregate = bounded_unit_interval(aggregate, cfg.normalization.clip_eps)

    denom = sum(float(entry["log_evidence"]) * float(entry["score"]) for entry in staged)
    if denom > 0.0:
        scale = aggregate / denom
        weights = [float(entry["log_evidence"]) * scale for entry in staged]
    else:
        weights = [0.0 for _ in staged]

    contributions: list[ComponentContribution] = []
    for weight, entry in zip(weights, staged):
        score = float(entry["score"])
        contribution = weight * score
        contributions.append(
            ComponentContribution(
                index=int(entry["idx"]),
                method_ref=str(entry["method_ref"]),
                kind=entry["kind"],  # type: ignore[arg-type]
                score=score,
                weight=weight,
                contribution=contribution,
                diagnostics={
                    "p_value": float(entry["p_value"]),
                    "p_adj": float(entry["p_adj"]),
                    "log_evidence": float(entry["log_evidence"]),
                    "reliability": float(entry["reliability"]),
                    "sample_size": entry["sample_size"],
                    "standard_error": entry["standard_error"],
                    "raw_score": entry["raw_score"],
                    "direction_mode": entry["direction_mode"],
                },
            )
        )

    return AggregateResult(
        candidate="Fisher-UP",
        aggregate_score=aggregate,
        contributions=contributions,
        skipped=skipped,
        warnings=warnings,
    )


CandidateFn = Callable[[Sequence[MetricComponent]], AggregateResult]


def get_candidate_registry(
    ivw_cfg: IVWCDFConfig | None = None,
    htg_cfg: HTGMaxConfig | None = None,
    fisher_cfg: FisherUPConfig | None = None,
) -> dict[str, CandidateFn]:
    return {
        "IVW-CDF": lambda components: aggregate_ivw_cdf(components, ivw_cfg),
        "HTG-Max": lambda components: aggregate_htg_max(components, htg_cfg),
        "Fisher-UP": lambda components: aggregate_fisher_up(components, fisher_cfg),
    }
