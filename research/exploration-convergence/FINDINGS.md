# Exploration-to-Falsification Convergence Criteria

## Research Question

What formal criteria determine when the Epistemic Explorer's causal graph refinement is "good enough" for the Lakatosian Fault Isolator to operate reliably, triggering the transition from exploration to falsification mode? The criteria must be domain-adaptable (different scientific DSL environments will have different convergence profiles), must avoid both premature transitions (LFI operates on a bad graph) and late transitions (budget wasted on diminishing returns), and must be monitorable from the Mode Controller's available signals. Success criteria: a formal specification of convergence conditions with characterized failure modes for premature and late transitions, validated against at least one synthetic causal graph scenario.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.3 (Epistemic Explorer) | Component definition — convergence criteria are an open research question |
| ARCHITECTURE.md | 6.1 (Exploration to Falsification) | Three trigger conditions: marginal gain decay, edge stability, minimum coverage |
| ARCHITECTURE.md | 6.2 (Falsification to Re-Exploration) | Return path — re-exploration uses same criteria on targeted subgraph |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: Medium. Failure mode is performance degradation, not silent corruption |
| VISION.md | Section 4.4 (Epistemic Exploration Phase) | Exploration phase design and motivation |
| VISION.md | Section 6.1 (Causal Bootstrapping Paradox) | Effectiveness of exploration phase is itself an open research question |
| VISION.md | Open Question #5 | Convergence criteria for exploration phase |

## Status

NOT STARTED

## Key Definitions

- **Marginal information gain decay**: The rate at which each additional exploration probe yields diminishing returns in DAG refinement, as measured by the Bayesian Surprise Evaluator.
- **Edge stability**: A graph property indicating that no edges have been added, removed, or reversed in direction over a sustained window. Edge weights may still adjust.
- **Minimum coverage**: The DAG includes causal variables relevant to the target hypothesis space — the Explorer has probed at minimum the variables the Hypothesis Generator would need to reference.
- **Premature transition**: The graph entering falsification is insufficiently accurate for fault isolation, causing the LFI to systematically misclassify failures.
- **Late transition**: The exploration phase continues past the point of meaningful graph improvement, consuming budget on diminishing returns.

## Investigation Log

*No entries yet.*

## Accumulated Findings

### What We Know

*No findings yet.*

### What We Suspect

*No findings yet.*

### What We Don't Know

*No findings yet.*

## Next Steps

1. **Survey convergence criteria in model-based RL and Bayesian optimization** — Review how model-based RL systems decide when their world model is "good enough" to plan with, and how Bayesian optimization handles the exploration-exploitation transition. Identify transferable criteria. Scope: 2-3 sessions.

2. **Formally analyze the three trigger conditions for pathological cases** — Take the conjunctive conditions from ARCHITECTURE.md 6.1 (marginal gain decay AND edge stability AND minimum coverage) and construct scenarios where they fail: conditions met but graph is wrong, conditions not met but graph is adequate. Characterize the gap between observable signals and actual graph quality. Scope: 2-3 sessions.

3. **Investigate domain-dependence** — How do convergence profiles differ across DSL environments? A molecular dynamics graph (OpenMM) likely has different density, connectivity, and noise characteristics than a climate model graph (CESM). Determine which convergence parameters must be domain-calibrated vs. which can be universal. Scope: 1-2 sessions.

4. **Analyze the re-exploration case** — The return from falsification to re-exploration (Section 6.2) applies convergence criteria to a targeted subgraph rather than the full graph. How do the criteria behave on subgraphs? Are there edge cases where subgraph convergence is misleading? Scope: 1-2 sessions.

5. **Draft candidate convergence specifications** — Propose 2-3 formal convergence criteria with domain-calibration parameters. Each should specify monitoring signals available to the Mode Controller and characterized failure modes. Scope: 2-3 sessions.
