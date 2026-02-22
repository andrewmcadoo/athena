# Calibration Simulation Summary (Session 2 Stretch)

Generated: 2026-02-22T06:50:13.774573+00:00

Base fixture: S6 calibration decomposability (6 metrics), 50 deterministic cycles per pattern.

| Pattern | Candidate | Metric | Value | Threshold | Metric pass | Max delta | Smoothness pass | Overall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A Gradual convergence | IVW-CDF | spearman_rho | -0.8728 | < -0.9 | FAIL | 0.1064 | PASS | FAIL |
| B Sudden regime change | IVW-CDF | step_ratio | 2.9533 | > 3.0 | FAIL | 0.5677 | FAIL | FAIL |
| C Oscillating uncertainty | IVW-CDF | pearson_r | 0.0000 | < -0.5 | FAIL | 0.0000 | PASS | FAIL |
| A Gradual convergence | HTG-Max | spearman_rho | -1.0000 | < -0.9 | PASS | 0.0010 | PASS | PASS |
| B Sudden regime change | HTG-Max | step_ratio | 1.0036 | > 3.0 | FAIL | 0.0035 | PASS | FAIL |
| C Oscillating uncertainty | HTG-Max | pearson_r | -0.8784 | < -0.5 | PASS | 0.0000 | PASS | PASS |
| A Gradual convergence | Fisher-UP | spearman_rho | -1.0000 | < -0.9 | PASS | 0.0386 | PASS | PASS |
| B Sudden regime change | Fisher-UP | step_ratio | 2553.2200 | > 3.0 | PASS | 0.9996 | FAIL | FAIL |
| C Oscillating uncertainty | Fisher-UP | pearson_r | -0.9635 | < -0.5 | PASS | 0.0017 | PASS | PASS |
