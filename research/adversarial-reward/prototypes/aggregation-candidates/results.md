# Aggregation Candidate Evaluation (Session 1 Prototype)

Generated: 2026-02-22T06:50:11.543669+00:00

## 3x7 Matrix (Primary Candidates)

| Candidate | S1 Noisy TV | S2 Unanimous weak signal | S3 Mixed signal | S4 Missing data | S5 Scale heterogeneity | S6 Calibration decomposability | S7 Boundary-seeking |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IVW-CDF | base=0.6457, doubled=0.8849 (FAIL) | agg=0.2698, max1=0.5818, target=0.8726 (FAIL) | mixed=0.5171, allC=0.6761, allA=0.3239 (PASS) | missing=0.5300, baseline=0.5261, delta=0.007 (PASS) | agg=0.8445, scores=[0.9545, 0.9901, 0.589] (PASS) | agg=0.8684, recon=0.8684, dom_share=0.657 (PASS) | boundary=0.6344, non_boundary=0.7686 (PASS) |
| HTG-Max | base=0.1165, doubled=0.0238 (PASS) | agg=0.3033, max1=0.3033, target=0.4549 (FAIL) | mixed=0.3795, allC=0.3795, allA=0.1774 (PASS) | missing=0.1593, baseline=0.3423, delta=0.535 (FAIL) | agg=0.5712, scores=[0.9545, 0.9901, 0.589] (PASS) | agg=0.9618, recon=0.9618, dom_share=1.000 (PASS) | boundary=0.3022, non_boundary=0.5085 (PASS) |
| Fisher-UP | base=0.5639, doubled=0.8227 (FAIL) | agg=0.0056, max1=0.5818, target=0.8726 (FAIL) | mixed=0.4002, allC=0.7134, allA=0.0454 (PASS) | missing=0.0662, baseline=0.3250, delta=0.796 (FAIL) | agg=0.9914, scores=[0.9545, 0.9901, 0.589] (PASS) | agg=0.9784, recon=0.9784, dom_share=0.568 (PASS) | boundary=0.9917, non_boundary=0.9917 (FAIL) |

## Scenario Pass Criteria

- S1 Noisy TV: score(value*2, se*2) <= score(value, se)
- S2 Unanimous weak signal: aggregate >= 1.5 * max(single_metric_scores)
- S3 Mixed signal: all_agreement <= mixed <= all_contradiction
- S4 Missing data: finite score and within 20% of full-uncertainty baseline
- S5 Scale heterogeneity: all normalized scores in [0.3, 0.99] (with tolerance) and stable ranking
- S6 Calibration decomposability: sum(w_i*u_i) ~= aggregate and one component clearly dominates
- S7 Boundary-seeking: boundary_case < equivalent_non_boundary_case

## Exploratory HTG Variant

The optional `HTG-Max` LogSumExp variant was also executed with re-bounding `1 - exp(-LSE)`. Scores are recorded in `results.json` under `htg_lse_exploratory`.
