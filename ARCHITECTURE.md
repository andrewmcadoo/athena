# ARCHITECTURE.md: ATHENA - (Adversarial Testing of Hypotheses through Evolutionary and Normative Analysis)

## 1. Abstract

Contemporary AI co-scientist architectures treat scientific discovery as a stochastic generation problem, optimizing for verification metrics through tournament evolution, tree search, or genetic algorithms. When experiments fail, these systems register a scalar penalty and generate alternatives -- they cannot determine whether the failure reflects a flawed theory, a flawed experiment, or a broken implementation. This conflation structurally prevents causal learning from negative results.

ATHENA is a multi-agent architecture that inverts this paradigm. Rather than maximizing verification metrics, ATHENA maximizes the causal information extracted from each experimental outcome -- particularly failures. Its architecture implements a computationally tractable form of Lakatosian fault isolation: when an experiment fails, a structured diagnostic pipeline classifies the failure as theoretical, methodological, or implementational before any update propagates to the hypothesis. An adversarial experiment designer then selects the next experiment to maximally stress-test surviving hypotheses.

The architecture operates under three non-negotiable constraints: it requires domain-specific language environments with API-separated theory and implementation layers; it requires warm-started causal priors rather than zero-knowledge bootstrapping; and its adversarial design is bounded to deterministic, domain-valid subspaces. These constraints make causal fault isolation tractable at the cost of general applicability.

---

## 2. Motivation

### 2.1 The Structural Flaw in Generation-First Discovery

The prevailing approach to AI-driven scientific discovery scales generation: produce many hypotheses, test them against metrics, keep the winners, mutate the losers. This approach inherits its epistemology from language model scaling -- the assumption that covering more of the search space will eventually yield the right answer.

This assumption holds in low-cost, high-dimensional search problems (hyperparameter sweeps, combinatorial chemistry screens). It fails in domains where executing a single experiment requires thousands of GPU-hours, expensive physical synthesis, or months of simulation time. In these domains, the question is not "can we generate more hypotheses?" but "can we extract more information from the experiments we have already run?"

Generation-first architectures are structurally unable to answer this question because they treat experimental environments as black-box oracles returning scalar rewards. When an experiment produces a poor result, the system faces a three-way ambiguity it cannot resolve:

1. The hypothesis may be theoretically wrong (the proposed mechanism does not exist).
2. The experiment may be methodologically incapable of testing the hypothesis (wrong metric, uncontrolled confounders, insufficient sampling).
3. The implementation may contain a localized engineering error (tensor mismatch, data loader bug, numerical overflow).

A generation-first system collapses all three cases into a single low-reward signal. The hypothesis is penalized or discarded regardless of the failure's actual cause. This means valid theories are abandoned due to broken code, and invalid theories survive when their implementations happen to produce high metrics through data leakage or benchmark selection.

### 2.2 Why Architecture Determines Epistemology

This is not a prompting problem or a logging problem. The inability to perform causal fault isolation is a consequence of architectural choices: scalar reward functions, tournament selection mechanisms, and genetic mutation operators do not preserve the causal structure of failure. Adding "reflection" steps -- prompting an LLM to reason about why a script failed -- does not change the underlying architecture. Empirical analysis of such reflection loops shows they are overwhelmingly constrained to syntactic error correction rather than causal theory analysis.

A system that can causally isolate failures requires a fundamentally different architecture: one where the objective function values the precise documentation of failure as much as success, where the theory layer is structurally separated from the implementation layer, and where experiments are designed to maximally challenge hypotheses rather than confirm them.

### 2.3 The Falsification-First Thesis

ATHENA's thesis is conditional and specific: **in scientific domains characterized by asymmetric verification costs -- where executing an experiment requires significantly more resources than generating a hypothesis -- establishing the exact causal locus of an experimental failure accelerates convergence on the true objective function faster than stochastic hypothesis generation.**

The mechanism is causal fault assignment. Each failure is parsed through a structured diagnostic pipeline that definitively assigns it to theory, methodology, or implementation. Knowing *why* the last hypothesis failed provides directed constraint in the discrete space of scientific theories, rather than the undirected random walk that scalar penalties induce.

This thesis has an honest boundary condition: when experiment execution is cheap and the search space is unstructured, brute-force generation achieves faster raw throughput. ATHENA's advantage is conditional on high verification costs persisting.

---

## 3. Architectural Constraints

ATHENA operates under three non-negotiable design constraints. Each is load-bearing -- removing any one makes the architecture either computationally intractable or epistemically unsound.

### 3.1 DSL-Only Environments

ATHENA operates exclusively within structured Domain-Specific Language (DSL) environments where the theoretical specification (equations, parameters, causal claims) and the computational implementation (data loading, memory management, numerical execution) are separated by the framework's API. Examples include molecular dynamics frameworks (OpenMM, GROMACS), climate modeling systems (CESM), and materials science simulators (VASP, Quantum ESPRESSO).

**Why necessary.** Lakatosian fault isolation requires the system to deterministically distinguish theory-layer operations from implementation-layer operations when auditing a failure. In arbitrary Python, this distinction is not structurally enforced -- theory and implementation are interleaved in ways that make automated separation intractable. DSL environments enforce the separation by design.

**What it enables.** Deterministic auditing of the implementation layer. The system can verify that data pipelines, numerical operations, and resource allocation executed correctly *independently* of whether the theoretical predictions were accurate.

**What it forecloses.** ATHENA cannot operate as a general-purpose autonomous coding agent. It is inapplicable to domains that lack mature, API-separated simulation frameworks. This is the largest scope constraint on the architecture.

### 3.2 Warm-Started Causal Priors

ATHENA does not attempt zero-knowledge causal discovery. The initial causal graph is generated from LLM-produced structural priors or domain-expert seed knowledge, then refined through a dedicated exploration phase before the system enters the falsification loop.

**Why necessary.** Causal discovery algorithms fail in high-dimensional spaces without structural priors. Fault isolation requires a causal model of the domain to classify failures -- without one, the system cannot determine whether a result contradicts the theory or merely reflects an unmodeled confounder. Bootstrapping this model from zero is an unsolved problem.

