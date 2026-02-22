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

### 2026-02-21 -- WDK#41 Session 3: HTG-Gated Fisher Product Hybrid (n_terms=1)

**Scope**

- Implement one cross-family hybrid candidate that composes HTG-style per-component confidence gating with Fisher product combination (`n_terms=1`) in the existing aggregation prototype.
- Preserve backward compatibility for existing candidates (`IVW-CDF`, `HTG-Max`, `Fisher-UP`) under the default Session 2 evaluator path.
- Validate all seven scenario gates with special focus on S2 compounding, S4 missing-data stability, S6 decomposition reconstruction, and boundedness/finite outputs.

**Method**

- Updated `research/adversarial-reward/prototypes/aggregation-candidates/candidates.py`:
  - Added `HybridConfig(alpha=1.5, tau=5.0, c_floor=0.1, c_missing=0.7, p_eps=1e-12, eps=1e-12)`.
  - Added `aggregate_hybrid()` with this pipeline per component:
    - normalize via `normalize_component(...)` with normalization-level SE dampening left OFF.
    - compute precision via `gate_precision(component, eps)`.
    - confidence rule: `max(c_floor, sigmoid(...))` when precision exists, else `c_missing`.
    - gated score to p-value via `p=max(p_eps,1-gated)`, evidence `-2*log(p)`.
    - aggregate with `chi_square_cdf_even_df(total_log_evidence, n_terms=1)` (product method).
    - exact decomposition weights from `log_evidence_i * (aggregate / sum(log_evidence_j * score_j))` when denominator > `eps`.
  - Registered `"Hybrid"` in `get_candidate_registry(...)` with `hybrid_cfg` parameter.
- Updated `research/adversarial-reward/prototypes/aggregation-candidates/evaluate.py`:
  - Added `HybridConfig` import.
  - Passed `hybrid_cfg=HybridConfig(normalization=normalization)` into `get_candidate_registry(...)`.
- Executed `python evaluate.py` in `research/adversarial-reward/prototypes/aggregation-candidates/`, then read `results.json` for exact metric checks.

**Findings**

- Backward compatibility held for existing candidates:
  - `IVW-CDF`: `5/7`
  - `HTG-Max`: `5/7`
  - `Fisher-UP`: `3/7`
- Hybrid passed all seven scenarios (`7/7`) in the default harness.
- S2 sensitivity numbers (Hybrid):
  - aggregate = `0.9234566367020085`
  - max_single = `0.5738586978538172`
  - ratio = `1.6092056113389201`
  - margin = `(aggregate / (1.5 * max_single)) - 1.0 = +7.280374%`
- S2 threshold-driver detail:
  - `s2.custom.1` is the highest single contributor by normalized score (`0.5817593768418363`) and highest single aggregate (`0.5738586978538172` after confidence gating).
  - Other S2 normalized component scores remain substantially lower (max non-custom = `0.382252125230751`).
- S4 missing-data robustness (Hybrid):
  - relative delta = `0.0719926034986539` (passes `<= 0.20`).
- S6 decomposition exactness (Hybrid):
  - reconstruction error = `1.1102230246251565e-16` (passes `<= 1e-8`).
- Boundedness and finiteness:
  - all Hybrid scenario aggregates and comparator values remained finite and within `[0,1]`.

**Implications**

- The cross-family hybrid resolves the Session 1/2 tradeoff in this fixture set: HTG-style front-end gating suppresses noisy components while Fisher product back-end compounds weak concordant evidence strongly enough to clear S2.
- Keeping normalization-level SE dampening off and avoiding Fisher reliability exponentiation did not prevent robustness; confidence gating alone was sufficient in this run.
- No S2 fallback sweep was required because the default hybrid configuration exceeded the 1.5x threshold with positive margin.

**Open Threads**

- Verify whether `7/7` is stable under broader perturbations (fixture resampling, stronger correlation structure, and alternative uncertainty-missingness patterns).
- Decide whether Session 4 should stress-test this hybrid against calibration-pattern criteria used in Session 2 stretch analyses.
- Determine whether the architecture-facing `AggregateScore` recommendation should now target this hybrid directly or require one additional robustness session.

