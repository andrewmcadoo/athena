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

IN PROGRESS

## Key Definitions

- **Epistemic information gain**: The expected reduction in uncertainty about the causal DAG structure resulting from an experiment's outcome. Formally related to KL divergence between prior and expected posterior beliefs.
- **Bounded subspace**: The deterministic, domain-valid region of experiment parameter space defined by the DSL Environment Interface's constraint specifications. The adversary cannot propose experiments outside this region.
- **Noisy TV degeneration**: A failure mode where a surprise-maximizing agent becomes fixated on stochastic, unlearnable noise because such noise consistently yields high prediction error, producing no epistemic value.
- **Conservatism failure**: A failure mode where the reward function is over-constrained, causing the adversary to select safe, uninformative experiments that do not stress-test hypotheses.
- **Boundary-seeking failure**: A failure mode where the adversary clusters experiments at the edge of the valid subspace without yielding theoretical insight, exploiting constraint boundaries rather than probing causal structure.

## Investigation Log

### 2026-02-22 — WDK#41 Session 1: Candidate Aggregation Prototype + Adversarial Stress Test

**Scope**

- Implement three prototype aggregation candidates mapping `Vec<MetricComponent> -> AggregateScore in [0,1]` under contract-preserving dataclasses.
- Enforce direction-aware CDF normalization (`Agreement` inversion, unsigned handling when direction absent/`None` variant).
- Implement uncertainty-aware weighting/gating to test Noisy-TV resistance and calibration decomposability constraints from `ARCHITECTURE.md` Section 5.4.
- Run a full 3x7 stress-test matrix (3 candidates x 7 scenarios), recording raw scores and per-component `(score, weight)` decomposition.

**Method**

- Added throwaway prototype package at `research/adversarial-reward/prototypes/aggregation-candidates/`:
  - `models.py`: contract-mirroring dataclasses/enums for `MetricComponent`, `UncertaintySummary`, `PointUncertainty`, and `EffectDirection` variants (`Contradiction`, `Agreement`, `None`).
  - `normalization.py`: kind-specific normalization to `[0,1]`:
    - `ZScore`, `EffectSize`: `2 * Phi(|x|) - 1` (stdlib `erf` CDF)
    - `BayesFactor`: `1 - 1/(1+BF)`
    - `KLDivergence`: `1 - exp(-kl)`
    - `AbsoluteDifference`: configurable sigmoid
    - `Custom`: required configurable sigmoid by `method_ref`; missing params => metric excluded + warning (no silent defaults).
  - `candidates.py`:
    - C1 `IVW-CDF`: inverse-variance weighted mean with decomposition-friendly normalized weights.
    - C2 `HTG-Max`: confidence-gated per-kind maxima + hard max across kinds (primary variant).
    - C3 `Fisher-UP`: reliability-adjusted p-value transform + Fisher-style chi-square CDF combination.
    - Optional exploratory variant: `HTG-Max` with LogSumExp + re-bounding (`1-exp(-LSE)`).
  - `scenarios.py`: 7 scenario fixtures with explicit comparator datasets where needed.
  - `evaluate.py`: executes 3x7 matrix, writes `results.json` and `results.md`.
- Removed all SciPy dependencies after runtime import failure; replaced with pure stdlib math:
  - `norm.cdf(z) = 0.5 * (1 + erf(z/sqrt(2)))`
  - `chi2.cdf(x, 2N) = 1 - exp(-x/2) * sum((x/2)^k / k!, k=0..N-1)`.

**Findings**

- Boundedness gate passed globally: all primary candidate outputs were finite and in `[0,1]` across all 21 cells.
- Primary 3x7 matrix:

