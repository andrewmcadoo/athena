# ATHENA Build Guide

Status snapshot. All content derived from project artifacts. Read source documents for detail.

---

## 1. What's Done

| Artifact | Summary |
|---|---|
| **VISION.md** | 8-section stress-tested vision. Core thesis, four differentiators (LFI, adversarial design, pathology resistance, epistemic exploration), five honest limitations, litmus test definition, five open questions. Source of truth for all claims. |
| **ARCHITECTURE.md** | 11-component architecture across 5 functional groups. Dependency graph, information flow for falsification loop and exploration phase, fault isolation decision tree, 6 mode transitions/escalation conditions, per-component and systemic risk analysis (6 risks), evaluation strategy. Source of truth for structural decisions. |
| **ADR 001** | Language split: Rust for Causal Graph Manager, Trace Semantics Engine, Bayesian Surprise Evaluator (performance-critical). Python for remaining 8 components (orchestration). PyO3/maturin interop. Prototypes may use Python regardless. |
| **5x FINDINGS.md** | Research scaffolding for all five open dependencies. Each contains: research question, architecture references, key definitions, next steps. All NOT STARTED (no investigation log entries). |
| **Evaluation spec** | Hidden Confounder Litmus Test specification (evaluation/hidden-confounder/README.md). Defines DSL environment, confounded dataset, generation-first baseline, evaluation harness, and pass/fail validation criteria. NOT STARTED; specification only. |

**Three non-negotiable constraints** established and surviving two adversarial review passes: DSL-only environments, warm-started causal priors, bounded adversarial design (VISION.md §2, ARCHITECTURE.md §3).

---

## 2. Where We Are

**Phase:** Research (Active Investigation). No production code exists. No investigation log entries in any FINDINGS.md.

### Open Research Investigations

| # | Investigation | Status | Priority | Severity | Blocks | Type |
|---|---|---|---|---|---|---|
| 1 | Trace Semantics IR | NOT STARTED | P1 | High | LFI effectiveness, litmus test | Research |
| 2 | Adversarial Reward Function | NOT STARTED | P2 | High | Adversarial experiment design, litmus test | Research |
| 3 | Exploration Convergence Criteria | NOT STARTED | P3 | Medium | Reliable mode transitions | Research |
| 4 | Structural Prior Generator Quality | NOT STARTED | P4 | **Critical** | Bootstrapping quality (deferred, not prevented) | Research |
| 5 | Bayesian Surprise over DAGs | NOT STARTED | P5 | Low-Medium | Experiment selection calibration | Research + Engineering |

**Dependency structure** (ARCHITECTURE.md Appendix):
- Items 1-2 block the system from functioning at all.
- Item 4 determines whether the functioning system produces correct results (Critical severity, deferred by warm-start).
- Items 3, 5 affect performance/calibration but do not block core functionality.
- Litmus test blocked by items 1 + 2 (evaluation/hidden-confounder/README.md Hard Dependencies).

### Evaluation Spec

NOT STARTED. Cannot be implemented until trace-semantics and adversarial-reward produce findings. Environment design depends on IR format (for trace compatibility) and adversary reward function (to ensure confounder is discoverable by design).

---

## 3. What's Next

Ordered by dependency chain, then priority. Research investigations produce findings; engineering builds on those findings; integration assembles components for the litmus test.

### Research — Blocking (must resolve before system can function)

#### P1: Trace Semantics IR (research/trace-semantics/FINDINGS.md)

| Step | Work |
|---|---|
| Survey DSL trace formats | Collect actual trace output structure from OpenMM, GROMACS, VASP. Identify common vs. framework-specific elements. |
| Survey existing IR designs | Review IRs in root cause analysis, formal verification (SAT solvers, model checkers), program analysis. Identify transferable patterns. |
| Map LFI audit to IR requirements | Derive minimum semantic distinctions from ARCHITECTURE.md §5.3 three-stage audit (implementation, methodology, theory). |
| Characterize the 21% baseline | Understand what drives low general RCA accuracy and what DSL structural properties improve it. |
| Draft candidate IR schemas | 2-3 candidate designs with tradeoffs (resolution vs. generality vs. cost), evaluated against minimum requirements. |

#### P2: Adversarial Reward Function (research/adversarial-reward/FINDINGS.md)

| Step | Work |
|---|---|
| Survey active learning / Bayesian experimental design | How is information gain formalized? Which handle bounded search spaces? |
| Characterize conservatism-vs-boundary-seeking spectrum | Formally describe both failure modes as functions of reward properties. Identify controlling parameters. |
| Analyze Noisy TV in DSL contexts | How does Noisy TV manifest in OpenMM/GROMACS/VASP? How does deterministic subspace constraint interact with noise? |
| Calibration loop constraints | What does ARCHITECTURE.md §5.4 calibration feedback require of the reward function's functional form? |
| Draft candidate reward specifications | 2-3 formalizations with tradeoff profiles, each evaluable against calibration requirements. |

### Research — Non-blocking (affects performance/calibration/correctness)

#### P3: Exploration Convergence (research/exploration-convergence/FINDINGS.md)

Survey convergence criteria in model-based RL and Bayesian optimization. Formally analyze the three trigger conditions (ARCHITECTURE.md §6.1) for pathological cases. Investigate domain-dependence across DSL environments. Analyze re-exploration subgraph case (§6.2). Draft 2-3 candidate convergence specifications with domain-calibration parameters.

#### P4: Structural Prior Quality (research/structural-priors/FINDINGS.md)

**Critical severity** — silent corruption propagates through all downstream components.

Survey LLM causal graph generation benchmarks (SHD values, systematic error patterns). Characterize correctable-vs-uncorrectable misspecification boundary. Investigate domain-seed alternatives. Analyze self-reinforcing loop conditions. Design testable quality threshold experiment (synthetic known structure, LLM generates DAG, Explorer corrects, measure initial SHD vs. post-exploration accuracy).

#### P5: Surprise over DAGs (research/surprise-over-dags/FINDINGS.md)

Survey information-theoretic quantities over discrete graph structures. Analyze forward simulation computational complexity. Coordinate with adversarial-reward investigation (reward function depends on predicted gain). Survey approximation methods for intractable exact KL. Investigate surprise invalidation edge cases.

### Integration — Blocked by Research

| Step | Blocked By | Type | Source |
|---|---|---|---|
| Implement litmus test DSL environment | P1 (IR format), P2 (reward function) | Engineering | evaluation/hidden-confounder/README.md |
| Engineer confounded dataset | P1, P2 | Engineering | evaluation/hidden-confounder/README.md §2 |
| Build generation-first baseline | P1 (needs same DSL env) | Engineering | evaluation/hidden-confounder/README.md §3 |
| Run end-to-end hidden confounder test | All above | Integration | ARCHITECTURE.md §7.2 |

### Classification Key

Per ARCHITECTURE.md §8.1:
- **Research problems**: Trace Semantics IR (#1), Adversarial Reward (#2), Exploration Convergence (#3), Structural Prior quality thresholds (#4), LFI three-way classification validation (§8.1 Medium)
- **Research + engineering**: Surprise over DAGs (#5 — KL computation well-studied, adaptation to graph structures requires research)
- **Engineering problems**: DSL Environment Interface wrappers (VISION.md Open Question #2), classical verification tool integration (VISION.md Open Question #4), Causal Graph Manager, Experiment Executor, Mode Controller
- **Engineering, blocked by research**: Litmus test implementation, Rust components per ADR 001 (interfaces must be validated through research-phase prototypes first)