### 2026-02-22 — WDK#41 Session 2: Structural Fixes + Two-Stage Sweep + Calibration + Correlation Robustness

**Scope**

- Implement Session 2 structural knobs in normalization and candidate aggregators while preserving Session 1 default behavior.
- Run a two-stage sweep using all seven scenarios for every candidate-config combination.
- Execute stretch analyses: deterministic 50-cycle calibration simulation and Fisher-UP correlation robustness with Brown-style correction.
- Update research artifacts and verify backward compatibility (`python evaluate.py`) remains exactly Session 1 with defaults.

**Method**

- Structural changes in `research/adversarial-reward/prototypes/aggregation-candidates/`:
  - `normalization.py`: added optional SE dampening (`se_dampen_enabled`, `se_dampen_k`, `se_dampen_x0`) applied at final score stage using raw `component.value / standard_error`.
  - `candidates.py`:
    - `IVW-CDF`: multiplicity bonus (`multiplicity_bonus_enabled`, threshold, scale).
    - `HTG-Max`: `soft_sum` mode with `soft_sum_boost=2.0` (configurable, default unchanged).
    - `Fisher-UP`: optional SE-aware reliability factor (`se_reliability_*`).
  - `candidates.py`: stabilized `chi_square_cdf_even_df` with recurrence-series evaluation to avoid overflow under large term counts.
- New prototype runners:
  - `sweep.py`: Stage 1 normalization sweep (81 normalization configs x 3 candidates = 243 candidate-configs) + Stage 2 candidate sweeps with best Stage 1 normalization (480 candidate-configs including Fisher isolation).
  - `calibration_sim.py`: deterministic 50-cycle patterns A/B/C with stdlib Spearman and Pearson implementations.
  - `correlation_test.py`: S2-like correlated weak signals at rho `{0.0,0.3,0.5,0.7,0.9}`, Cholesky generation, Brown-style corrected df (capped terms at 1000).
- Compatibility and execution checks:
  - `python evaluate.py` (before and after Session 2 changes) confirmed exact Session 1 matrix.
  - `python sweep.py`, `python calibration_sim.py`, and `python correlation_test.py` completed and wrote artifacts.

**Findings**

- Backward compatibility held exactly with default flags disabled:
  - `IVW-CDF`: 5/7 (S1 FAIL, S2 FAIL)
  - `HTG-Max`: 5/7 (S2 FAIL, S4 FAIL)
  - `Fisher-UP`: 3/7 (S1 FAIL, S2 FAIL, S4 FAIL, S7 FAIL)
- Stage 1 normalization sweep selected `N061` (`abs_diff_k=2000`, `abs_diff_x0=5e-4`, `se_dampen_k=8`, `se_dampen_x0=1`) with `10/21` passes (top by pass-count then avg-pass-score).
- Stage 2 best results (no 7/7 found):
  - `IVW-CDF`: best `2/7` (fails S1,S2,S4,S5,S6) despite multiplicity bonus sweep.
  - `HTG-Max`: best `5/7` (fails S5,S6), strongest overall in Session 2 sweep.
  - `Fisher-UP` main sweep (`se_dampen=True`): best `4/7` (fails S1,S2,S5).
  - `Fisher-UP` isolation (`se_dampen=False`, SE-reliability on): best `5/7` (fails S1,S2), indicating overlap/tension between normalization-level dampening and Fisher reliability scaling.
- S2 sensitivity frontier on 6/7-with-only-S2-fail configs was empty for all three candidates (no qualifying configs), so no feasible multiplier frontier from 1.0 to 2.0 could be established under that criterion subset.
- Calibration simulation (Pattern A/B/C) with best configs:
  - IVW: failed all three patterns (`rho=-0.8728`, `step_ratio=2.9533`, `r=0.0000`).
  - HTG: passed A and C, failed B (`step_ratio=1.0036`).
  - Fisher: passed A and C, failed B on smoothness (step jump too sharp; `max_delta=0.9996`).
- Fisher correlation robustness results:
  - Inflation ratios were near 1.0 across all rhos (`1.0000`, `1.0000`, `1.0000`, `1.0025`, `1.0000` for rho `0.0..0.9`).
  - No flag at rho=0.5 (`inflation_ratio > 1.5` condition not met).
  - In this setup both corrected and uncorrected aggregates were at floor-level (~`1e-12`), limiting interpretability of inflation magnitude.

