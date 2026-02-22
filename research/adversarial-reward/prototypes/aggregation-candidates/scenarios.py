from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping, Sequence

from models import (
    DivergenceKind,
    EffectDirection,
    MetricComponent,
    NoUncertainty,
    SigmoidParams,
    Summary,
    UncertaintySummary,
)


@dataclass(frozen=True)
class ScenarioFixture:
    idx: int
    name: str
    what_it_tests: str
    pass_criterion: str
    datasets: Mapping[str, Sequence[MetricComponent]]


def _summary(
    sample_size: int | None,
    standard_error: float | None,
    method_ref: str,
) -> UncertaintySummary:
    return UncertaintySummary(
        point=Summary(
            sample_size=sample_size,
            standard_error=standard_error,
            interval=None,
            method_ref=method_ref,
        )
    )


def _no_uncertainty(reason: str) -> UncertaintySummary:
    return UncertaintySummary(point=NoUncertainty(reason=reason))


def _metric(
    kind: DivergenceKind,
    value: float,
    direction: str | None,
    sample_size: int | None,
    standard_error: float | None,
    method_ref: str,
    units: str | None = None,
    component_sample_size: int | None = None,
) -> MetricComponent:
    if standard_error is None:
        uncertainty = None
    else:
        uncertainty = _summary(sample_size, standard_error, f"{method_ref}.unc")

    enum_direction = None if direction is None else EffectDirection[direction]
    return MetricComponent(
        kind=kind,
        value=value,
        direction=enum_direction,
        uncertainty=uncertainty,
        sample_size=component_sample_size,
        units=units,
        method_ref=method_ref,
    )


DEFAULT_CUSTOM_SIGMOIDS: dict[str, SigmoidParams] = {
    "s2.custom.1": SigmoidParams(k=2.2, x0=0.0),
    "s6.custom.1": SigmoidParams(k=1.8, x0=0.3),
}


