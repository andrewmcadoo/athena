# Session 5 Stretch Summary

Generated: 2026-02-22T22:35:48.423214+00:00

Hybrid: HTG gating + Fisher product, log-scaled BF normalization (c=0.083647, bf_max_target=10000).

## Phase 0 — Baseline Verification

| Scenario | Pass | Margin | Baseline margin | Delta |
| :--- | :---: | ---: | ---: | ---: |
| S1 Noisy TV | PASS | +0.030647 | +0.030647 | +0.000e+00 |
| S2 Unanimous weak signal | PASS | +0.072804 | +0.072804 | +0.000e+00 |
| S3 Mixed signal | PASS | +0.006164 | +0.006164 | +0.000e+00 |
| S4 Missing data | PASS | +0.128007 | +0.128007 | +0.000e+00 |
| S5 Scale heterogeneity | PASS | +0.008802 | +0.008802 | +4.508e-08 |
| S6 Calibration decomposability | PASS | +0.000000 | +0.000000 | +0.000e+00 |
| S7 Boundary-seeking | PASS | +0.102971 | +0.102971 | +0.000e+00 |

## Phase 1 — Calibration

| Pattern | Metric | Value | Max delta | Metric pass | Smoothness pass | Overall |
| :--- | :--- | ---: | ---: | :---: | :---: | :---: |
| A Gradual convergence | spearman_rho | -1.0000 | 0.0493 | PASS | PASS | PASS |
| B Sudden regime change | step_ratio | 1.0290 | 0.0282 | FAIL | PASS | FAIL |
| C Oscillating uncertainty | pearson_r | -0.9341 | 0.0000 | PASS | PASS | PASS |

### Pattern B Narrative

Key question: does the hybrid respond to sudden single-metric regime change where Session 2 single-family candidates failed?

Hybrid Pattern B: step_ratio=1.0290, max_delta=0.0282, classification=non-responsive but smooth.

### Session 2 Calibration Comparison

| Pattern | Hybrid (value, delta, overall) | IVW-CDF | HTG-Max | Fisher-UP |
| :--- | :--- | :--- | :--- | :--- |
| A Gradual convergence | -1.0000, 0.0493, PASS | -0.8728, 0.1064, FAIL | -1.0000, 0.0010, PASS | -1.0000, 0.0386, PASS |
| B Sudden regime change | 1.0290, 0.0282, FAIL | 2.9533, 0.5677, FAIL | 1.0036, 0.0035, FAIL | 2553.2200, 0.9996, FAIL |
| C Oscillating uncertainty | -0.9341, 0.0000, PASS | 0.0000, 0.0000, FAIL | -0.8784, 0.0000, PASS | -0.9635, 0.0017, PASS |

## Phase 2 — Correlation Robustness

| rho | mean unc | mean cor | inflation | var(T) | eff_df | corr_terms | floor_count | floor_saturated |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | :---: |
| 0.0 | 9.999914e-01 | 9.542017e-01 | 1.0480 | 4.823486e+00 | 14.93 | 7 | 0 | no |
| 0.3 | 9.999896e-01 | 9.913926e-01 | 1.0087 | 6.592648e+00 | 10.92 | 5 | 0 | no |
| 0.5 | 9.999866e-01 | 9.965461e-01 | 1.0035 | 7.948960e+00 | 9.06 | 4 | 0 | no |
| 0.7 | 9.999876e-01 | 9.968506e-01 | 1.0031 | 8.128999e+00 | 8.86 | 4 | 0 | no |
| 0.9 | 9.999846e-01 | 9.963658e-01 | 1.0036 | 8.683527e+00 | 8.29 | 4 | 0 | no |

### Session 2 Correlation Comparison

| rho | Hybrid inflation | Hybrid floor-saturated | Session 2 inflation | Session 2 floor-saturated (inferred) |
| ---: | ---: | :---: | ---: | :---: |
| 0.0 | 1.0480 | no | 1.0000 | yes |
| 0.3 | 1.0087 | no | 1.0000 | yes |
| 0.5 | 1.0035 | no | 1.0000 | yes |
| 0.7 | 1.0031 | no | 1.0025 | yes |
| 0.9 | 1.0036 | no | 1.0000 | yes |

## Summary Verdict

- Phase 0 gate: PASS
- Pattern B classification: non-responsive but smooth
- Correlation pass at rho=0.5: PASS
- Floor-saturation clear across all rho: PASS