| Candidate | S1 Noisy TV | S2 Unanimous weak signal | S3 Mixed signal | S4 Missing data | S5 Scale heterogeneity | S6 Calibration decomposability | S7 Boundary-seeking |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| IVW-CDF | base=0.6457, doubled=0.8849 (**FAIL**) | agg=0.2698, max1=0.5818, target=0.8726 (**FAIL**) | mixed=0.5171, allC=0.6761, allA=0.3239 (**PASS**) | missing=0.5300, baseline=0.5261, delta=0.007 (**PASS**) | agg=0.8445, scores=[0.9545, 0.9901, 0.589] (**PASS**) | agg=0.8684, recon=0.8684, dom_share=0.657 (**PASS**) | boundary=0.6344, non_boundary=0.7686 (**PASS**) |
| HTG-Max | base=0.1165, doubled=0.0238 (**PASS**) | agg=0.3033, max1=0.3033, target=0.4549 (**FAIL**) | mixed=0.3795, allC=0.3795, allA=0.1774 (**PASS**) | missing=0.1593, baseline=0.3423, delta=0.535 (**FAIL**) | agg=0.5712, scores=[0.9545, 0.9901, 0.589] (**PASS**) | agg=0.9618, recon=0.9618, dom_share=1.000 (**PASS**) | boundary=0.3022, non_boundary=0.5085 (**PASS**) |
| Fisher-UP | base=0.5639, doubled=0.8227 (**FAIL**) | agg=0.0056, max1=0.5818, target=0.8726 (**FAIL**) | mixed=0.4002, allC=0.7134, allA=0.0454 (**PASS**) | missing=0.0662, baseline=0.3250, delta=0.796 (**FAIL**) | agg=0.9914, scores=[0.9545, 0.9901, 0.589] (**PASS**) | agg=0.9784, recon=0.9784, dom_share=0.568 (**PASS**) | boundary=0.9917, non_boundary=0.9917 (**FAIL**) |

- Pass counts:
  - `IVW-CDF`: 5/7
  - `HTG-Max`: 5/7
  - `Fisher-UP`: 3/7
- No candidate passed all seven scenarios in Session 1.
- Calibration decomposition (S6) is now explicit and reconstructs aggregate in all primary candidates, with dominant component identification:
  - Dominant component for all three was `s6.z.strong`; dominance share: IVW `0.657`, HTG `1.000`, Fisher `0.568`.
- Scale heterogeneity ranking stability held across all three candidates (S5): `s5.bf.1 > s5.z.1 > s5.abs.1`.
- Exploratory `HTG-Max-LSE` remained bounded but failed S6 decomposition reconstruction under current decomposition mapping (`recon=0.9180` vs `agg=0.6263`), so it is retained as exploratory only.

**Implications**

- `HTG-Max` is strongest on Noisy-TV resistance in this session (passes S1 and S7), but currently too brittle under missing/partial uncertainty (S4 fail).
- `IVW-CDF` remains attractive for calibration diagnostics and missing-data robustness, but currently fails the Noisy-TV stressor (S1).
- `Fisher-UP` compounds evidence and is calibratable, but in this fixture set it is the least Noisy-TV resistant and most sensitive to missing uncertainty defaults.
- Since no candidate passed all seven criteria, Session 2 should focus on parameter sensitivity and uncertainty-floor tuning before a recommendation in Session 3.

**Open Threads**

- Session 2 (not executed): parameter sweeps for `HTG alpha/tau/c_floor`, `IVW w_default`, and `Fisher n_ref/r_floor`.
- Session 2 (not executed): uncertainty fallback policy ablations for `NoUncertainty` to reduce S4 degradation in HTG/Fisher.
- Session 2 (not executed): criterion sensitivity check for S2 compounding target under bounded aggregators.
- Session 3 (not executed): recommendation and `AggregateScore` type definition for architecture integration.

## Accumulated Findings

### What We Know

- All three candidates are bounded in practice for Session 1 fixtures: no NaN and no out-of-range scores in the full 3x7 matrix.  
  Evidence: Investigation Log entry `2026-02-22 — WDK#41 Session 1` (`results.json`, boundedness check).
- No candidate satisfies all seven stress scenarios in Session 1.  
  Evidence: same log entry; pass totals `IVW 5/7`, `HTG 5/7`, `Fisher 3/7`.
- `HTG-Max` currently dominates Noisy-TV resistance among tested options (passes S1 and S7), while `IVW-CDF` and `Fisher-UP` inflate or fail to decrease under the S1 value+uncertainty doubling stressor.  
  Evidence: same log entry, S1 and S7 rows in matrix.
- Calibration decomposability is workable for all primary candidates after normalized decomposition weights in IVW (`sum(w_i*u_i) ~= aggregate`).  
  Evidence: same log entry, S6 reconstruction values.
- CDF normalization handles scale heterogeneity stably across disparate units/kinds (`BF`, `Z`, `AbsDiff`) without collapse to 0/1 in this fixture.  
  Evidence: same log entry, S5 row and ranking stability.

### What We Suspect

- A hybrid that combines HTG-style uncertainty gating with non-max compounding may better satisfy both Noisy-TV resistance and weak-signal accumulation than any single candidate tested so far.  
  Evidence basis: HTG passes S1/S7 but fails S2; IVW/Fisher behavior suggests tradeoff in current forms (same log entry).
