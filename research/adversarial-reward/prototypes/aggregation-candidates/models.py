from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional, TypeAlias, Union


class DivergenceKind(str, Enum):
    AbsoluteDifference = "AbsoluteDifference"
    ZScore = "ZScore"
    BayesFactor = "BayesFactor"
    KLDivergence = "KLDivergence"
    EffectSize = "EffectSize"
    Custom = "Custom"


# Functional Enum form allows the exact "None" variant name from the contract.
EffectDirection = Enum(
    "EffectDirection",
    {
        "Contradiction": "Contradiction",
        "Agreement": "Agreement",
        "None": "None",
    },
)


@dataclass(frozen=True)
class IntervalEstimate:
    lower: Optional[float] = None
    upper: Optional[float] = None
    confidence_level: Optional[float] = None
    method_ref: Optional[str] = None


DistributionPayload: TypeAlias = dict[str, Any]


@dataclass(frozen=True)
class Summary:
    sample_size: Optional[int]
    standard_error: Optional[float]
    interval: Optional[IntervalEstimate]
    method_ref: str


@dataclass(frozen=True)
class NoUncertainty:
    reason: str


PointUncertainty: TypeAlias = Union[Summary, NoUncertainty]


@dataclass(frozen=True)
class UncertaintySummary:
    point: PointUncertainty
    distribution: Optional[DistributionPayload] = None


@dataclass(frozen=True)
class MetricComponent:
    kind: DivergenceKind
    value: float
    direction: Optional[EffectDirection]
    uncertainty: Optional[UncertaintySummary]
    sample_size: Optional[int]
    units: Optional[str]
    method_ref: str


@dataclass(frozen=True)
class SigmoidParams:
    k: float
    x0: float


@dataclass(frozen=True)
class ComponentContribution:
    index: int
    method_ref: str
    kind: DivergenceKind
    score: float
    weight: float
    contribution: float
    diagnostics: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class AggregateResult:
    candidate: str
    aggregate_score: float
    contributions: list[ComponentContribution]
    skipped: list[str]
    warnings: list[str]