def build_scenario_fixtures() -> list[ScenarioFixture]:
    fixtures: list[ScenarioFixture] = []

    # 1) Noisy TV: value and uncertainty both increase.
    fixtures.append(
        ScenarioFixture(
            idx=1,
            name="Noisy TV",
            what_it_tests="One metric with high divergence and high uncertainty; inflation should be suppressed.",
            pass_criterion="score(value*2, se*2) <= score(value, se)",
            datasets={
                "base": [
                    _metric(
                        kind=DivergenceKind.AbsoluteDifference,
                        value=0.0012,
                        direction="Contradiction",
                        sample_size=80,
                        standard_error=0.30,
                        method_ref="s1.absdiff.base",
                        units="eV",
                    )
                ],
                "doubled": [
                    _metric(
                        kind=DivergenceKind.AbsoluteDifference,
                        value=0.0024,
                        direction="Contradiction",
                        sample_size=80,
                        standard_error=0.60,
                        method_ref="s1.absdiff.doubled",
                        units="eV",
                    )
                ],
            },
        )
    )

    # 2) Unanimous weak signal: many small contradiction metrics should compound.
    fixtures.append(
        ScenarioFixture(
            idx=2,
            name="Unanimous weak signal",
            what_it_tests="Eight weak but consistent contradiction metrics.",
            pass_criterion="aggregate >= 1.5 * max(single_metric_scores)",
            datasets={
                "unanimous": [
                    _metric(DivergenceKind.ZScore, 0.30, "Contradiction", 120, 0.25, "s2.z.1"),
                    _metric(DivergenceKind.EffectSize, 0.25, "Contradiction", 140, 0.23, "s2.d.1"),
                    _metric(DivergenceKind.KLDivergence, 0.10, "Contradiction", 150, 0.21, "s2.kl.1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0003,
                        "Contradiction",
                        160,
                        0.20,
                        "s2.abs.1",
                        units="eV",
                    ),
                    _metric(DivergenceKind.ZScore, 0.34, "Contradiction", 110, 0.26, "s2.z.2"),
                    _metric(DivergenceKind.EffectSize, 0.28, "Contradiction", 130, 0.24, "s2.d.2"),
                    _metric(DivergenceKind.KLDivergence, 0.12, "Contradiction", 115, 0.25, "s2.kl.2"),
                    _metric(DivergenceKind.Custom, 0.15, "Contradiction", 125, 0.22, "s2.custom.1"),
                ],
            },
        )
    )

    # 3) Mixed signal: contradiction and agreement should counterbalance.
    fixtures.append(
        ScenarioFixture(
            idx=3,
            name="Mixed signal",
            what_it_tests="Three contradiction + three agreement metrics.",
            pass_criterion="all_agreement <= mixed <= all_contradiction",
            datasets={
                "mixed": [
                    _metric(DivergenceKind.ZScore, 1.5, "Contradiction", 100, 0.22, "s3.z.c1"),
                    _metric(DivergenceKind.KLDivergence, 0.8, "Contradiction", 95, 0.24, "s3.kl.c1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0011,
                        "Contradiction",
                        110,
                        0.26,
                        "s3.abs.c1",
                        units="eV",
                    ),
                    _metric(DivergenceKind.ZScore, 1.4, "Agreement", 100, 0.22, "s3.z.a1"),
                    _metric(DivergenceKind.KLDivergence, 0.7, "Agreement", 95, 0.24, "s3.kl.a1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0010,
                        "Agreement",
                        110,
                        0.26,
                        "s3.abs.a1",
                        units="eV",
                    ),
                ],
                "all_contradiction": [
                    _metric(DivergenceKind.ZScore, 1.5, "Contradiction", 100, 0.22, "s3.z.c1"),
                    _metric(DivergenceKind.KLDivergence, 0.8, "Contradiction", 95, 0.24, "s3.kl.c1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0011,
                        "Contradiction",
                        110,
                        0.26,
                        "s3.abs.c1",
                        units="eV",
                    ),
                    _metric(DivergenceKind.ZScore, 1.4, "Contradiction", 100, 0.22, "s3.z.a1"),
                    _metric(DivergenceKind.KLDivergence, 0.7, "Contradiction", 95, 0.24, "s3.kl.a1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0010,
                        "Contradiction",
                        110,
                        0.26,
                        "s3.abs.a1",
                        units="eV",
                    ),
                ],
                "all_agreement": [
                    _metric(DivergenceKind.ZScore, 1.5, "Agreement", 100, 0.22, "s3.z.c1"),
                    _metric(DivergenceKind.KLDivergence, 0.8, "Agreement", 95, 0.24, "s3.kl.c1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0011,
                        "Agreement",
                        110,
                        0.26,
                        "s3.abs.c1",
                        units="eV",
                    ),
                    _metric(DivergenceKind.ZScore, 1.4, "Agreement", 100, 0.22, "s3.z.a1"),
                    _metric(DivergenceKind.KLDivergence, 0.7, "Agreement", 95, 0.24, "s3.kl.a1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0010,
                        "Agreement",
                        110,
                        0.26,
                        "s3.abs.a1",
                        units="eV",
                    ),
                ],
            },
        )
    )

    # 4) Missing data: absent/partial uncertainty should not crash or explode.
    full_baseline = [
        _metric(DivergenceKind.ZScore, 1.1, "Contradiction", 90, 0.20, "s4.z.1"),
        _metric(DivergenceKind.EffectSize, 0.8, "Contradiction", 85, 0.21, "s4.d.1"),
        _metric(DivergenceKind.KLDivergence, 0.4, "Contradiction", 92, 0.18, "s4.kl.1"),
        _metric(DivergenceKind.AbsoluteDifference, 0.0008, "Contradiction", 80, 0.24, "s4.abs.1"),
    ]
    missing_uncertainty = [
        MetricComponent(
            kind=DivergenceKind.ZScore,
            value=1.1,
            direction=EffectDirection["Contradiction"],
            uncertainty=_no_uncertainty("simulator omitted SE"),
            sample_size=90,
            units=None,
            method_ref="s4.z.1",
        ),
        MetricComponent(
            kind=DivergenceKind.EffectSize,
            value=0.8,
            direction=EffectDirection["Contradiction"],
            uncertainty=_summary(sample_size=85, standard_error=None, method_ref="s4.d.1.unc"),
            sample_size=None,
            units=None,
            method_ref="s4.d.1",
        ),
        MetricComponent(
            kind=DivergenceKind.KLDivergence,
            value=0.4,
            direction=EffectDirection["Contradiction"],
            uncertainty=None,
            sample_size=92,
            units=None,
            method_ref="s4.kl.1",
        ),
        MetricComponent(
            kind=DivergenceKind.AbsoluteDifference,
            value=0.0008,
            direction=EffectDirection["Contradiction"],
            uncertainty=_summary(sample_size=80, standard_error=0.24, method_ref="s4.abs.1.unc"),
            sample_size=None,
            units="eV",
            method_ref="s4.abs.1",
        ),
    ]
    fixtures.append(
        ScenarioFixture(
            idx=4,
            name="Missing data",
            what_it_tests="Partial uncertainty payloads and NoUncertainty variants.",
            pass_criterion="finite score and within 20% of full-uncertainty baseline",
            datasets={
                "missing": missing_uncertainty,
                "baseline_full": full_baseline,
            },
        )
    )

    # 5) Scale heterogeneity: disparate metric scales should normalize cleanly.
    fixtures.append(
        ScenarioFixture(
            idx=5,
            name="Scale heterogeneity",
            what_it_tests="Z=2.0, BF=100, AbsDiff=0.001eV normalization behavior.",
            pass_criterion="all normalized scores in [0.3, 0.99] (with tolerance) and stable ranking",
            datasets={
                "heterogeneous": [
                    _metric(DivergenceKind.ZScore, 2.0, "Contradiction", 120, 0.20, "s5.z.1"),
                    _metric(DivergenceKind.BayesFactor, 100.0, "Contradiction", 120, 0.20, "s5.bf.1"),
                    _metric(
                        DivergenceKind.AbsoluteDifference,
                        0.0010,
                        "Contradiction",
                        120,
                        0.20,
                        "s5.abs.1",
                        units="eV",
                    ),
                ]
            },
        )
    )

    # 6) Calibration decomposability: verify sum(w_i * u_i) reconstruction.
    fixtures.append(
        ScenarioFixture(
            idx=6,
            name="Calibration decomposability",
            what_it_tests="Per-component decomposition should reconstruct aggregate and expose dominant term.",
            pass_criterion="sum(w_i*u_i) ~= aggregate and one component clearly dominates",
            datasets={
                "calibration": [
                    _metric(DivergenceKind.ZScore, 3.0, "Contradiction", 220, 0.10, "s6.z.strong"),
                    _metric(DivergenceKind.EffectSize, 0.9, "Contradiction", 140, 0.18, "s6.d.mid"),
                    _metric(DivergenceKind.KLDivergence, 0.3, "Contradiction", 130, 0.24, "s6.kl.weak"),
                    _metric(DivergenceKind.AbsoluteDifference, 0.0009, "Contradiction", 110, 0.26, "s6.abs.mid"),
                    _metric(DivergenceKind.BayesFactor, 12.0, "Contradiction", 160, 0.16, "s6.bf.strong"),
                    _metric(DivergenceKind.Custom, 0.85, "Contradiction", 125, 0.25, "s6.custom.1"),
                ]
            },
        )
    )

    # 7) Boundary-seeking: inflated uncertainty at boundary should lower evidence.
    fixtures.append(
        ScenarioFixture(
            idx=7,
            name="Boundary-seeking",
            what_it_tests="High contradiction near parameter bounds with inflated uncertainty.",
            pass_criterion="boundary_case < equivalent_non_boundary_case",
            datasets={
                "boundary": [
                    _metric(DivergenceKind.ZScore, 3.2, "Contradiction", 100, 1.20, "s7.z.boundary"),
                    _metric(DivergenceKind.KLDivergence, 0.9, "Contradiction", 100, 0.20, "s7.kl.boundary"),
                    _metric(DivergenceKind.EffectSize, 1.0, "Contradiction", 100, 0.25, "s7.d.boundary"),
                ],
                "non_boundary": [
                    _metric(DivergenceKind.ZScore, 3.2, "Contradiction", 100, 0.20, "s7.z.boundary"),
                    _metric(DivergenceKind.KLDivergence, 0.9, "Contradiction", 100, 0.20, "s7.kl.boundary"),
                    _metric(DivergenceKind.EffectSize, 1.0, "Contradiction", 100, 0.25, "s7.d.boundary"),
                ],
            },
        )
    )

    return fixtures

