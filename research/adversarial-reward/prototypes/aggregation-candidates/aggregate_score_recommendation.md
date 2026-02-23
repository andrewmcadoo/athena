# AggregateScore Recommendation

**Status**: LOCKED
**Version**: 1.0
**Date**: 2026-02-22
**Bead**: athena-6ax
**Supersedes**: All prior candidate evaluations (Sessions 1-5, 4.1, 4.2/4.3)

## 1. Recommended Algorithm

**HTG-gated Fisher product hybrid** with log-scaled BF normalization, `n_terms=1`.

### 1.1 Pipeline (per component)

```
Input: Vec<MetricComponent>

For each component_i:
  1. score_i       = normalize_component(component_i, normalization_config)
  2. precision_i   = gate_precision(component_i, eps)
                     // = log1p(sample_size / (standard_error^2 + eps))
                     // = None when uncertainty absent
  3. confidence_i  = max(c_floor, sigmoid(precision_i, alpha, tau))  if precision_i exists
                     c_missing                                        if precision_i is None
  4. gated_i       = score_i * confidence_i
  5. p_i           = max(p_eps, 1.0 - gated_i)
  6. evidence_i    = -2 * log(p_i)

Aggregate:
  7. T             = sum(evidence_i)
  8. aggregate     = chi_square_cdf(T, df=2)    // n_terms=1 => df=2
                     = 1 - exp(-T/2)

Decomposition:
  9. weight_i      = evidence_i * (aggregate / sum(evidence_j * score_j))
 10. contribution_i = weight_i * score_i
     // sum(contribution_i) = aggregate (exact reconstruction)
```

### 1.2 BF Normalization

The default `bf/(bf+1)` mapping is replaced by log-scaled normalization for `DivergenceKind.BayesFactor`:

```
bf_norm_log_scaled(bf, c) = log1p(bf) / (log1p(bf) + c)
```

where `c` is calibrated so the function reaches a target score at `bf_max_target`:

```
c = log1p(bf_max_target) * (1 - target_score) / target_score
```

With `bf_max_target=10000` and `target_score = 1 - 1/(1 + bf_max_target)`:

```
c = 0.083647
```

All other normalization branches (`ZScore`, `KLDivergence`, `AbsoluteDifference`, `EffectSize`, `Custom`) are unchanged from the Session 1 baseline.

### 1.3 Fixed Parameters

| Parameter | Value | Source |
| :--- | ---: | :--- |
| `alpha` (confidence sigmoid steepness) | 1.5 | `HybridConfig` default, Session 3 |
| `tau` (confidence sigmoid midpoint) | 5.0 | `HybridConfig` default, Session 3 |
| `c_floor` (minimum confidence, uncertainty present) | 0.1 | `HybridConfig` default, Session 3 |
| `c_missing` (confidence when uncertainty absent) | 0.7 | `HybridConfig` default, Session 3 |
| `p_eps` (p-value floor) | 1e-12 | `HybridConfig` default, Session 3 |
| `eps` (numerical stability) | 1e-12 | `HybridConfig` default, Session 3 |
| `n_terms` (Fisher chi-square df/2) | 1 | `aggregate_hybrid()`, Session 3 |
| `bf_norm_c` (BF log-scale constant) | 0.083647 | `ceiling_analysis.py`, Session 4.1 |
| `bf_max_target` | 10000 | Ceiling analysis decision, Session 4.1 |
| `clip_eps` (unit-interval clamp) | 1e-12 | `NormalizationConfig` default |
| `absolute_difference_sigmoid.k` | 1200.0 | `NormalizationConfig` default |
| `absolute_difference_sigmoid.x0` | 7e-4 | `NormalizationConfig` default |

### 1.4 Guardrail

**GR-S2-CUSTOM-SIGMOID-X0-NONNEG**: For all entries in `NormalizationConfig.custom_sigmoids`, require `x0 >= 0`.

