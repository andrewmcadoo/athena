# Adversarial Reward Function: Epistemic Information Gain

## Research Question

How should "epistemic information gain within a bounded, deterministic subspace" be formalized as a reward function for the Adversarial Experiment Designer? The formalization must satisfy three competing constraints: it must drive the adversary toward experiments that maximally stress-test hypotheses (information-seeking), it must exclude experiments in stochastic/unlearnable regions that trigger Noisy TV degeneration (noise-avoiding), and it must remain computationally tractable when evaluated via forward simulation against the causal DAG (efficiency). Success criteria: a formal reward function specification with defined behavior across the conservatism-vs-boundary-seeking failure spectrum, validated against at least one synthetic scenario. This investigation blocks adversarial experiment design.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.4 (Adversarial Experiment Designer) | Component definition — maximizes expected epistemic gain within bounded subspaces |
| ARCHITECTURE.md | 5.4 (Adversarial Calibration Feedback) | Calibration loop: predicted vs. actual surprise, persistent miscalibration triggers |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: High. Reward function formalization is an open research problem |
| VISION.md | Section 4.2 (Adversarial Experiment Design) | Bounded active learning, Noisy TV constraint |
| VISION.md | Section 6.2 (Noisy TV Problem) | Pathological adversarial design failure modes |
| VISION.md | Open Question #3 | "Tuning the Adversarial Reward Function" — must penalize unlearnable stochasticity |
| Constraint | Bounded Adversarial Design | Non-negotiable: adversary restricted to deterministic, domain-valid subspaces |

## Status

NOT STARTED

## Key Definitions

- **Epistemic information gain**: The expected reduction in uncertainty about the causal DAG structure resulting from an experiment's outcome. Formally related to KL divergence between prior and expected posterior beliefs.
- **Bounded subspace**: The deterministic, domain-valid region of experiment parameter space defined by the DSL Environment Interface's constraint specifications. The adversary cannot propose experiments outside this region.
- **Noisy TV degeneration**: A failure mode where a surprise-maximizing agent becomes fixated on stochastic, unlearnable noise because such noise consistently yields high prediction error, producing no epistemic value.
- **Conservatism failure**: A failure mode where the reward function is over-constrained, causing the adversary to select safe, uninformative experiments that do not stress-test hypotheses.
- **Boundary-seeking failure**: A failure mode where the adversary clusters experiments at the edge of the valid subspace without yielding theoretical insight, exploiting constraint boundaries rather than probing causal structure.

## Investigation Log

*No entries yet.*

## Accumulated Findings

### What We Know

*No findings yet.*

### What We Suspect

*No findings yet.*

### What We Don't Know

*No findings yet.*

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| *None yet* | | | |

## Next Steps

1. **Survey formalizations in active learning and Bayesian experimental design** — Review how information gain is formalized in discriminative active learning, Bayesian optimization (expected improvement, knowledge gradient), and optimal experimental design. Identify which formalizations handle bounded search spaces. Scope: 2-3 sessions.

2. **Characterize the conservatism-vs-boundary-seeking failure spectrum** — Formally describe the two failure modes as functions of reward function properties. Under what conditions does a given formalization collapse to conservatism? To boundary-seeking? Identify the design parameters that control this tradeoff. Scope: 1-2 sessions.

3. **Analyze Noisy TV in DSL simulation contexts** — The Noisy TV problem is well-studied in RL. Characterize how it manifests specifically in DSL simulation environments: what sources of irreducible stochasticity exist in OpenMM/GROMACS/VASP? How does the DSL's deterministic subspace constraint interact with noise sources? Scope: 1-2 sessions.

4. **Investigate calibration loop constraints on functional form** — The calibration feedback (ARCHITECTURE.md 5.4) compares predicted vs. actual surprise. What constraints does this calibration mechanism place on the reward function's functional form? Must the function be decomposable in specific ways for calibration to be meaningful? Scope: 1-2 sessions.

5. **Draft candidate reward function specifications** — Propose 2-3 candidate formalizations with explicit tradeoff profiles across the failure spectrum. Each should be evaluable against the calibration loop requirements. Scope: 2-3 sessions.
