# Guardrail Specification (Session 4.3)

## Guardrail ID

`GR-S2-CUSTOM-SIGMOID-X0-NONNEG`

## Constraint Statement

For all entries in `NormalizationConfig.custom_sigmoids`, require:

`x0 >= 0`

This is a hard validation constraint. Negative `x0` values are invalid configuration.

## Rationale

Session 4 perturbation evidence shows S2 fragility is localized to negative midpoint settings:

- Failure locus: `x0=-0.2` with `k>=2.0`
- In-grid non-failure region: all sampled `x0>=0.0` combinations passed
- Evidence source: `perturbation_summary.md` (S2 custom-sigmoid margin grid)

Interpretation from normalization semantics:

- In `normalization.py`, custom metrics are transformed by `sigmoid(value, k, x0)` for `DivergenceKind.Custom`.
- For contradiction-oriented nonnegative custom inputs, `x0<0` shifts midpoint left of the zero-evidence region and over-amplifies weak custom signal.
- This creates avoidable entry into the observed S2 failure region.

`x0 >= 0` provides a clean margin and excludes the empirically identified failure locus.

## Scope

Guardrail applies to custom sigmoid parameters only:

- `normalization.py`: `normalize_component(...)` branch for `DivergenceKind.Custom`, parameter source `config.custom_sigmoids[method_ref]`.
- `scenarios.py`: current defaults already satisfy the guardrail:
  - `s2.custom.1`: `x0=0.0`
  - `s6.custom.1`: `x0=0.3`

Out of scope:

- `absolute_difference_sigmoid` (`NormalizationConfig.absolute_difference_sigmoid`) is a separate normalization parameter family and is not constrained by this guardrail.

## Enforcement Mechanism (Architecture-Level)

Enforce at configuration construction/validation time (before any aggregation run):

1. Iterate all `custom_sigmoids` entries.
2. If any `x0 < 0`, reject configuration.
3. Raise explicit validation error that includes:
   - guardrail ID
   - offending `method_ref`
   - offending `x0` value
   - expected constraint (`x0 >= 0`)

Required behavior:

- Reject with error.
- No silent clamping.
- No auto-correction.

## Violation Handling

If violated, pipeline state is `invalid_configuration` and aggregation execution must not start. The operator/workflow must provide corrected parameters and rerun validation.

## Relationship to Other Session 4/5 Findings

- Pattern B under-response (`step_ratio=1.029` under 50x single-metric jump) is classified as an out-of-range stress condition in `regime_validity.md`; treat as accepted limitation boundary, not as a new guardrail target.
- S1 fragility at `SE_mult=5.0` and `10.0` is also out-of-range under realistic DSL uncertainty scaling; treat as accepted limitation boundary.
- S5 and S6 BF-related failures are resolved by Session 4.1 log-scaled BF normalization (`bf_max_target=10000`) and require no additional guardrail in this session.

## Status

Specification complete for bead `athena-zvg`.
