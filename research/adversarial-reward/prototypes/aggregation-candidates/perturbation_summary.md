# Hybrid Perturbation Robustness (Session 4)

Generated: 2026-02-22T09:32:32.856961+00:00

## Baseline Sanity Checks

- S2 margin `((agg/max_single)/1.5)-1`: `+0.072804` (ratio `1.609206`)
- S4 relative delta: `0.071993`

## Top-Level Verdict

| Axis | Scenario | Pass rate | Pass/Total | Min margin | Median margin |
| :--- | :--- | ---: | ---: | ---: | ---: |
| `s2_custom_sigmoid` | S2 Unanimous weak signal | 83.3% | 20/24 | -0.131781 | +0.204245 |
| `s2_non_custom_se_scale` | S2 Unanimous weak signal | 100.0% | 5/5 | +0.054091 | +0.072804 |
| `s5_bayes_factor` | S5 Scale heterogeneity | 44.4% | 4/9 | -0.008001 | -0.000736 |
| `s7_boundary_se` | S7 Boundary-seeking | 100.0% | 7/7 | +0.001838 | +0.023612 |
| `s6_joint_compress` | S6 Calibration decomposability | 68.8% | 11/16 | -0.024749 | +0.000000 |
| `s4_missing_count` | S4 Missing data | 100.0% | 4/4 | +0.094934 | +0.135069 |
| `s1_se_mult` | S1 Noisy TV | 60.0% | 3/5 | -0.023928 | +0.030647 |

## Critical Axis: S2 Custom Sigmoid Margin Grid

| k \\ x0 | -0.2 | 0.0 | 0.2 | 0.5 |
| ---: | ---: | ---: | ---: | ---: |
| 1.0 | +0.06491 (PASS) | +0.15142 (PASS) | +0.25707 (PASS) | +0.46100 (PASS) |
| 1.5 | +0.00218 (PASS) | +0.11693 (PASS) | +0.27183 (PASS) | +0.55668 (PASS) |
| 2.0 | -0.05047 (FAIL) | +0.08494 (PASS) | +0.28696 (PASS) | +0.54427 (PASS) |
| 2.2 | -0.06909 (FAIL) | +0.07280 (PASS) | +0.29312 (PASS) | +0.53950 (PASS) |
| 2.5 | -0.09467 (FAIL) | +0.05526 (PASS) | +0.30247 (PASS) | +0.53257 (PASS) |
| 3.0 | -0.13178 (FAIL) | +0.02773 (PASS) | +0.31838 (PASS) | +0.52168 (PASS) |

## Critical Axis: S5 BayesFactor Sweep

| BF value | Margin | Pass | Max component score | Min component score |
| ---: | ---: | :---: | ---: | ---: |
| 80 | +0.003346 | PASS | 0.987654 | 0.589040 |
| 90 | +0.001989 | PASS | 0.989011 | 0.589040 |
| 100 | +0.000901 | PASS | 0.990099 | 0.589040 |
| 110 | +0.000009 | PASS | 0.990991 | 0.589040 |
| 120 | -0.000736 | FAIL | 0.991736 | 0.589040 |
| 150 | -0.002377 | FAIL | 0.993377 | 0.589040 |
| 200 | -0.004025 | FAIL | 0.995025 | 0.589040 |
| 500 | -0.007004 | FAIL | 0.998004 | 0.589040 |
| 1000 | -0.008001 | FAIL | 0.999001 | 0.589040 |

## Tipping Points

- `s2_custom_sigmoid`
  - x0=-0.2, k 1.5->2: PASS -> FAIL
- `s5_bayes_factor`
  - bf 110->120: PASS -> FAIL
- `s6_joint_compress`
  - d_mid=3, bf 100->500: PASS -> FAIL
  - d_mid=4, bf 12->100: PASS -> FAIL
  - bf=100, d_mid 3->4: PASS -> FAIL
  - bf=500, d_mid 2->3: PASS -> FAIL
  - bf=1000, d_mid 2->3: PASS -> FAIL
