from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Callable, Optional

from models import (
    DivergenceKind,
    MetricComponent,
    NoUncertainty,
    SigmoidParams,
    Summary,
)


BF_NORM_LOG_SCALED_C = 0.083647


def bf_norm_log_scaled(bf: float, c: float = BF_NORM_LOG_SCALED_C) -> float:
    log_term = math.log1p(bf)
    return log_term / (log_term + c)


@dataclass(frozen=True)
class NormalizationConfig:
    absolute_difference_sigmoid: SigmoidParams = field(
        default_factory=lambda: SigmoidParams(k=1200.0, x0=7e-4)
    )
    custom_sigmoids: dict[str, SigmoidParams] = field(default_factory=dict)
    bf_norm_fn: Callable[[float], float] = field(default_factory=lambda: bf_norm_log_scaled)
    clip_eps: float = 1e-12
    se_dampen_enabled: bool = False
    se_dampen_k: float = 5.0
    se_dampen_x0: float = 2.0

    def __post_init__(self) -> None:
        for key, params in self.custom_sigmoids.items():
            if params.x0 < 0:
                raise ValueError(
                    "GR-S2-CUSTOM-SIGMOID-X0-NONNEG: "
                    f"custom_sigmoids['{key}'] has x0={params.x0}; expected x0 >= 0"
                )


@dataclass(frozen=True)
class UncertaintySnapshot:
    sample_size: Optional[int]
    standard_error: Optional[float]
    uncertainty_present: bool
    source: str


def sigmoid(x: float, k: float, x0: float) -> float:
    return 1.0 / (1.0 + math.exp(-k * (x - x0)))


def se_dampen(raw_score: float, value: float, se: float, config: NormalizationConfig) -> float:
    snr = abs(value) / se
    return raw_score * sigmoid(snr, config.se_dampen_k, config.se_dampen_x0)


def normal_cdf(z: float) -> float:
    return 0.5 * (1.0 + math.erf(z / math.sqrt(2.0)))


def bounded_unit_interval(x: float, eps: float = 1e-12) -> float:
    return float(min(1.0 - eps, max(0.0 + eps, x)))


def _direction_value(direction: object) -> Optional[str]:
    if direction is None:
        return None
    if hasattr(direction, "value"):
        return str(getattr(direction, "value"))
    return str(direction)


def direction_is_agreement(direction: object) -> bool:
    return _direction_value(direction) == "Agreement"


def direction_is_none_variant(direction: object) -> bool:
    return _direction_value(direction) == "None"


def extract_uncertainty_snapshot(component: MetricComponent) -> UncertaintySnapshot:
    summary = None
    if component.uncertainty is not None and isinstance(component.uncertainty.point, Summary):
        summary = component.uncertainty.point

    sample_size = component.sample_size
    source = "component.sample_size"
    if sample_size is None and summary is not None:
        sample_size = summary.sample_size
        source = "uncertainty.point.sample_size"

    standard_error = summary.standard_error if summary is not None else None
    uncertainty_present = summary is not None

    if component.uncertainty is not None and isinstance(component.uncertainty.point, NoUncertainty):
        uncertainty_present = False

    return UncertaintySnapshot(
        sample_size=sample_size,
        standard_error=standard_error,
        uncertainty_present=uncertainty_present,
        source=source,
    )


def inverse_variance_weight(
    component: MetricComponent, eps: float, w_default: float
) -> tuple[float, UncertaintySnapshot]:
    snapshot = extract_uncertainty_snapshot(component)
    if (
        not snapshot.uncertainty_present
        or snapshot.sample_size is None
        or snapshot.standard_error is None
    ):
        return w_default, snapshot
    weight = float(snapshot.sample_size) / (snapshot.standard_error**2 + eps)
    return weight, snapshot


def gate_precision(component: MetricComponent, eps: float) -> Optional[float]:
    snapshot = extract_uncertainty_snapshot(component)
    if (
        not snapshot.uncertainty_present
        or snapshot.sample_size is None
        or snapshot.standard_error is None
    ):
        return None
    base_precision = float(snapshot.sample_size) / (snapshot.standard_error**2 + eps)
    return math.log1p(base_precision)


def normalize_component(
    component: MetricComponent, config: NormalizationConfig
) -> tuple[Optional[float], list[str], dict[str, float | str | None]]:
    warnings: list[str] = []

    direction_value = _direction_value(component.direction)
    if component.direction is None or direction_is_none_variant(component.direction):
        transformed_value = abs(component.value)
        direction_mode = "unsigned"
    else:
        transformed_value = component.value
        direction_mode = direction_value or "unset"

    kind = component.kind
    if kind is DivergenceKind.ZScore:
        raw_score = 2.0 * normal_cdf(abs(transformed_value)) - 1.0
    elif kind is DivergenceKind.BayesFactor:
        bf = max(transformed_value, 0.0)
        raw_score = config.bf_norm_fn(bf)
    elif kind is DivergenceKind.KLDivergence:
        kl = max(transformed_value, 0.0)
        raw_score = 1.0 - math.exp(-kl)
    elif kind is DivergenceKind.AbsoluteDifference:
        absdiff = abs(transformed_value)
        params = config.absolute_difference_sigmoid
        raw_score = sigmoid(absdiff, params.k, params.x0)
    elif kind is DivergenceKind.EffectSize:
        raw_score = 2.0 * normal_cdf(abs(transformed_value)) - 1.0
    elif kind is DivergenceKind.Custom:
        params = config.custom_sigmoids.get(component.method_ref)
        if params is None:
            warnings.append(
                f"Excluded Custom metric '{component.method_ref}': missing required sigmoid params (k, x0)"
            )
            return None, warnings, {
                "raw_score": None,
                "direction_mode": direction_mode,
                "transformed_value": transformed_value,
            }
        raw_score = sigmoid(transformed_value, params.k, params.x0)
    else:
        warnings.append(f"Unsupported metric kind '{component.kind}'")
        return None, warnings, {
            "raw_score": None,
            "direction_mode": direction_mode,
            "transformed_value": transformed_value,
        }

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