**Implications**

- Session 2 did not produce a 7/7 candidate within the constrained single-candidate families.
- HTG remains the best single-family performer in overall pass count, but improved S2 compounding still trades off against other scenario gates.
- Fisher behaves better on missing-data/boundary than Session 1 under isolation, but Noisy-TV (S1) and weak-signal compounding (S2) remain unresolved.
- Correlation-inflation risk was not observed in the tested S2-like regime, but this result is confounded by aggregate floor saturation.

**Open Threads**

- Session 3 should focus on cross-family designs (explicitly out of scope for Session 2) because single-family tuning did not reach 7/7.
- Revisit S2 fixture regime for Fisher correlation stress where aggregates are not floor-saturated; otherwise inflation diagnostics are weak.
- Investigate why Session 2 normalization winner degrades IVW/HTG S5-S6 behavior despite helping S1 suppression.

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

- A cross-family hybrid candidate (`Hybrid`) now satisfies all seven stress scenarios in the default harness while preserving baseline behavior of prior candidates (`IVW 5/7`, `HTG 5/7`, `Fisher 3/7`).  
  Evidence: Investigation Log entry `2026-02-21 -- WDK#41 Session 3` (`research/adversarial-reward/prototypes/aggregation-candidates/results.json`).
- Hybrid S2 compounding clears the fixed threshold without criterion relaxation: aggregate `0.9234566367020085`, max_single `0.5738586978538172`, ratio `1.6092056113389201`, margin `+7.280374%`.  
  Evidence: same Session 3 log entry (`results.json` S2 raw scores).
- Hybrid meets S4 and S6 integrity gates with margin: S4 relative delta `0.0719926034986539` (`<=0.20`), S6 reconstruction error `1.1102230246251565e-16` (`<=1e-8`).  
  Evidence: same Session 3 log entry (`results.json` S4/S6 raw scores and decomposition).
- All three candidates are bounded in practice for Session 1 fixtures: no NaN and no out-of-range scores in the full 3x7 matrix.  
  Evidence: Investigation Log entry `2026-02-22 — WDK#41 Session 1` (`results.json`, boundedness check).
- Session 2 structural flags are backward-compatible: with defaults, `evaluate.py` exactly reproduces Session 1 pass/fail outputs (`5/7`, `5/7`, `3/7`).  
  Evidence: Investigation Log entry `2026-02-22 — WDK#41 Session 2` (`evaluate.py` rerun before/after modifications).
- No candidate satisfies all seven stress scenarios after Session 2 sweeps; best pass counts are `IVW 2/7`, `HTG 5/7`, `Fisher 4/7` in main Stage 2 runs, with Fisher isolation at `5/7`.  
  Evidence: same Session 2 log entry (`sweep_summary.md`, `sweep_results.json`).
- `HTG-Max` remains the strongest single-family performer by aggregate pass count in Session 2 sweeps, but still fails two gates in best settings (S5,S6).  
  Evidence: same Session 2 log entry, Stage 2 top-table.
- Calibration decomposability is workable for all primary candidates after normalized decomposition weights in IVW (`sum(w_i*u_i) ~= aggregate`).  
  Evidence: same log entry, S6 reconstruction values.
- S2 criterion-sensitivity frontier (for configs at 6/7 failing only S2) yielded no qualifying configs for any candidate in Session 2.  
  Evidence: same Session 2 log entry (`sweep_summary.md`, S2 frontier table).
- Calibration pattern B is unstable across all best-per-candidate configs (either insufficient step response or excessive jump/smoothness failure).  
  Evidence: same Session 2 log entry (`calibration_summary.md`).
- Fisher correlation inflation flag did not trigger at rho=0.5 (`inflation_ratio` did not exceed 1.5), but the test aggregates were floor-saturated.  
  Evidence: same Session 2 log entry (`correlation_results.json`).

### What We Suspect

- Joint use of normalization-level SE dampening and Fisher SE-reliability scaling may be over-attenuating evidence in some regimes.  
  Evidence basis: Fisher isolation (`se_dampen=False`) improved from 4/7 to 5/7 vs. main sweep (Session 2).