- Enforcement: reject configuration with explicit error at construction time.
- No silent clamping or auto-correction.
- Full specification: `guardrail_spec.md`.

### 1.5 Output Contract

```
AggregateResult:
  candidate:        "Hybrid"
  aggregate_score:  float in (0, 1)     // bounded by clip_eps
  contributions:    Vec<ComponentContribution>
                    // sum(contribution_i) = aggregate_score (exact)
  skipped:          Vec<method_ref>      // excluded components
  warnings:         Vec<str>             // normalization warnings
```

## 2. Evidence Map

Each design choice is traced to one or more evidence artifacts.

### 2.1 Algorithm Selection (HTG-gated Fisher product)

| Decision | Evidence | Key result |
| :--- | :--- | :--- |
| Hybrid over single-family candidates | `results.json` (S3), FINDINGS.md S3 entry | Hybrid 7/7; IVW 5/7, HTG 5/7, Fisher 3/7 |
| HTG gating provides Noisy-TV resistance | `results.json` (S1), FINDINGS.md S1 entry | HTG passes S1; IVW and Fisher fail S1 |
| Fisher product provides weak-signal compounding | `results.json` (S3), FINDINGS.md S3 entry | Hybrid S2 ratio=1.609 (margin +7.3%) |
| `n_terms=1` (df=2) for chi-square CDF | `candidates.py:433`, FINDINGS.md S3 entry | Avoids over-aggregation; 7/7 with single-term |
| No single-family config achieves 7/7 | `sweep_results.json` (S2), FINDINGS.md S2 entry | Best single: HTG 5/7, even after 723-config sweep |

### 2.2 BF Normalization (log-scaled, bf_max=10000)

| Decision | Evidence | Key result |
| :--- | :--- | :--- |
| Old bf/(bf+1) ceiling at BF=110 | `perturbation_summary.md` S5 table | PASS@110 (margin +0.000009), FAIL@120 |
| Log-scaled extends ceiling to BF=9999 | `ceiling_analysis.md` Section 3 | S5 pass through BF=1000, margin +0.003 |
| bf_max=10000 over lower targets | `ceiling_analysis.md` Section 2 | Max ceiling headroom while retaining 7/7 and pre-filter |
| S6 side-benefit: recovers all 5 failing cells | `ceiling_analysis.md` Section 4 | All (d_mid, bf) failing cells now pass |
| 15 candidates evaluated, 13/13 viable passed 7/7 | `ceiling_analysis.json` Phase 3 | Log-scaled best by ceiling metric |

### 2.3 Guardrail (x0 >= 0)

| Decision | Evidence | Key result |
| :--- | :--- | :--- |
| S2 failure localized to x0=-0.2, k>=2.0 | `perturbation_summary.md` S2 grid | 4/24 failures, all at negative x0 |
| x0>=0 eliminates entire failure locus | `perturbation_summary.md` S2 grid | All x0>=0 rows pass across all k values |
| Negative x0 is out-of-range for DSL workflows | `regime_validity.md` parameter table | Realistic x0 in [0.0, 0.5], confidence MEDIUM |
| Enforcement: reject, not clamp | `guardrail_spec.md` | Prevents silent misconfiguration |

### 2.4 Session 5 Stretch Validation

| Decision | Evidence | Key result |
| :--- | :--- | :--- |
| Post-ceiling baseline holds 7/7 | `stretch_summary.md` Phase 0 | All margins match ceiling_analysis baseline |
| Pattern A (gradual convergence) PASS | `stretch_summary.md` Phase 1 | spearman_rho=-1.0, max_delta=0.049 |
| Pattern C (oscillating uncertainty) PASS | `stretch_summary.md` Phase 1 | pearson_r=-0.934, max_delta~0 |
| Correlation robustness PASS at all rho | `stretch_summary.md` Phase 2 | inflation<=1.048, floor_saturated=False everywhere |
| Floor-saturation pathology resolved | `stretch_summary.md` S2 comparison | S2 floor_saturated=yes -> S5 floor_saturated=no |

