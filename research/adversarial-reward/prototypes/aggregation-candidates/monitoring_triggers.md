# AggregateScore Monitoring Trigger Contract (Session 7 Integration)

## Status

Future-implementation monitoring specification. ATHENA currently has no production monitoring stack; this document defines what must be wired once telemetry is available.

Normative source: `aggregate_score_recommendation.md` Section 4 (`T1`-`T5`).

## Purpose

Define concrete revisit-trigger checks that reopen the locked recommendation only when specified trigger conditions occur in operation.

## Trigger Matrix

| Trigger | Source signal | Threshold | Owner | Action path |
| :--- | :--- | :--- | :--- | :--- |
| `T1` Empirical operating range violation | Distribution reports from production DSL trace ingestion and config-validation logs (`x0`, `SE_mult`, single-metric jump factors) on QA-approved runs | Trip when out-of-range conditions are recurring in valid workflows (e.g., sustained over 2 consecutive reporting windows): `x0 < 0`, `SE_mult > 3.0`, or isolated jump factor `> 5x` | Adversarial-reward owner + data-ingestion owner | Open bead tagged `revisit-T1`; rerun `regime_validity` analysis with empirical distributions; if confirmed, escalate to research lead and propose updated envelope/guardrail review |
| `T2` New `DivergenceKind` addition | Contract/schema diff checks in CI (enum registry, parser schemas, normalization dispatch map) | Any new `DivergenceKind` variant not covered by existing normalization branches | Aggregation implementation owner | Open bead tagged `revisit-T2`; implement normalization mapping for the new kind; rerun 7-scenario baseline + perturbation/acceptance suite before merge |
| `T3` Pattern B recovery becomes blocking | Calibration-loop requirement changes and downstream error budgets from Adversarial Experiment Designer | Trip when a downstream contract explicitly requires sharp single-metric step-response (`step_ratio >= 3.0`) and current AggregateScore behavior causes calibration-loop failure | Adversarial Experiment Designer owner | Open bead tagged `revisit-T3`; notify research lead; run targeted Pattern B recovery investigation without modifying other locked guardrails until review completes |
| `T4` Scenario suite expansion | Scenario catalog/version manifest changes in evaluation harness (new scenarios beyond S1-S7) | Any approved addition to adversarial scenario suite that is part of release criteria | Evaluation harness owner | Open bead tagged `revisit-T4`; rerun full evaluation on expanded suite; if new in-range failures appear, trigger architecture review with evidence bundle |
| `T5` Correlation structure change | Correlation telemetry from production traces and post-hoc AggregateScore diagnostics (`inflation_ratio`, floor saturation markers) | `inflation_ratio > 1.5` at `rho=0.5` equivalent operating region, or return of floor-saturation in non-S6 fixture families | Bayesian Surprise Evaluator owner | Open bead tagged `revisit-T5`; rerun correlation robustness probes; evaluate Brown-style correction sufficiency vs explicit correlation-aware aggregation changes |

## Minimum Telemetry Requirements

To make these triggers enforceable, implementation must emit:

- normalization/config metrics: `method_ref`, `DivergenceKind`, custom sigmoid `x0/k`, BF values, SE multipliers
- aggregate diagnostics: `aggregate_score`, per-component contributions, decomposition residual
- calibration diagnostics: predicted vs actual surprise deltas by scenario family
- correlation diagnostics: inflation ratio and floor-saturation counters

## Reopen Procedure

When any trigger trips:

1. Create bead with trigger tag (`revisit-T1`..`revisit-T5`) and attach evidence artifact links.
2. Notify adversarial-reward research lead and architecture owner.
3. Freeze contract changes outside the trigger scope.
4. Run only the analyses specified in the trigger action path.
5. Record decision outcome in `research/adversarial-reward/FINDINGS.md` and update ADRs only if contract changes are approved.