- The S2 pass margin may be sensitive to fixture-level score concentration in `s2.custom.1`, because that component sets the max-single threshold and dominates the target denominator.  
  Evidence basis: Session 3 S2 decomposition (`s2.custom.1` score `0.5818`; max non-custom `0.3823`).

### What We Don't Know

- Whether the hybrid remains `7/7` outside the current fixed fixture set (especially under correlated weak signals and non-floor-saturated Fisher regimes).
- Whether any non-hybrid single-family configuration can match hybrid performance under current criteria.
- Whether Fisher correlation inflation remains near-neutral when evaluated in a non-floor-saturated weak-signal regime.
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
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep.py` | Session 2 two-stage parameter sweep driver (normalization + candidate sweeps + S2 sensitivity) | Complete (Session 2) | Exhaustive scenario evaluation over 723 candidate-configs (243 Stage 1 + 480 Stage 2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep_results.json` | Full Session 2 sweep records for all evaluated configs and scenarios | Complete (Session 2) | Machine-readable pass/fail matrices, raw scores, and config metadata |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep_summary.md` | Human-readable Session 2 sweep rankings and frontier summary | Complete (Session 2) | Top-5 per candidate, no-7/7 finding, Fisher isolation comparison |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_sim.py` | Deterministic 50-cycle calibration stress simulation for patterns A/B/C | Complete (Session 2 Stretch) | Pattern metrics (Spearman/step-ratio/Pearson) + smoothness diagnostics per candidate |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_results.json` | Raw cycle-level calibration outputs | Complete (Session 2 Stretch) | Per-cycle scores and pass/fail stats for each pattern/candidate |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_summary.md` | Human-readable calibration summary | Complete (Session 2 Stretch) | Pattern-by-candidate pass matrix and smoothness outcomes |
| `research/adversarial-reward/prototypes/aggregation-candidates/correlation_test.py` | Fisher-UP correlation robustness probe with Cholesky generation + Brown-style correction | Complete (Session 2 Stretch) | Inflation-ratio diagnostics across rho levels with overflow-safe corrected CDF terms |
| `research/adversarial-reward/prototypes/aggregation-candidates/correlation_results.json` | Correlation robustness outputs | Complete (Session 2 Stretch) | Inflation ratios at rho `{0.0,0.3,0.5,0.7,0.9}` and rho=0.5 flag status |

## Next Steps

0. **WDK#41 Step 0 (updated): Session 3 bridge from sweep outcomes** — Design and evaluate cross-family/hybrid candidates explicitly targeting the unresolved `(S1,S2)` and `(S5,S6)` tradeoffs identified in Session 2. Scope: 1-2 sessions.

1. **Survey formalizations in active learning and Bayesian experimental design** — Review how information gain is formalized in discriminative active learning, Bayesian optimization (expected improvement, knowledge gradient), and optimal experimental design. Identify which formalizations handle bounded search spaces. Scope: 2-3 sessions.

2. **Characterize the conservatism-vs-boundary-seeking failure spectrum** — Formally describe the two failure modes as functions of reward function properties. Under what conditions does a given formalization collapse to conservatism? To boundary-seeking? Identify the design parameters that control this tradeoff. Scope: 1-2 sessions.

3. **Analyze Noisy TV in DSL simulation contexts** — The Noisy TV problem is well-studied in RL. Characterize how it manifests specifically in DSL simulation environments: what sources of irreducible stochasticity exist in OpenMM/GROMACS/VASP? How does the DSL's deterministic subspace constraint interact with noise sources? Scope: 1-2 sessions.

4. **Investigate calibration loop constraints on functional form** — The calibration feedback (ARCHITECTURE.md 5.4) compares predicted vs. actual surprise. What constraints does this calibration mechanism place on the reward function's functional form? Must the function be decomposable in specific ways for calibration to be meaningful? Scope: 1-2 sessions.

5. **Draft candidate reward function specifications** — Propose 2-3 candidate formalizations with explicit tradeoff profiles across the failure spectrum. Each should be evaluable against the calibration loop requirements. Scope: 2-3 sessions.