## 3. Operating Boundaries

### 3.1 Resolved Risks

These failure modes existed in earlier sessions and have been fixed by design changes included in this recommendation.

| Risk | Resolution | Evidence |
| :--- | :--- | :--- |
| S5 BF ceiling at BF=110 | Log-scaled BF normalization (bf_max=10000) | `ceiling_analysis.md`: positive S5 margins through BF=1000 |
| S6 compression failures at high d_mid + BF | Same BF normalization change | `ceiling_analysis.md` Section 4: all 5 failing cells recovered |
| S2 custom sigmoid fragility at negative x0 | GR-S2-CUSTOM-SIGMOID-X0-NONNEG guardrail | `guardrail_spec.md`: config-time rejection prevents entry |
| Session 2 correlation floor-saturation | S6-based probe with non-floor aggregates | `stretch_summary.md` Phase 2: floor_count=0 at all rho |

### 3.2 Accepted Limitations

These are known behaviors that fall outside realistic DSL operating ranges and are accepted as stress-test boundaries, not architectural defects.

#### L1: Pattern B Under-Response

- **Behavior**: Hybrid step_ratio=1.029 under 50x isolated single-metric jump (threshold >3.0 for PASS).
- **Classification**: Out-of-range stress condition.
- **Rationale**: 50x isolated one-metric jump is not representative of physically valid behavior in OpenMM, GROMACS, CESM, or VASP. Multi-observable regime shifts in these domains are coupled and temporally distributed, not instantaneous single-metric spikes.
- **Confidence**: MEDIUM.
- **Evidence**: `regime_validity.md` (Domain Check table), `stretch_summary.md` (Pattern B narrative).
- **Impact on downstream**: Hybrid may not sharply respond to extreme isolated-metric regime changes. In real workflows, such events indicate run-quality failure rather than genuine scientific signal.

#### L2: S1 SE Multiplier Fragility at 5x/10x

- **Behavior**: Hybrid fails Noisy-TV (S1) criterion when standard-error scaling exceeds 5x baseline.
- **Classification**: Out-of-range stress condition.
- **Rationale**: Realistic SE multiplier band is [0.5, 3.0] for quality-controlled DSL campaigns. 5x-10x maps to under-sampled or non-equilibrated runs.
- **Confidence**: MEDIUM.
- **Evidence**: `regime_validity.md` (parameter range table), `perturbation_summary.md` (S1 axis).
- **Impact on downstream**: Runs with extreme uncertainty inflation should be filtered by upstream QA before reaching the aggregator, not handled by aggregation-level robustness.

### 3.3 Operating Envelope Summary

| Parameter | Validated range | Boundary behavior |
| :--- | :--- | :--- |
| Bayes factor | 1 - 1000 | Positive margins through BF=1000 (log-scaled) |
| Custom sigmoid x0 | [0.0, 0.5] | Guardrail enforces x0>=0 |
| Custom sigmoid k | [0.8, 3.0] | All tested k values pass when x0>=0 |
| SE multiplier | [0.5, 3.0] | Stable; fails at 5x+ |
| Missing uncertainty count | 0 - 2 per bundle | S4 pass with margin +0.128 |
| Correlation (rho) | [0.0, 0.9] | inflation<=1.048, not floor-saturated |

## 4. Revisit Triggers

This recommendation must be reopened if any of the following conditions are met.

### T1: Empirical Operating Range Violation

**Trigger**: Production DSL trace distributions show parameter values regularly entering regions classified as out-of-range in `regime_validity.md`.

**Specifically**:
- Custom sigmoid x0 values frequently below 0 in legitimate configurations.
- SE multipliers consistently above 3.0 in quality-controlled runs.
- Isolated single-metric jumps above 5x in valid scientific outputs.

**Action**: Re-run regime validity analysis with empirical distributions. Reclassify failure boundaries. May require guardrail scope changes or aggregation adjustments.

