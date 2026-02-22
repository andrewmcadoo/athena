# Aggregation Candidate Sweep Summary (Session 2)

Generated: 2026-02-22T06:50:12.860348+00:00

## Stage 1 Normalization Sweep

- Configs evaluated: 81 normalization configs x 3 candidates = 243 candidate-configs
- Top normalization configs:
  - N061: total_passes=10/21, avg_pass_score=0.5757, params={'abs_diff_k': 2000.0, 'abs_diff_x0': 0.0005, 'se_dampen_enabled': True, 'se_dampen_k': 8.0, 'se_dampen_x0': 1.0}
  - N070: total_passes=10/21, avg_pass_score=0.5757, params={'abs_diff_k': 2000.0, 'abs_diff_x0': 0.0007, 'se_dampen_enabled': True, 'se_dampen_k': 8.0, 'se_dampen_x0': 1.0}
  - N034: total_passes=10/21, avg_pass_score=0.5757, params={'abs_diff_k': 1200.0, 'abs_diff_x0': 0.0005, 'se_dampen_enabled': True, 'se_dampen_k': 8.0, 'se_dampen_x0': 1.0}
- Winner: N061 (used for Stage 2)

## Stage 2 Candidate Sweeps

- Candidate-configs evaluated: 480 total (including Fisher isolation runs)

### IVW-CDF Top 5

| Rank | Norm | Passes | Avg pass score | Failed scenarios | Config |
| --- | --- | --- | --- | --- | --- |
| 1 | N061 | 2/7 | 0.7793 | S1,S2,S4,S5,S6 | `{"eps": 1e-12, "multiplicity_bonus_enabled": true, "multiplicity_scale": 0.5, "multiplicity_threshold": 0.05, "w_default": 2.0}` |
| 2 | N061 | 2/7 | 0.7793 | S1,S2,S4,S5,S6 | `{"eps": 1e-12, "multiplicity_bonus_enabled": true, "multiplicity_scale": 0.5, "multiplicity_threshold": 0.1, "w_default": 2.0}` |
| 3 | N061 | 2/7 | 0.7793 | S1,S2,S4,S5,S6 | `{"eps": 1e-12, "multiplicity_bonus_enabled": true, "multiplicity_scale": 0.5, "multiplicity_threshold": 0.05, "w_default": 1.0}` |
| 4 | N061 | 2/7 | 0.7793 | S1,S2,S4,S5,S6 | `{"eps": 1e-12, "multiplicity_bonus_enabled": true, "multiplicity_scale": 0.5, "multiplicity_threshold": 0.1, "w_default": 1.0}` |
| 5 | N061 | 2/7 | 0.7793 | S1,S2,S4,S5,S6 | `{"eps": 1e-12, "multiplicity_bonus_enabled": true, "multiplicity_scale": 0.5, "multiplicity_threshold": 0.05, "w_default": 0.5}` |

### HTG-Max Top 5

| Rank | Norm | Passes | Avg pass score | Failed scenarios | Config |
| --- | --- | --- | --- | --- | --- |
| 1 | N061 | 5/7 | 0.7565 | S5,S6 | `{"alpha": 1.5, "c_floor": 0.7, "eps": 1e-12, "lse_beta": 0.5, "mode": "lse_rebound", "soft_sum_boost": 2.0, "tau": 5.0}` |
| 2 | N061 | 5/7 | 0.7565 | S5,S6 | `{"alpha": 2.0, "c_floor": 0.7, "eps": 1e-12, "lse_beta": 0.5, "mode": "lse_rebound", "soft_sum_boost": 2.0, "tau": 5.0}` |
| 3 | N061 | 5/7 | 0.7561 | S5,S6 | `{"alpha": 1.0, "c_floor": 0.7, "eps": 1e-12, "lse_beta": 0.5, "mode": "lse_rebound", "soft_sum_boost": 2.0, "tau": 5.0}` |
| 4 | N061 | 5/7 | 0.7556 | S5,S6 | `{"alpha": 1.5, "c_floor": 0.5, "eps": 1e-12, "lse_beta": 0.5, "mode": "lse_rebound", "soft_sum_boost": 2.0, "tau": 5.0}` |
| 5 | N061 | 5/7 | 0.7556 | S5,S6 | `{"alpha": 2.0, "c_floor": 0.5, "eps": 1e-12, "lse_beta": 0.5, "mode": "lse_rebound", "soft_sum_boost": 2.0, "tau": 5.0}` |

### Fisher-UP Top 5 (main sweep)

| Rank | Norm | Passes | Avg pass score | Failed scenarios | Config |
| --- | --- | --- | --- | --- | --- |
| 1 | N061 | 4/7 | 0.4904 | S1,S2,S5 | `{"n_ref": 50.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 2.0, "se_reliability_x0": 3.0}` |
| 2 | N061 | 4/7 | 0.4836 | S1,S2,S5 | `{"n_ref": 100.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 2.0, "se_reliability_x0": 3.0}` |
| 3 | N061 | 4/7 | 0.4792 | S1,S2,S5 | `{"n_ref": 50.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 3.0, "se_reliability_x0": 3.0}` |
| 4 | N061 | 4/7 | 0.4723 | S1,S2,S5 | `{"n_ref": 100.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 3.0, "se_reliability_x0": 3.0}` |
| 5 | N061 | 4/7 | 0.4499 | S1,S2,S5 | `{"n_ref": 50.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 5.0, "se_reliability_x0": 3.0}` |

## 7/7 Configurations

- No 7/7 configuration found in Stage 2.

## S2 Criterion Sensitivity Frontier

Configs considered: those with 6/7 passes and only `S2` failing under the default multiplier `1.5`.

| Candidate | Qualifying configs | Best ratio agg/max_single | Max multiplier passed (1.0-2.0 grid) |
| --- | --- | --- | --- |
| IVW-CDF | 0 | n/a | n/a |
| HTG-Max | 0 | n/a | n/a |
| Fisher-UP | 0 | n/a | n/a |

## Fisher SE-Reliability Isolation (se_dampen=False)

| Sweep | Passes | Avg pass score | Failed scenarios | Norm | Config |
| --- | --- | --- | --- | --- | --- |
| Main (se_dampen=True) | 4/7 | 0.4904 | S1,S2,S5 | N061 | `{"n_ref": 50.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 2.0, "se_reliability_x0": 3.0}` |
| Isolation (se_dampen=False) | 5/7 | 0.5891 | S1,S2 | N061 | `{"n_ref": 50.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": false, "se_reliability_enabled": true, "se_reliability_k": 2.0, "se_reliability_x0": 3.0}` |

## Best Per Candidate

| Candidate | Passes | Failed scenarios | Norm | Config |
| --- | --- | --- | --- | --- |
| IVW-CDF | 2/7 | S1,S2,S4,S5,S6 | N061 | `{"eps": 1e-12, "multiplicity_bonus_enabled": true, "multiplicity_scale": 0.5, "multiplicity_threshold": 0.05, "w_default": 2.0}` |
| HTG-Max | 5/7 | S5,S6 | N061 | `{"alpha": 1.5, "c_floor": 0.7, "eps": 1e-12, "lse_beta": 0.5, "mode": "lse_rebound", "soft_sum_boost": 2.0, "tau": 5.0}` |
| Fisher-UP | 4/7 | S1,S2,S5 | N061 | `{"n_ref": 50.0, "p_eps": 1e-12, "r_floor": 0.7, "se_dampen_enabled": true, "se_reliability_enabled": true, "se_reliability_k": 2.0, "se_reliability_x0": 3.0}` |