- Missing-data behavior is likely dominated by fallback uncertainty-floor choices (`c_floor`, `r_floor`, `w_default`) more than by normalization transforms themselves.  
  Evidence basis: S4 deltas diverge strongly by candidate despite shared normalization pipeline (same log entry).

### What We Don't Know

- Whether a tuned parameter region exists where one candidate can pass all seven scenarios simultaneously.
- Which fallback policy for `NoUncertainty` best preserves Noisy-TV resistance without suppressing genuine weak-signal compounding.
- Whether S2’s compounding criterion should remain fixed or be parameterized by expected bounded-aggregator behavior.
- Which candidate/variant should be promoted to a formal `AggregateScore` definition for architecture integration (Session 3 decision).

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| `research/adversarial-reward/prototypes/aggregation-candidates/models.py` | Contract-preserving dataclasses/enums for aggregation prototype inputs and outputs | Complete (Session 1) | Input contract mirrored without mutating schema semantics |
| `research/adversarial-reward/prototypes/aggregation-candidates/normalization.py` | Shared CDF normalization, direction handling, uncertainty extraction, and weight helpers | Complete (Session 1) | Uniform `[0,1]` mapping across heterogeneous divergence kinds; agreement inversion implemented |
| `research/adversarial-reward/prototypes/aggregation-candidates/candidates.py` | C1 IVW-CDF, C2 HTG-Max, C3 Fisher-UP candidate implementations + exploratory HTG-LSE | Complete (Session 1) | Three bounded aggregation formulations with per-component decomposition output |
| `research/adversarial-reward/prototypes/aggregation-candidates/scenarios.py` | Seven adversarial scenario fixtures for stress testing | Complete (Session 1) | Standardized scenario coverage for Noisy-TV, calibration, heterogeneity, and missing-data stressors |
| `research/adversarial-reward/prototypes/aggregation-candidates/evaluate.py` | Matrix runner and artifact generator for candidate-by-scenario evaluation | Complete (Session 1) | 3x7 matrix, pass/fail adjudication, decomposition capture, exploratory variant execution |
| `research/adversarial-reward/prototypes/aggregation-candidates/results.json` | Raw machine-readable Session 1 outputs | Complete (Session 1) | Full per-cell scores, pass/fail, warnings/skips, and decompositions |
| `research/adversarial-reward/prototypes/aggregation-candidates/results.md` | Human-readable Session 1 matrix summary | Complete (Session 1) | Compact 3x7 evidence table for research log integration |

## Next Steps

0. **WDK#41 Step 0 (inserted): Session 2 parameter sweeps + fallback-policy ablations** — Run sensitivity analysis over `IVW(w_default)`, `HTG(alpha,tau,c_floor)`, and `Fisher(n_ref,r_floor)`; evaluate missing-uncertainty policies and identify any configuration that can satisfy all seven scenarios. Scope: 1-2 sessions.

1. **Survey formalizations in active learning and Bayesian experimental design** — Review how information gain is formalized in discriminative active learning, Bayesian optimization (expected improvement, knowledge gradient), and optimal experimental design. Identify which formalizations handle bounded search spaces. Scope: 2-3 sessions.

2. **Characterize the conservatism-vs-boundary-seeking failure spectrum** — Formally describe the two failure modes as functions of reward function properties. Under what conditions does a given formalization collapse to conservatism? To boundary-seeking? Identify the design parameters that control this tradeoff. Scope: 1-2 sessions.

3. **Analyze Noisy TV in DSL simulation contexts** — The Noisy TV problem is well-studied in RL. Characterize how it manifests specifically in DSL simulation environments: what sources of irreducible stochasticity exist in OpenMM/GROMACS/VASP? How does the DSL's deterministic subspace constraint interact with noise sources? Scope: 1-2 sessions.

4. **Investigate calibration loop constraints on functional form** — The calibration feedback (ARCHITECTURE.md 5.4) compares predicted vs. actual surprise. What constraints does this calibration mechanism place on the reward function's functional form? Must the function be decomposable in specific ways for calibration to be meaningful? Scope: 1-2 sessions.

5. **Draft candidate reward function specifications** — Propose 2-3 candidate formalizations with explicit tradeoff profiles across the failure spectrum. Each should be evaluable against the calibration loop requirements. Scope: 2-3 sessions.