**What it enables.** The system enters the falsification loop with a usable (if imperfect) causal graph, making fault isolation functional from the first cycle. The exploration phase corrects the worst errors in the initial priors before they propagate.

**What it forecloses.** ATHENA inherits the biases of its initial priors. If the priors are fundamentally misspecified in ways the exploration phase cannot detect, the system will reason from a corrupted causal model. This is the most severe theoretical threat to the architecture (see Section 8.1).

### 3.3 Bounded Adversarial Design

The adversarial experiment designer maximizes epistemic information gain strictly within deterministic, domain-valid subspaces. It cannot propose experiments with physically impossible parameters, nonsensical boundary conditions, or configurations that introduce unlearnable stochastic noise.

**Why necessary.** An unconstrained surprise-maximizing agent suffers from the Noisy TV problem: it becomes fixated on stochastic, unlearnable noise because such noise consistently yields high prediction error. Bounding the adversary to deterministic, physically valid subspaces ensures that high surprise corresponds to genuine theoretical information rather than irreducible noise.

**What it enables.** The adversary can aggressively stress-test hypotheses without degenerating into noise-seeking. Every proposed experiment is guaranteed to be within the domain's valid physics, making results interpretable.

**What it forecloses.** The adversary cannot explore beyond the domain constraints, even when the most informative experiment might lie at the boundary of known physics. This trades comprehensiveness for reliability.

---

## 4. Component Architecture

ATHENA consists of eleven components organized into five functional groups: graph infrastructure, exploration, the falsification loop, the analysis pipeline, and orchestration. The DSL Environment Interface serves as the system boundary.

### 4.1 System Boundary

