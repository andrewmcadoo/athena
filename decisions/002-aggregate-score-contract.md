# ADR 002: Locked AggregateScore Contract for Adversarial Calibration

## Status

Accepted — 2026-02-23

## Context

Session 6 (`athena-6ax`) closed the adversarial-reward aggregation research track with a locked recommendation:

- `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md`
- `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.json`

This decision resolves the architecture-level ambiguity previously captured in `ARCHITECTURE.md` Sections 4.4, 5.4, and 8.1 around reward formalization for the Adversarial Experiment Designer.

The recommendation is explicitly versioned and locked (Version 1.0, 2026-02-22). Integration work must preserve this specification and must not retune algorithm family, parameters, normalization constants, or guardrails during implementation.

## Decision

ATHENA adopts the Session 6 AggregateScore contract as the normative architecture contract for adversarial calibration.

The contract is the HTG-gated Fisher product hybrid with:

- `n_terms=1` (single-term chi-square, `df=2`)
- log-scaled Bayes-factor normalization `bf_norm_log_scaled(bf, c)`
- fixed `bf_norm_c=0.083647` calibrated at `bf_max_target=10000`
- schema-level guardrail `GR-S2-CUSTOM-SIGMOID-X0-NONNEG` (`x0 >= 0` for all custom sigmoid entries)
- decomposition invariant `sum(contribution_i) = aggregate_score`

Locked parameter set:

| Parameter | Value |
| :--- | ---: |
| `alpha` | 1.5 |
| `tau` | 5.0 |
| `c_floor` | 0.1 |
| `c_missing` | 0.7 |
| `p_eps` | 1e-12 |
| `eps` | 1e-12 |
| `n_terms` | 1 |
| `bf_norm_c` | 0.083647 |
| `bf_max_target` | 10000 |
| `clip_eps` | 1e-12 |
| `absolute_difference_sigmoid.k` | 1200.0 |
| `absolute_difference_sigmoid.x0` | 7e-4 |

## Evidence Basis

- Algorithm pipeline and output contract: recommendation Sections 1.1 and 1.5.
- BF normalization family and constant: recommendation Section 1.2; `ceiling_analysis.md` Sections 3-4.
- Fixed parameters: recommendation Section 1.3; machine-readable values in `aggregate_score_recommendation.json`.
- Guardrail semantics and reject-only behavior: recommendation Section 1.4 and `guardrail_spec.md`.
- Intentional `n_terms=1` (not placeholder): recommendation Section 6 note 4.
- Calibration dependency on exact decomposition: recommendation Section 6 note 3 and `ARCHITECTURE.md` Section 5.4.

## Consequences

- Architecture risk posture changes from "open reward formalization research" to "specified, pending implementation fidelity" for the Adversarial Experiment Designer.
- Implementation work must preserve the exact contract and add explicit checks for:
  - BF normalization seam exposure with locked default (`c=0.083647`)
  - guardrail validation at `NormalizationConfig` construction (`x0 >= 0`, reject-only)
  - decomposition invariant assertion (`1e-8` tolerance)
- Acceptance testing and monitoring are required to enforce the locked contract under future changes.

## Out of Scope for This ADR

- Any retuning of algorithm family, constants, normalization calibration target, or guardrail scope.
- Any change to accepted Session 6 operating boundaries (Pattern B and extreme SE behavior).
- Any production implementation details beyond contract obligations.

## References

- `ARCHITECTURE.md` Sections 4.4.1, 5.4, 8.1, Appendix
- `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md`
- `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.json`
- `research/adversarial-reward/prototypes/aggregation-candidates/ceiling_analysis.md`
- `research/adversarial-reward/prototypes/aggregation-candidates/guardrail_spec.md`
- `research/adversarial-reward/prototypes/aggregation-candidates/regime_validity.md`
