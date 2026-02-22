# BF Normalization Ceiling Analysis

Generated: 2026-02-22T21:17:51.813147+00:00

## 1. S6 Decomposition

| d_mid | bf_strong | dominant_share | recon_error | failure_is_dominant_share | failure_is_recon_error |
| ---: | ---: | ---: | ---: | :---: | :---: |
| 3.0 | 500.0 | 0.335866 | 0.000e+00 | yes | no |
| 3.0 | 1000.0 | 0.331983 | 0.000e+00 | yes | no |
| 4.0 | 100.0 | 0.347105 | 0.000e+00 | yes | no |
| 4.0 | 500.0 | 0.328977 | 1.110e-16 | yes | no |
| 4.0 | 1000.0 | 0.325251 | 1.110e-16 | yes | no |

All five failing S6 cells are dominant-share failures (`dominant_share < 0.35`) with reconstruction error within tolerance.

## 2. BF Normalization Comparison

| name | bf_ceiling | score@100 | score@500 | score@1000 | pre-filter | 7/7 pass? |
| :--- | ---: | ---: | ---: | ---: | :---: | :---: |
| current_reference | 110 | 0.990099 | 0.998004 | 0.999001 | yes | baseline |
| exp_decay_bfmax_1000 | 999 | 0.375656 | 0.905132 | 0.991000 | yes | yes |
| exp_decay_bfmax_10000 | 9999 | 0.046013 | 0.209845 | 0.375656 | no | not-run |
| exp_decay_bfmax_200 | 199 | 0.905132 | 0.999992 | 1.000000 | yes | yes |
| exp_decay_bfmax_500 | 499 | 0.610194 | 0.991000 | 0.999919 | yes | yes |
| exp_decay_bfmax_5000 | 4999 | 0.089909 | 0.375656 | 0.610194 | no | not-run |
| log_scaled_bfmax_1000 | 999 | 0.986587 | 0.990008 | 0.991000 | yes | yes |
| log_scaled_bfmax_10000 | 9999 | 0.982198 | 0.986723 | 0.988037 | yes | yes |
| log_scaled_bfmax_200 | 199 | 0.989672 | 0.992312 | 0.993077 | yes | yes |
| log_scaled_bfmax_500 | 499 | 0.987915 | 0.991000 | 0.991894 | yes | yes |
| log_scaled_bfmax_5000 | 4999 | 0.983516 | 0.987710 | 0.988928 | yes | yes |
| power_law_bfmax_1000 | 999 | 0.957006 | 0.985572 | 0.991000 | yes | yes |
| power_law_bfmax_10000 | 9999 | 0.905611 | 0.958389 | 0.970794 | yes | yes |
| power_law_bfmax_200 | 199 | 0.983415 | 0.996001 | 0.997838 | yes | yes |
| power_law_bfmax_500 | 499 | 0.969713 | 0.991000 | 0.994673 | yes | yes |
| power_law_bfmax_5000 | 4999 | 0.922104 | 0.967874 | 0.978091 | yes | yes |

## 3. Best Candidate Detail

Best candidate: `log_scaled_bfmax_10000` (family=`log_scaled`, bf_max_target=`10000`, c=`0.083647`, bf_ceiling=`9999`)

### Baseline 7-scenario margins

| scenario | pass | margin | margin_label |
| :--- | :---: | ---: | :--- |
| S1 Noisy TV | PASS | +0.030647 | base-doubled |
| S2 Unanimous weak signal | PASS | +0.072804 | (aggregate/max_single)/1.5-1 |
| S3 Mixed signal | PASS | +0.006164 | min(mixed-lo,hi-mixed) |
| S4 Missing data | PASS | +0.128007 | 0.20-relative_delta |
| S5 Scale heterogeneity | PASS | +0.008802 | min(min(score)-0.3,0.991-max(score)) |
| S6 Calibration decomposability | PASS | +0.000000 | min(dominant_share-0.35,1e-8-abs(recon-aggregate)) |
| S7 Boundary-seeking | PASS | +0.102971 | non_boundary-boundary |

### S5 BF sweep

| BF | pass | margin | max_component | min_component |
| ---: | :---: | ---: | ---: | ---: |
| 80 | PASS | +0.009679 | 0.981321 | 0.589040 |
| 100 | PASS | +0.008802 | 0.982198 | 0.589040 |
| 120 | PASS | +0.008143 | 0.982857 | 0.589040 |
| 200 | PASS | +0.006528 | 0.984472 | 0.589040 |
| 500 | PASS | +0.004277 | 0.986723 | 0.589040 |
| 1000 | PASS | +0.002963 | 0.988037 | 0.589040 |

## 4. S6 Side-Benefit

| candidate | bf_max_target | improved failing S6 cells |
| :--- | ---: | :--- |
| log_scaled_bfmax_200 | 200 | (d_mid=3.0, bf=500.0) |
| power_law_bfmax_200 | 200 | (d_mid=4.0, bf=100.0) |
| exp_decay_bfmax_200 | 200 | (d_mid=4.0, bf=100.0) |
| log_scaled_bfmax_500 | 500 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0) |
| power_law_bfmax_500 | 500 | (d_mid=3.0, bf=500.0), (d_mid=4.0, bf=100.0) |
| exp_decay_bfmax_500 | 500 | (d_mid=3.0, bf=500.0), (d_mid=4.0, bf=100.0) |
| log_scaled_bfmax_1000 | 1000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0) |
| power_law_bfmax_1000 | 1000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0), (d_mid=4.0, bf=500.0) |
| exp_decay_bfmax_1000 | 1000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0), (d_mid=4.0, bf=500.0) |
| log_scaled_bfmax_5000 | 5000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0), (d_mid=4.0, bf=500.0) |
| power_law_bfmax_5000 | 5000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0), (d_mid=4.0, bf=500.0), (d_mid=4.0, bf=1000.0) |
| log_scaled_bfmax_10000 | 10000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0), (d_mid=4.0, bf=500.0), (d_mid=4.0, bf=1000.0) |
| power_law_bfmax_10000 | 10000 | (d_mid=3.0, bf=500.0), (d_mid=3.0, bf=1000.0), (d_mid=4.0, bf=100.0), (d_mid=4.0, bf=500.0), (d_mid=4.0, bf=1000.0) |

## 5. Recommendation

- Decision: GO for athena-e2a adoption.
- Recommended normalization family: `log_scaled`
- Recommended bf_max_target: `10000`
- Evidence: baseline 7/7 = `True`, bf_ceiling = `9999`, S5 pass at BF>=500 = `True`.