### T2: New DivergenceKind Addition

**Trigger**: A new `DivergenceKind` variant is added to the contract that requires normalization behavior not covered by existing branches.

**Action**: Extend `normalize_component` with the new kind's mapping. Re-run the 7-scenario baseline suite + perturbation sweep on the extended function.

### T3: Pattern B Recovery Becomes Blocking

**Trigger**: Downstream component (e.g., Adversarial Experiment Designer calibration loop per ARCHITECTURE.md 5.4) requires sharp step-response detection for single-metric regime changes, and the current step_ratio=1.029 causes calibration feedback failure.

**Action**: Investigate targeted hybrid adjustments to lift Pattern B step_ratio above 3.0 without reintroducing Fisher-like non-smooth jumps. This is documented as an open thread from Session 5.

### T4: Scenario Suite Expansion

**Trigger**: New adversarial scenarios are added that test failure modes not covered by S1-S7 (e.g., adversarial fixture gaming, metric-count scaling, temporal autocorrelation).

**Action**: Re-evaluate hybrid against expanded suite. If new failures emerge in-range, assess whether they require normalization changes, parameter adjustments, or architectural redesign.

### T5: Correlation Structure Change

**Trigger**: Production traces exhibit correlation structures where inflation ratio exceeds 1.5 at rho=0.5 or floor-saturation returns under non-S6 fixture families.

**Action**: Re-run correlation robustness probes. Evaluate whether Brown-style correction needs adjustment or whether the hybrid requires explicit correlation-aware combination logic.

## 5. Prototype Artifact Cross-Reference

| Artifact | Session | Role in this recommendation |
| :--- | :--- | :--- |
| `candidates.py` | S3 | Canonical `aggregate_hybrid()` + `HybridConfig` |
| `normalization.py` | S1 | All normalization branches except BF log-scaled |
| `ceiling_analysis.py` | S4.1 | `bf_norm_log_scaled()` definition and calibration |
| `evaluate.py` | S3 | 7-scenario evaluation harness |
| `perturbation_test.py` | S4 | 70-run robustness sweep |
| `stretch_test.py` | S5 | Calibration + correlation post-ceiling validation |
| `perturbation_summary.md` | S4 | Pass-rate matrix, tipping points |
| `ceiling_analysis.md` | S4.1 | BF normalization decision summary |
| `stretch_summary.md` | S5 | Calibration patterns + correlation robustness |
| `regime_validity.md` | S4.2 | In-range vs out-of-range failure classification |
| `guardrail_spec.md` | S4.3 | x0>=0 constraint specification |
| `regime_validity.json` | S4.2 | Machine-readable operating ranges |
| `ceiling_analysis.json` | S4.1 | Machine-readable BF analysis results |

## 6. Architecture Integration Notes

This specification is a research output. It is not production code. The following notes are for the session that implements `AggregateScore` in the ATHENA architecture.

1. **BF normalization seam**: The prototype embeds BF normalization inside `normalize_component()`. Production implementation should expose BF normalization as a first-class configurable hook, not a hard-coded branch. The log-scaled function is the default, but the seam enables future normalization family changes without touching aggregation logic.

2. **Guardrail enforcement point**: The x0>=0 constraint is validated at config construction time, before any aggregation runs. This is a schema-level validation, not a runtime check inside the aggregation loop.

3. **Decomposition contract**: `sum(contribution_i) = aggregate_score` is exact by construction (weight derivation in step 9). This property is required by ARCHITECTURE.md Section 5.4 for calibration feedback. Implementations must preserve this invariant.

4. **n_terms=1 is intentional**: The Fisher combination uses df=2 (single-term chi-square), not df=2N where N is the component count. This prevents over-aggregation where many weak signals compound into artificially high aggregate scores. The gating stage (steps 2-4) handles evidence quality; the Fisher stage (steps 5-8) handles combination.
