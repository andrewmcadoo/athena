# Bayesian Surprise Over Causal DAG Structures

## Research Question

How should KL divergence be computed over causal DAG structures (rather than parameter spaces) to quantify epistemic value and guide experiment selection? The Bayesian Surprise Evaluator serves dual roles: post-experiment, it computes actual surprise from results; pre-experiment, it estimates predicted information gain by forward-simulating candidates against the DAG. The pre-experiment role is the harder problem — it requires defining a probability distribution over graph structures and efficiently computing expected divergence. Success criteria: a specified method for computing predicted KL divergence over DAG structures that is tractable for graphs of the scale ATHENA operates on, with characterized approximation error bounds.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.5 (Bayesian Surprise Evaluator) | Component definition — dual post-experiment/pre-experiment responsibility |
| ARCHITECTURE.md | 5.4 (Adversarial Calibration Feedback) | Predicted vs. actual surprise comparison for calibration |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: Low-Medium. Primarily engineering with some adaptation research |
| VISION.md | Section 3 (The Insight) | Bayesian surprise as KL divergence between prior causal model and posterior |
| Constraint | Bounded Adversarial Design | Forward simulation must respect deterministic subspace bounds |

## Status

NOT STARTED

## Key Definitions

- **KL divergence over DAGs**: A measure of the information gained by updating from a prior belief distribution over causal graph structures to a posterior belief distribution after observing experimental results. Unlike standard KL divergence over parameter vectors, this operates over discrete combinatorial structures.
- **Forward simulation**: The process of predicting what experimental results a candidate experiment would produce under the current DAG, used to estimate information gain before running the experiment. Computationally expensive because it requires marginalizing over uncertainty in the graph.
- **Graph-structured distribution**: A probability distribution where the sample space is the set of possible DAG structures (edge presence/absence, edge direction, edge weights). Defining and computing over this space is non-trivial.
- **Surprise invalidation**: When the LFI classifies a failure as an implementation artifact, the surprise value from that experiment is marked unreliable and excluded from trend data. The surprise computation must support this retroactive invalidation.

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

1. **Survey information-theoretic quantities over graph structures** — Review how KL divergence, mutual information, and entropy are defined and computed over discrete graph structures in the causal inference literature. Identify tractable approximations. Scope: 2-3 sessions.

2. **Analyze computational complexity of forward simulation** — Characterize the cost of forward-simulating a candidate experiment against the current DAG. How does cost scale with graph size, number of uncertain edges, and simulation fidelity? Identify the dominant computational bottleneck. Scope: 1-2 sessions.

3. **Coordinate with adversarial-reward investigation** — The reward function for the Adversarial Experiment Designer depends on predicted information gain from this component. Ensure the surprise computation's functional form is compatible with the reward function requirements being developed in `research/adversarial-reward/`. Scope: ongoing coordination.

4. **Survey approximation methods** — Exact KL divergence over DAG structures is likely intractable for non-trivial graphs. Survey approximation methods: variational bounds, sampling-based estimates, local approximations. Characterize accuracy-cost tradeoffs. Scope: 2-3 sessions.

5. **Investigate the invalidation mechanism** — How does retroactive surprise invalidation (triggered by LFI implementation-artifact classification) interact with the trend data used by the Mode Controller? Are there edge cases where invalidation distorts convergence signals? Scope: 1 session.
