# AggregateScore Acceptance Test Specification (Session 7 Integration)

## Status

Locked-integration spec derived from Session 6 recommendation (Version 1.0, 2026-02-22, bead `athena-6ax`).

## Scope

This document defines implementation acceptance tests for the production AggregateScore contract. It is a translation layer from research evidence to architecture-level gates. It does not change algorithm decisions.

Normative sources:

- `aggregate_score_recommendation.md` Sections 1.1-1.5 and Section 6 notes.
- `aggregate_score_recommendation.json` (`baseline_margins`).
- `ceiling_analysis.md` Sections 3-4.
- `guardrail_spec.md`.
- `regime_validity.md` and `stretch_summary.md`.

## Test Categories

- **Category A (blocking gates)**: Must pass for release.
- **Category B (resolved-risk regressions)**: Must pass to prevent recurrence of known in-range failures.
- **Category C (accepted-boundary assertions)**: Must assert documented limitation behavior; these are not release-blocking failures.

## Category A: Baseline Scenario Gates (Blocking)

Use the canonical seven scenarios (`S1`-`S7`) with the locked configuration.

Pass condition per scenario:

- scenario pass/fail label must be `PASS`
- `abs(observed_margin - baseline_margin_json) <= 1e-6`

Baseline margins (source: `aggregate_score_recommendation.json::baseline_margins`):

| Scenario | Baseline margin |
| :--- | ---: |
| `S1_noisy_tv` | 0.030647 |
| `S2_unanimous_weak_signal` | 0.072804 |
| `S3_mixed_signal` | 0.006164 |
| `S4_missing_data` | 0.128007 |
| `S5_scale_heterogeneity` | 0.008802 |
| `S6_calibration_decomposability` | 0.000000 |
| `S7_boundary_seeking` | 0.102971 |

Evidence:

- `aggregate_score_recommendation.json` (`baseline_margins`)
- `stretch_summary.md` Phase 0 (`1e-6` baseline comparison precedent)

## Category B: Resolved-Risk Regression Gates (Blocking)

### B1. S5 BF-Ceiling Regression

- Fixture: S5 scale heterogeneity.
- Condition: `BF=1000` still passes under locked BF normalization.
- Expected: PASS with positive margin (no return of old `BF>=120` failure boundary).
- Evidence: `ceiling_analysis.md` Section 3 (S5 BF sweep rows through BF=1000).

### B2. S6 Compression Regression

- Fixture: S6 calibration/decomposition stress cells previously failing under old BF normalization.
- Cells to test (all must PASS):
  - `(d_mid=3.0, bf=500.0)`
  - `(d_mid=3.0, bf=1000.0)`
  - `(d_mid=4.0, bf=100.0)`
  - `(d_mid=4.0, bf=500.0)`
  - `(d_mid=4.0, bf=1000.0)`
- Expected: all five cells recover under locked BF mapping.
- Evidence: `ceiling_analysis.md` Section 4.

### B3. S2 Guardrail Regression

- Input: invalid `NormalizationConfig.custom_sigmoids` entry with `x0 < 0`.
- Expected: config construction fails fast with explicit guardrail violation error.
- Error payload must include:
  - guardrail ID: `GR-S2-CUSTOM-SIGMOID-X0-NONNEG`
  - offending `method_ref`
  - offending `x0`
  - expected constraint text: `x0 >= 0`
- Forbidden: silent clamping or auto-correction.
- Evidence: `aggregate_score_recommendation.md` Section 1.4 and `guardrail_spec.md`.

## Category C: Accepted-Boundary Assertions (Non-Blocking)

These tests confirm known out-of-range limitations remain explicitly documented and classified, rather than being misinterpreted as regressions.

### C1. Pattern B Step-Ratio Behavior

- Stimulus: Pattern B sudden isolated jump fixture.
- Observed expected behavior: `step_ratio < 3.0` (historically `1.0290`) may occur.
- Assertion target: classification must be `accepted limitation` / `out-of-range`, not release-fail.
- Evidence: `regime_validity.md` (Pattern B domain check + failure overlay), `stretch_summary.md` Pattern B narrative.

### C2. S1 Extreme SE Multiplier Behavior

- Stimulus: S1 Noisy-TV fixture with `SE_mult >= 5.0`.
- Observed expected behavior: failure can occur at extreme SE inflation.
- Assertion target: classification must be `accepted limitation` / `out-of-range`, not release-fail for in-range contract conformance.
- Evidence: `regime_validity.md` (SE realistic range and failure boundary overlay).

## Decomposition Invariant Gate

Applicable to Categories A and B:

- Require `abs(sum(contribution_i) - aggregate_score) <= 1e-8`.
- Evidence: Session 3 gate and recommendation Section 1.5 + Section 6 note 3.

## Reporting Contract

Each acceptance run must emit:

- contract version under test (`1.0`)
- parameter snapshot (including `n_terms=1`, `bf_norm_c=0.083647`, guardrail enabled)
- per-test result (`PASS`/`FAIL`/`ASSERTED_LIMITATION`)
- evidence reference ID(s) for each assertion