**DSL Environment Interface.** Mediates all interaction between ATHENA and the external simulation framework. Receives experiment specifications from internal components; produces validated, framework-conformant configurations and domain constraint specifications (valid parameter ranges, available observables, deterministic subspace boundaries). This is an engineering component -- its implementation varies per target DSL (Open Question #2 in the companion vision document), but its interface contract is fixed. All experiment execution and domain constraint queries pass through this boundary.

### 4.2 Graph Infrastructure

**Structural Prior Generator.** Produces the initial causal DAG from LLM-generated structural priors or domain-specific seed knowledge. Receives a domain specification (target DSL, field of study, available literature). Produces a confidence-weighted DAG representing hypothesized causal relationships. This component has no internal dependencies -- it is the bootstrap entry point for the entire system. Its output quality is load-bearing: the severity gap between this component's importance and its design maturity is the widest of any component in the architecture (see Section 8.1).

**Causal Graph Manager.** The central data structure of the architecture. Maintains, queries, and updates the directed acyclic graph representing known and hypothesized causal relationships. Receives the initial DAG from the Structural Prior Generator; edge-level update directives from the Lakatosian Fault Isolator (prune, confirm, reweight); refinement signals from the Epistemic Explorer; and confidence updates from the Bayesian Surprise Evaluator. Produces the current DAG state queryable by four other components (Hypothesis Generator, Adversarial Experiment Designer, Lakatosian Fault Isolator, Mode Controller); graph stability metrics; and versioned snapshots for audit and mode transition decisions. This component is the single most critical artifact in the system -- if the graph is corrupted, every downstream component degrades (see Section 8.6).

### 4.3 Exploration Subsystem

**Epistemic Explorer.** Runs low-cost system-identification experiments to prune and refine the initial causal graph before the system enters the falsification loop. Queries the current DAG from the Causal Graph Manager, identifies edges with highest uncertainty, and designs targeted interventional probes to resolve specific edge uncertainties. Produces refined DAG edges and exploration convergence metrics that the Mode Controller uses to trigger the exploration-to-falsification transition. The convergence criteria for this component are an open research question -- defining when the graph is "good enough" for fault isolation is domain-dependent and currently lacks formal specification.

### 4.4 Falsification Loop

**Hypothesis Generator.** Generates falsifiable scientific hypotheses constrained to the valid subspace defined by the current causal DAG. Receives the current DAG and accumulated failure history (which hypothesis-space regions have been eliminated). Produces candidate hypotheses with explicit causal claims and testable predictions. The architectural novelty is not the generation mechanism (LLM-based hypothesis generation exists in competing systems) but the constraint: generation is bounded by the DAG rather than free-form.

**Adversarial Experiment Designer.** Designs experiments that maximize expected epistemic information gain within domain-valid, deterministic subspaces. Receives the candidate hypothesis, the current DAG, predicted information gain estimates from the Bayesian Surprise Evaluator, and domain validity constraints from the DSL Environment Interface. Produces a fully specified experiment configuration (parameters, data distribution, boundary conditions, expected observables) engineered to maximally challenge the hypothesis. The predicted-gain estimation requires forward-simulating candidate experiments against the current DAG, a computationally non-trivial sub-responsibility that should not be understated. The aggregation contract for "epistemic information gain within a bounded subspace" is now locked in Section 4.4.1.

#### 4.4.1 AggregateScore Contract (Locked)

The reward-formalization research dependency for adversarial aggregation is now specified by a locked contract: `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md` (Version 1.0, 2026-02-22, bead `athena-6ax`). Downstream implementation must treat this as normative.

**Per-component and aggregate pipeline (locked):**

```
Input: Vec<MetricComponent>

For each component_i:
  1. score_i        = normalize_component(component_i, normalization_config)
  2. precision_i    = log1p(sample_size / (standard_error^2 + eps)) when uncertainty exists
  3. confidence_i   = max(c_floor, sigmoid(precision_i, alpha, tau)) when precision exists
                      c_missing when uncertainty is absent
  4. gated_i        = score_i * confidence_i
  5. p_i            = max(p_eps, 1.0 - gated_i)
  6. evidence_i     = -2 * log(p_i)

Aggregate:
  7. T              = sum(evidence_i)
  8. aggregate      = chi_square_cdf(T, df=2) = 1 - exp(-T/2)   // n_terms=1

Decomposition:
  9. weight_i       = evidence_i * (aggregate / sum(evidence_j * score_j))
 10. contribution_i = weight_i * score_i
```

**BF normalization branch (locked default):**

- `DivergenceKind.BayesFactor` uses `bf_norm_log_scaled(bf, c) = log1p(bf) / (log1p(bf) + c)`.
- `c = 0.083647`, calibrated from `bf_max_target=10000`.
- Other normalization families (`ZScore`, `KLDivergence`, `AbsoluteDifference`, `EffectSize`, `Custom`) remain unchanged from the Session 1 baseline.

**Locked parameters:**

| Parameter | Value |
| :--- | ---: |
| `alpha` | 1.5 |
| `tau` | 5.0 |
| `c_floor` | 0.1 |
| `c_missing` | 0.7 |
| `p_eps` | 1e-12 |
| `eps` | 1e-12 |
| `n_terms` | 1 |
| `bf_norm_c` | 0.083647 |
| `bf_max_target` | 10000 |
| `clip_eps` | 1e-12 |
| `absolute_difference_sigmoid.k` | 1200.0 |
| `absolute_difference_sigmoid.x0` | 7e-4 |

**Guardrail requirement (locked):**

- `GR-S2-CUSTOM-SIGMOID-X0-NONNEG` applies to all `NormalizationConfig.custom_sigmoids`.
- Constraint: `x0 >= 0`.
- Enforcement point: config construction/validation (reject invalid config with explicit error; no silent clamping).
- Full guardrail specification: `research/adversarial-reward/prototypes/aggregation-candidates/guardrail_spec.md`.

**Output contract (locked):**

```
AggregateResult:
  candidate:       "Hybrid"
  aggregate_score: float in (0,1)
  contributions:   Vec<ComponentContribution>
                   // decomposition invariant: sum(contribution_i) = aggregate_score
  skipped:         Vec<method_ref>
  warnings:        Vec<str>
```

`n_terms=1` is intentional (single-term chi-square, `df=2`) and is not a placeholder. This is required to preserve calibration behavior and avoid over-aggregation.

**Experiment Executor.** Executes experiment specifications within the DSL environment and captures complete trace logs, execution pathways, and result data. Receives experiment specifications from the Adversarial Experiment Designer (in falsification mode) or the Epistemic Explorer (in exploration mode). Produces raw experimental results, structured trace logs, and execution metadata. The primary novel requirement is completeness of trace capture -- the logs must be sufficient for downstream causal analysis.

### 4.5 Analysis Pipeline

**Trace Semantics Engine.** Translates raw DSL trace logs into a structured semantic intermediate representation (IR) suitable for causal fault analysis. Receives raw trace logs from the Experiment Executor. Produces a structured semantic failure/success representation that maps theory-layer operations to implementation-layer events. This is a critical research component: general root-cause analysis achieves approximately 21% accuracy on unstructured execution traces, though accuracy improves substantially within constrained DSL environments. The IR design itself is unsolved. Classical verification tools (SAT solvers, formal provers, static analyzers) are well-suited to DSL environments and represent a known engineering path for augmenting this component.

**Lakatosian Fault Isolator (LFI).** The primary architectural differentiator. Classifies each experimental failure into exactly one of three categories -- theoretical falsification, methodological falsification, or implementation artifact -- through a sequential, outside-in audit of the experimental pipeline. Receives the structured IR from the Trace Semantics Engine, the current DAG from the Causal Graph Manager, the experiment specification, and the hypothesis under test. Produces a fault classification with causal attribution, a specific graph update directive, and a documented failure record. The three-way classification within a causal framework is architecturally novel; the individual audit techniques draw on existing work in constraint satisfaction and formal verification.

**Bayesian Surprise Evaluator.** Computes the KL divergence between prior beliefs (the current causal model) and posterior observations to quantify the epistemic value of experimental outcomes. Serves a dual role: post-experiment, it computes actual surprise magnitude from results; pre-experiment, it estimates predicted information gain for candidate experiments to guide the Adversarial Experiment Designer's selection. The pre-experiment role requires forward-simulating candidates against the DAG, which is the more computationally demanding sub-responsibility. Receives experimental results from the Executor and prior DAG state from the Causal Graph Manager. Produces surprise magnitudes, predicted gain estimates, and convergence trend metrics. Also receives a surprise invalidation signal from the LFI: when a failure is classified as an implementation artifact, the surprise value computed from that experiment's results is marked unreliable and excluded from trend data (since the hypothesis was never actually tested).

### 4.6 Orchestration

**Mode Controller.** Orchestrates the system's operating phase (exploration vs. falsification), manages loop progression, enforces the computational budget, and triggers human escalation. Receives convergence metrics from the Epistemic Explorer, graph stability metrics from the Causal Graph Manager, surprise trends from the Bayesian Surprise Evaluator, and fault classification patterns from the LFI. Produces phase transition signals, human escalation requests with context, and budget enforcement decisions. The transition criteria themselves require definition and are detailed in Section 6.

### 4.7 Component Dependency Summary

The dependency structure has three important properties. First, the Causal Graph Manager is a hub: it receives updates from four components and is queried by four others. Second, the analysis pipeline is strictly sequential: Executor -> Trace Semantics Engine -> LFI, with the Bayesian Surprise Evaluator running in parallel but subject to invalidation by the LFI. Third, the Structural Prior Generator has zero internal dependencies but the highest downstream impact -- its output quality propagates through every subsequent component.

```
                    Structural Prior Generator
                              |
                              v
                     Causal Graph Manager  <---+---+---+
                    /    |    |    \            |   |   |
                   v     v    v     v           |   |   |
              Hyp.Gen  AED  LFI  Mode.Ctrl     |   |   |
                 |       |                      |   |   |
                 v       v                      |   |   |
              Adv.Exp.Designer                  |   |   |
                      |                         |   |   |
    DSL Env. ---------+                         |   |   |
    Interface         |                         |   |   |
                      v                         |   |   |
              Experiment Executor               |   |   |
                /            \                  |   |   |
               v              v                 |   |   |
    Trace Semantics Eng.   Bayes. Surprise -----+   |   |
               |              ^  (invalidation)     |   |
               v              |                     |   |
              LFI  -----------+---------------------+   |
               |                                        |
               +----------------------------------------+
                     (graph update directives)

    Epistemic Explorer -----> Causal Graph Manager
         |                         ^
         +--- (convergence) -----> Mode Controller
```

---

## 5. Information Flow

### 5.1 The Main Falsification Loop

The loop proceeds through a fixed sequence with two branching outcomes (failure vs. corroboration) and three sub-branches within failure.

**Hypothesis generation.** The Hypothesis Generator queries the Causal Graph Manager for the current DAG and the accumulated failure history. It produces a candidate hypothesis constrained to the remaining valid subspace -- regions of the DAG that have not been eliminated by prior theoretical falsifications.

**Adversarial experiment design.** The Adversarial Experiment Designer receives the hypothesis and queries three sources: the Causal Graph Manager for the DAG structure, the DSL Environment Interface for domain validity constraints, and the Bayesian Surprise Evaluator for predicted information gain across candidate experiment configurations. The predicted-gain query is computationally expensive -- it forward-simulates candidate experiments against the DAG to estimate which configuration would produce the largest expected KL divergence between prior and posterior beliefs. The Designer selects the experiment that maximizes predicted surprise within the deterministic, domain-valid subspace. The DSL Environment Interface enforces the bounded adversarial constraint: if the Designer proposes an experiment outside the valid subspace, the Interface rejects it and the Designer must reformulate. This is the architectural mechanism that prevents Noisy TV degeneration.

**Execution.** The Experiment Executor submits the specification to the DSL framework and captures complete trace logs, raw results, and execution metadata.

**Dual-path analysis.** Two analyses proceed from the results. The Bayesian Surprise Evaluator receives the raw results and prior DAG state, computing the actual KL divergence (the magnitude of belief shift). In parallel, the Trace Semantics Engine translates the trace logs into the structured semantic IR and passes it to the LFI. An important sequencing constraint applies: the surprise value is provisionally computed in parallel with trace analysis, but is subject to retroactive invalidation. If the LFI classifies the failure as an implementation artifact, the surprise value is marked unreliable and excluded from trend data, since the experimental results do not reflect a genuine test of the hypothesis.

**Fault isolation.** The LFI receives the semantic IR, current DAG, experiment specification, and hypothesis. It executes a three-stage sequential audit (detailed in Section 5.3). The output is a fault classification with a specific graph update directive.

**Branching on failure classification:**

*Implementation Artifact.* The hypothesis is untouched. The implementation error is documented. The corrected experiment specification loops back to the Executor. The graph is not updated. The surprise value from this cycle is invalidated.

*Methodological Falsification.* The hypothesis is untouched. The experiment design is flagged as incapable of testing the stated claims. The documented methodological constraint is sent back to the Adversarial Experiment Designer, which must design a new experiment avoiding the documented flaw. The graph is not updated.

*Theoretical Falsification.* The hypothesis's causal claims are contradicted by clean evidence from a valid methodology. The Causal Graph Manager receives a directed update specifying which edges to prune, reweight, or annotate as falsified. The failure record enters the accumulated history. The loop returns to the Hypothesis Generator, which must generate from the updated, narrower DAG.

**Corroboration path.** If the hypothesis survives the adversarial experiment -- the stress test fails to break it -- this registers as high Bayesian surprise (the adversary expected failure). The causal edges supporting the hypothesis are strengthened in the graph. The Adversarial Experiment Designer is invoked again to design a harder test. A hypothesis that survives multiple rounds of escalating adversarial pressure becomes the accepted explanation. The asymmetry is critical: survival under adversarial conditions constitutes far stronger evidence than survival under confirmatory conditions.

**Cycle boundary.** The Mode Controller checks budget consumption, surprise trends, and graph stability before authorizing the next cycle.

### 5.2 The Exploration Phase

The exploration phase runs before the falsification loop. Its sole output is a refined causal graph.

The Structural Prior Generator produces the initial DAG, loaded into the Causal Graph Manager with low edge confidences. The Epistemic Explorer identifies edges with highest uncertainty and designs low-cost system-identification experiments -- targeted interventional probes aimed at resolving specific edge uncertainties, not at testing full hypotheses. Probes are submitted through the Experiment Executor and DSL Environment Interface.

Results update the DAG: edges are confirmed (confidence increased), pruned (confidence below threshold), reweighted, or reversed (causal direction corrected). The Bayesian Surprise Evaluator computes information gain per probe to guide subsequent probe selection. The Epistemic Explorer reports convergence metrics to the Mode Controller, which governs the transition to falsification (Section 6.1).

The quality of the exploration phase directly determines whether the LFI can function reliably. If the graph entering falsification is grossly inaccurate, the LFI will misattribute failures -- blaming theory for what is actually a missing edge, or preserving a bad theory because the graph wrongly suggests the methodology was flawed. This is where the causal bootstrapping paradox manifests architecturally.

### 5.3 The Fault Isolation Decision Tree

The LFI executes a sequential, outside-in audit. The ordering is deliberate: check the outermost layer (implementation) first, then methodology, then theory. This prevents the most common error in generation-first systems -- penalizing a valid theory for a broken script.

**Stage 1: Implementation Audit.** The LFI examines the semantic IR for implementation-layer failures. The DSL's API separation makes theory-layer and implementation-layer operations structurally distinguishable. The audit checks: Did execution complete without framework-level exceptions? Do input data pipelines match the specification? Are numerical operations within precision bounds? Does the hardware/resource state match expectations? If any implementation fault is found, the classification is *Implementation Artifact*. The specific fault is documented. No further analysis proceeds.

**Stage 2: Methodological Audit.** If the implementation is clean, the LFI examines whether the experiment was methodologically capable of testing the hypothesis. This requires comparing the experiment specification against the hypothesis's causal claims: Does the experiment measure the variables the hypothesis links causally? Is the intervention on the hypothesized cause or a correlated proxy? Is the sampling sufficient to distinguish the effect from noise? Are there known confounders (from the current DAG) that the experiment did not control for? If methodology is insufficient, the classification is *Methodological Falsification*. An important caveat: the confounder check depends on the DAG's accuracy. If the DAG is missing a real confounder or contains a spurious one, this audit will either miss real confounders or flag phantom ones. The methodological audit's quality is bounded by the DAG's quality -- this is a propagation path for bootstrapping errors (see Section 8.5).

**Stage 3: Theoretical Evaluation.** Reached only if implementation is verified clean and methodology is verified sound. The LFI compares results against the hypothesis's predictions. If the evidence contradicts the hypothesis under clean, sound conditions, the classification is *Theoretical Falsification*, and the graph update directive specifies which edges to prune or reweight.

**Ambiguity handling.** When the LFI cannot confidently assign a failure to a single category, this is an escalation condition. The Mode Controller is notified, and depending on policy, the experiment may be re-run with enhanced logging, a more targeted probe may be designed, or a human may be escalated to.

### 5.4 Adversarial Calibration Feedback

The Adversarial Experiment Designer maintains a calibration loop. After each experiment, the actual surprise is compared to the predicted surprise. Persistent divergence -- the Designer consistently over-predicts or under-predicts information gain -- signals that either the DAG is inaccurate (the forward simulation model is wrong) or the Designer's candidate generation is biased. This calibration signal feeds to both the Designer and the Mode Controller. Persistent miscalibration is a trigger for re-exploration (Section 6.2).

Predicted and actual surprise values in this loop must be computed against the locked AggregateScore contract in Section 4.4.1, including fixed `n_terms=1`, log-scaled BF normalization (`c=0.083647`), and the decomposition invariant `sum(contribution_i) = aggregate_score`. Calibration diagnostics that violate the decomposition invariant are contract violations, not tuning outcomes.

---

## 6. Mode Transitions

### 6.1 Exploration to Falsification

The system transitions from graph refinement to hypothesis testing when the causal DAG is assessed as sufficiently accurate for fault isolation.

**Trigger conditions (conjunctive -- all must hold):**

*Marginal information gain decay.* The Bayesian Surprise Evaluator reports that information gained per exploration probe has fallen below a domain-calibrated threshold across a sustained window. The graph is no longer changing meaningfully with additional probes.

*Edge stability.* The Causal Graph Manager reports that no edges have been added, removed, or reversed in direction for a sustained window. Edge weights may still adjust, but the graph's structure has stabilized.

*Minimum coverage.* The DAG covers the causal variables relevant to the target hypothesis space. The Explorer has probed at minimum the variables the Hypothesis Generator would need to reference.

**Pre-transition check.** The Mode Controller verifies internal DAG consistency: no cycles, no orphaned nodes the priors indicated should be connected, no edges with confidence scores so low they represent unresolved unknowns. Inconsistencies trigger targeted exploration of the affected subgraph before the transition proceeds.

**Post-transition state.** The current DAG is snapshotted as the "exploration baseline." This snapshot is used to identify destabilized subgraphs if the system later returns to re-exploration.

**Caveat.** The convergence criteria are heuristic and domain-dependent. Defining "good enough" is an open research question. A premature transition means the LFI operates on a bad graph; a late transition wastes budget on diminishing returns.

### 6.2 Falsification to Re-Exploration

The falsification loop has generated evidence that the DAG is insufficiently accurate for reliable fault isolation. The system pauses hypothesis testing and returns to targeted graph refinement.

This transition requires the Mode Controller to distinguish "the hypothesis space is hard" from "my causal map is wrong." Four disjunctive conditions trigger the transition (any one is sufficient):

*Repeated ambiguous classifications.* The LFI produces a sustained pattern of low-confidence fault classifications -- failures it cannot cleanly assign. A pattern (not an occasional instance) indicates the DAG provides insufficient context for fault isolation.

*Graph oscillation.* The Causal Graph Manager detects contradictory updates across recent cycles -- edges pruned then re-added, or confidence scores cycling rather than converging. This signals that the graph structure is likely wrong.

*Persistent prediction miscalibration.* The Adversarial Experiment Designer's predicted information gain consistently diverges from actual surprise. Since predictions are forward-simulated against the DAG, systematic miscalibration indicates the DAG does not accurately model the domain's causal response to interventions.

*Hypothesis space exhaustion.* The Hypothesis Generator reports that the entire valid subspace has been pruned -- every region has been falsified -- yet no accepted explanation has emerged. The true explanation likely lies in causal structure the graph does not represent.

**Pre-transition check.** The Mode Controller compares the current DAG against the most recent baseline snapshot. The delta identifies which subgraphs were destabilized by the falsification loop. Re-exploration is then *targeted* -- the Epistemic Explorer focuses probes on the unstable regions rather than restarting from scratch. Accumulated failure records are preserved; they remain valid evidence even if the graph is being revised.

**Baseline versioning.** After re-exploration completes, the baseline snapshot is updated to the post-re-exploration graph state. The previous baseline is retained in a version stack. This design prioritizes the primary use case -- targeting probes at recently destabilized subgraphs, for which the most recent stable state is most useful -- while preserving the version history for cumulative drift analysis and audit.

**Return to falsification.** The transition back to falsification follows the same convergence criteria as Section 6.1, applied to the targeted subgraph rather than the full graph. This is the explicit return path: re-exploration terminates when the targeted subgraph satisfies the marginal gain decay, edge stability, and coverage checks.

**Consistency risk.** When the graph changes significantly during re-exploration, failures previously classified under the old graph may warrant reclassification. A theoretical falsification recorded under the old graph might, under the revised graph, be better understood as a methodological issue (the old graph missed a confounder). A full reclassification sweep after every re-exploration would be expensive and is not prescribed here. However, this is a real consistency concern: stale classifications in the failure history can mislead the Hypothesis Generator. The proposal flags this as an architectural risk requiring resolution during implementation (see Section 8.5).

### 6.3 Normal Operation to Human Escalation

The system encounters conditions it cannot resolve autonomously. Six escalation conditions fall into four categories.

**Category A: Diagnostic failure.**

*A1: Irresolvable fault classification.* The LFI cannot assign a failure even after enhanced logging. The evidence is genuinely ambiguous between categories. Escalation provides the raw evidence and requests human judgment on fault assignment.

*A2: Causal graph collapse.* The system has entered re-exploration more than a configured threshold number of times, or re-exploration itself fails to converge. The initial structural priors may be fundamentally misspecified. Escalation requests human review of the prior structure.

**Category B: Resource boundary.**

*A3: Budget exhaustion.* The cycle or compute budget is depleted without convergence. Escalation reports the current best explanation with its confidence and requests a decision: allocate more budget, adjust constraints, or accept the current result.

**Category C: Convergence ambiguity.**

The Bayesian Surprise Evaluator reports that predicted information gain has dropped below threshold. The Mode Controller distinguishes two sub-cases:

*A4: Flatline-because-converged.* The hypothesis has survived multiple adversarial rounds. The graph is stable, classifications have been clean, prediction calibration is tight, and no remaining experiments within the bounded subspace are predicted to yield significant information gain. Escalation recommends acceptance and requests human confirmation that the system is finished.

*A5: Flatline-because-stuck.* Predicted information gain is low but the hypothesis has not been strongly corroborated. Graph instability or calibration drift may be present. The bounded subspace may be too restrictive or the DAG may be missing relevant structure. Escalation requests human review of domain constraints and graph completeness.

**Category D: Prediction integrity.**

*A6: Persistent uncorrectable miscalibration.* The Designer's prediction accuracy has not recovered even after re-exploration. Escalation reports that the system's experimental model is systematically wrong in ways graph refinement cannot fix, and requests domain-specific correction.

**Human reintegration (Gap A).** All six conditions specify what the system communicates to the human. The inverse -- what the human provides back -- requires an interface contract that this architecture specifies at the type level but does not design. The five reintegration action types are: (a) direct DAG edits (add, remove, or reverse edges), (b) new structural priors injected into the Structural Prior Generator, (c) expansion or modification of bounded subspace constraints in the DSL Environment Interface, (d) override of a specific LFI classification, (e) budget extension or constraint adjustment. Each escalation condition maps to a subset of these action types; the mapping and the interface are implementation-level concerns.

---

## 7. Evaluation Strategy

### 7.1 Per-Component Evaluation

Each component can be evaluated independently before integration.

**Structural Prior Generator.** Evaluate by comparing generated priors against known causal structures in well-characterized domains. Measure: structural Hamming distance between the generated DAG and the ground-truth DAG. The generator need not be perfect -- the question is whether its errors are correctable by the Epistemic Explorer.

**Causal Graph Manager.** Evaluate by injecting known update sequences and verifying that the DAG converges to the correct structure. Measure: convergence rate, consistency maintenance (no cycles, no orphans), and snapshot/versioning correctness.

**Epistemic Explorer.** Evaluate by measuring how many probes are required to refine an intentionally degraded prior DAG to a target accuracy threshold. Measure: probe efficiency (information gain per probe), convergence rate, and the accuracy of the refined graph relative to ground truth.

**Hypothesis Generator.** Evaluate by verifying that generated hypotheses are within the valid subspace defined by the DAG and that previously falsified regions are excluded. Measure: constraint satisfaction rate and diversity of generated hypotheses.

**Adversarial Experiment Designer.** Evaluate by comparing its predicted information gain against actual information gain across a test suite. Measure: calibration accuracy (predicted vs. actual surprise), and whether selected experiments are within domain-valid bounds. Also measure: does the Designer preferentially select experiments that expose flaws, or does it converge on safe, uninformative configurations?

**DSL Environment Interface.** Evaluate by testing constraint enforcement: does it correctly reject out-of-bounds experiment specifications? Does it accurately report domain constraints? Standard interface testing.

**Experiment Executor.** Evaluate by verifying completeness of trace capture. Measure: given a known set of execution events, does the captured trace log contain all events necessary for downstream fault isolation?

**Trace Semantics Engine.** Evaluate by providing trace logs with known failure causes and measuring whether the IR correctly represents the failure's location and nature. Measure: accuracy of the semantic mapping against ground-truth fault annotations.

**Lakatosian Fault Isolator.** Evaluate by providing the LFI with traces containing known faults (planted implementation errors, planted methodological flaws, genuine theoretical contradictions) and measuring classification accuracy. The three-way classification accuracy is the most critical metric for the entire system.

**Bayesian Surprise Evaluator.** Evaluate by verifying that computed KL divergence matches analytically derived values for known prior-posterior pairs. For the predicted-gain sub-responsibility, measure calibration against actual outcomes.

**Mode Controller.** Evaluate by simulating metric trajectories (convergence, oscillation, exhaustion, flatline) and verifying that the correct transitions and escalations trigger. Standard state-machine testing.

### 7.2 End-to-End Evaluation: The Hidden Confounder Litmus Test

The definitive end-to-end evaluation is a head-to-head comparison between ATHENA and a generation-first baseline within a controlled synthetic environment.

**Setup.** Both systems operate within the same structured DSL environment. They receive access to a dataset containing a complex, non-linear physical relationship. The dataset is deliberately engineered to contain a hidden spurious confounder -- a sophisticated data artifact that enables 98% accuracy if exploited but fails entirely on a strictly withheld, out-of-distribution (OOD) test set. The confounder is designed to be invisible to systems that optimize for in-distribution metrics and discoverable only through interventional experiments that probe confounding structure.

**Constraints.** Both systems receive identical, strictly limited computational budgets and a maximum of 50 experiment execution cycles.

**Expected baseline behavior.** A generation-first system optimizing for reward scalars is expected to rapidly generate a hypothesis exploiting the confounder, achieve high validation accuracy, cease exploration, and fail on the OOD test.

**Expected ATHENA behavior.** ATHENA is expected to generate an initial hypothesis, identify the spurious correlation during adversarial experiment design (the adversary seeks data distributions that break the high-accuracy theory), record the failure, perform fault isolation to tag the dataset as confounded (a methodological failure, not a theoretical one), and subsequently search for the true causal mechanism.

**Validation criteria.** ATHENA's thesis is validated if and only if, within the 50-cycle limit, it identifies the confounder, bypasses it, and outputs a causal DAG representing the true relationship, while the generation-first system does not. The thesis is falsified if ATHENA exhausts its budget analyzing logs without discovering the true mechanism, or if the generation system successfully evolves past the confounder through volumetric search without causal analysis.

This test is deliberately designed to be passable by either architecture in principle. It does not assume ATHENA wins. It tests the specific architectural claim: that causal fault isolation is more sample-efficient than stochastic generation in the presence of systematic methodological traps.

---

## 8. Architectural Risks

### 8.1 Per-Component Risks

**Structural Prior Generator (Severity: Critical).** This component has the widest gap between its importance and its design maturity. It is the entry point for the entire system, its output quality determines whether the causal bootstrapping paradox manifests, and the architecture specifies almost nothing about its internals. If the initial priors are fundamentally misspecified -- not merely imprecise, but structurally wrong (missing critical variables, inverted causal directions) -- the Epistemic Explorer may be unable to detect or correct the errors, and the system enters self-reinforcing loops of incorrect deduction. Risk classification: this component requires novel research, and its failure mode is catastrophic (silent corruption that propagates through every downstream component).

**Trace Semantics Engine (Severity: High).** The IR design is an unsolved research problem. If the engine produces an insufficiently resolved IR, the LFI cannot distinguish between failure categories, and the system's core differentiator is disabled. The DSL constraint improves tractability relative to arbitrary code, but the gap between "structured trace logs" and "semantically parsed causal narratives" remains wide. Risk classification: requires novel research.

**Adversarial Experiment Designer (Severity: Medium, downgraded from High after Session 6 contract lock).** The reward aggregation contract is now specified (Section 4.4.1 and `aggregate_score_recommendation.md` v1.0), so the primary risk is no longer research uncertainty but implementation drift from the locked contract. Residual failure modes are: (a) contract-violating implementation (wrong normalization branch, wrong `n_terms`, missing guardrail, broken decomposition invariant), (b) calibration degradation when operating data moves outside validated ranges, and (c) forward-simulation compute bottlenecks over large DAGs. Risk classification: specified, pending implementation and operational monitoring.

**Epistemic Explorer (Severity: Medium).** Convergence criteria are undefined and domain-dependent. A premature transition delivers a bad graph to the falsification loop; a late transition wastes budget. Risk classification: open research question, but bounded -- the failure mode is performance degradation, not silent corruption.

**Lakatosian Fault Isolator (Severity: Medium).** The three-way classification is novel and unvalidated. Edge cases (failures that genuinely span categories, or novel failure modes not anticipated by the three-category taxonomy) may produce misclassifications. Risk classification: the classification framework itself may need extension; this is architectural research, not engineering.

**Bayesian Surprise Evaluator (Severity: Low-Medium).** KL divergence computation is well-studied, but computing it over causal DAG structures (rather than parameter spaces) introduces representational challenges. The dual responsibility (post-experiment computation and pre-experiment forward simulation) means this component's failure manifests in two different ways: corrupted trend data and miscalibrated experiment selection. Risk classification: primarily engineering, with some adaptation research needed.

**Remaining components (Severity: Low).** The Hypothesis Generator, Experiment Executor, DSL Environment Interface, Causal Graph Manager, and Mode Controller are either well-understood engineering problems or compositions of existing techniques. Their risks are primarily integration risks, not research risks.

### 8.2 Systemic Risk: Causal Graph as Single Point of Failure

The Causal Graph Manager is the most connected component in the architecture. It receives from four components and is queried by four others. Every analysis decision (fault classification, experiment design, hypothesis generation, mode transition) depends on its accuracy. A corrupted graph does not produce a visible error -- it produces systematically biased decisions across the entire system. This is the highest-severity systemic risk: a single component failure with no localized symptom and global downstream impact.

Mitigation is architectural rather than component-level: the versioned snapshot system, the re-exploration transition, and the human escalation conditions (A2, A5, A6) are all designed to detect and respond to graph degradation. But detection depends on the degradation producing observable symptoms (oscillation, miscalibration, ambiguous classifications). Silent graph corruption -- where the graph is wrong in ways that produce consistent but incorrect fault classifications -- is the hardest failure mode to detect and the most damaging.

### 8.3 Systemic Risk: Bootstrapping Error Propagation

The warm-start constraint creates a dependency chain: Structural Prior Generator -> Causal Graph Manager -> Epistemic Explorer -> refined graph -> LFI -> graph updates. If the initial priors contain structural errors that the Epistemic Explorer cannot detect (because the errors are in regions the Explorer does not probe, or because the errors are self-consistent), these errors propagate through the entire chain. The LFI will classify failures against a wrong graph, producing wrong update directives, which further entrench the wrong graph.

This is architecturally distinct from the single-point-of-failure risk (8.2). That risk concerns corruption of the graph *during operation*. This risk concerns corruption *at initialization* that the system's self-correction mechanisms cannot reach.

### 8.4 Systemic Risk: Incomplete Observability

The LFI's outside-in audit assumes that the protective belt is observable in the trace logs. In physical sciences and real-world systems, unrecorded state changes (equipment drift, latent environmental noise, measurement degradation) introduce invisible failures. When the trace log does not contain the data of the actual failing component, the LFI will misattribute the failure to either the theory or an innocent auxiliary variable. This misattribution corrupts the causal graph and derails subsequent cycles. The DSL constraint partially mitigates this (simulation environments have more complete observability than physical labs), but does not eliminate it -- even simulations can have hidden state (numerical precision loss, non-deterministic parallelism, undocumented framework behavior).

### 8.5 Systemic Risk: Classification Staleness

The methodological audit (Stage 2 of the LFI decision tree) checks for confounders using the current DAG. If the DAG is wrong about confounders -- missing real ones or containing spurious ones -- the audit inherits those errors. This is a propagation path for bootstrapping errors: an incorrect DAG produces incorrect methodological audits, which produce incorrect classifications, which produce incorrect graph updates.

Additionally, when the system transitions to re-exploration and the graph changes significantly, failures previously classified under the old graph may warrant reclassification. A failure tagged as "theoretical falsification" under the old graph might, under the revised graph, be better understood as methodological (the old graph missed a confounder). Stale classifications in the accumulated failure history can mislead the Hypothesis Generator into avoiding hypothesis-space regions that were never actually falsified. The architecture acknowledges this as a consistency concern requiring resolution during implementation. Options range from full reclassification sweeps (expensive but thorough) to targeted reclassification of failures whose classification depended on subgraph regions that changed during re-exploration (more efficient but requires dependency tracking in the failure records).

### 8.6 Systemic Risk: Brute-Force Scaling Crossover

ATHENA's per-failure analysis is computationally expensive: parsing trace logs, running causal inference, computing KL divergences, and prompting LLMs for deep reflection on a single failure. If experiment execution costs drop significantly -- through faster simulators, specialized hardware, or efficient emulation -- the sample efficiency advantage evaporates. A generation-first system running millions of cheap experiments may outperform ATHENA analyzing hundreds of expensive ones. The architecture's advantage is conditional on the cost asymmetry between hypothesis generation and experiment execution persisting in the target domain. This is not an architectural flaw but a scope boundary: ATHENA should not be deployed in domains where this asymmetry has collapsed.

---

## 9. Relation to Prior Work

ATHENA's architecture differs from existing AI co-scientist systems in structural ways that follow from its falsification-first epistemology. The differences are in the architecture's topology, not in capability claims.

### 9.1 Sakana AI Scientist V2

Sakana V2 uses a progressive agentic tree search where each node represents a solution state assigned a scalar evaluation score. When a node receives a low score or throws an exception, the system lacks architecture to determine whether the theory is wrong or the implementation failed. Empirically, 41% of V2's operational errors are computational execution issues (tensor mismatches, syntax faults) that are conflated with theoretical inadequacy.

**Structural difference.** ATHENA replaces the tree search with a directed falsification loop. Where V2 branches by mutating the solution state (creating sibling nodes), ATHENA branches by classifying the failure (routing to implementation fix, methodological redesign, or theoretical revision). V2's branching is combinatorial -- it generates alternatives. ATHENA's branching is diagnostic -- it isolates causes. The key architectural consequence: V2's search tree grows exponentially with depth; ATHENA's causal graph grows linearly with the number of confirmed or falsified edges.

### 9.2 Google AI Co-Scientist

Google's system uses multi-agent tournament evolution with an Elo auto-evaluation metric. Hypotheses compete in ranking tournaments; losers are pruned or genetically mutated. The explicit correlation between "spending more time in computation" and "improving the Elo metric" encodes a brute-force scaling assumption.

**Structural difference.** Elo ranking is an ordinal metric -- it tells you which hypothesis is *better than* another, not *why* either hypothesis fails. ATHENA's LFI produces a causal attribution, not a ranking. When a Google Co-Scientist hypothesis loses a tournament, the system discards it. When an ATHENA hypothesis fails a test, the system extracts a permanent constraint on the causal graph. The tournament system treats negative results as penalties; ATHENA treats them as data. This difference is visible in the architecture's data flow: Google's system has a unidirectional path (generate -> evaluate -> prune); ATHENA has a bidirectional path (generate -> test -> classify -> update graph -> generate from updated graph).

### 9.3 Standard Active Learning

ATHENA's Adversarial Experiment Designer is architecturally related to discriminative active learning, which selects unlabeled data points the model is most uncertain about. The relationship is genuine but the differences are significant.

Standard active learning operates on a parameter space: it selects data points that maximize uncertainty in the model's parameters. ATHENA's adversary operates on a causal graph: it selects experiments that maximize uncertainty in the graph's structure. Standard active learning assumes a fixed model class and seeks to identify parameters within it. ATHENA's loop can modify the model class itself (by adding or removing edges in the causal graph), which standard active learning cannot do.

Additionally, standard active learning does not include fault isolation. It assumes that every observation is a valid signal. ATHENA's architecture explicitly accounts for the possibility that observations are corrupted by implementation errors or methodological flaws, and filters them through the LFI before they update the model.

### 9.4 AI2 CodeScientist and Related Systems

AI2 CodeScientist uses genetic search with a "generate-execute-reflect" debugging loop. The reflection step is largely constrained to syntactic error correction -- when a script fails, the system prompts an LLM to fix the script. This is architecturally similar to ATHENA's implementation-artifact classification, but CodeScientist does not proceed further. It lacks the methodological and theoretical classification stages, and it does not maintain a causal graph that accumulates structural knowledge across cycles. Each reflection is local and stateless; ATHENA's fault isolation is global and graph-updating.

Other systems (HKUDS AI-Researcher, NovelSeek) incorporate multi-dimensional scoring or multi-agent coalitions, but share the same fundamental architectural limitation: they evaluate hypotheses against scalar metrics rather than performing structured causal analysis of failures. The scoring is more sophisticated; the epistemology is the same.

---

## Appendix: Open Research Dependencies

This architecture identifies five components or sub-responsibilities that require novel research rather than engineering. Their resolution order is partially constrained by dependencies.

| Priority | Component/Sub-Responsibility | Research Question | Dependency |
| :--- | :--- | :--- | :--- |
| 1 | Trace Semantics Engine | IR design for translating DSL traces to semantic failure representations | Blocks LFI effectiveness |
| 2 | Adversarial Experiment Designer AggregateScore contract | Session 6 locked formalization complete (Section 4.4.1); remaining work is implementation against fixed parameters, guardrail, and invariant checks | Implementation blocker for production adversarial design |
| 3 | Epistemic Explorer convergence criteria | Defining "good enough" for exploration-to-falsification transition | Blocks reliable mode transitions |
| 4 | Structural Prior Generator internals | Generating accurate enough initial DAGs from LLM priors | Blocks bootstrapping quality |
| 5 | Bayesian Surprise Evaluator over DAGs | Computing predicted KL divergence over causal graph structures | Blocks experiment selection |

**A note on priority vs. severity.** The ordering above reflects resolution urgency, not failure severity. Item 4 (Structural Prior Generator) is rated Critical severity in Section 8.1 -- its failure mode is catastrophic silent corruption that propagates through every downstream component. It ranks #4 in resolution priority because the warm-start approach provides a usable, if imperfect, starting point that defers the failure rather than preventing it. This is a deliberate architectural bet: the system can operate with imperfect priors (the Epistemic Explorer and re-exploration transitions exist to compensate), but it cannot operate at all without a working Trace Semantics Engine. Item 2's research formalization is now locked in Section 4.4.1; its remaining blocker is implementation fidelity. A collaborator should read this as: item 1 remains the immediate research blocker; item 2 is now an implementation blocker; item 4 determines whether the functioning system produces correct results.

Items 3 and 5 affect system performance and calibration but do not block core functionality.
