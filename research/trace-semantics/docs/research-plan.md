# P1: Trace Semantics IR Research Investigation

## Context

The Trace Semantics Engine is ATHENA's highest-priority research dependency (ARCHITECTURE.md 8.1, HIGH severity). It translates raw DSL trace logs into a structured IR for the Lakatosian Fault Isolator's three-stage audit (implementation -> methodology -> theory). Without it, the LFI cannot function, and downstream components (Bayesian Surprise, Adversarial Designer, Convergence Criteria) all cascade-fail.

Current state: **NOT STARTED**. FINDINGS.md exists with research question and scaffolding but zero investigation entries.

The 21% Top@1 baseline for unstructured trace RCA is the number to beat. DSL structure improves this substantially — the investigation determines how much and what IR design captures it.

---

## Plan: 10 Sessions, 5 Steps

Steps 1+2 run in parallel (sessions 1-3). Steps 3-5 are sequential, each informed by prior results. Four decision gates pause progress if critical assumptions fail.

### Sessions 1-3: Survey DSL Trace Formats (Step 1) + Survey Existing IR Designs (Step 2)

**These run in parallel.**

#### Step 1: DSL Trace Formats

**Session 1** — OpenMM trace characterization
- Research: OpenMM docs, GitHub source (`openmm/app/statedatareporter.py`, `simulation.py`), GitHub issues for failure modes
- Questions to answer: Complete reporter type inventory, exception/error exposure mechanism, theory-implementation API boundary (`ForceField.createSystem()` is the key boundary), execution metadata available, custom reporter extensibility, walk-through of a NaN energy failure trace
- Deliverable: `research/trace-semantics/dsl-evaluation/openmm-trace-analysis.md`

**Session 2** — GROMACS + VASP trace characterization
- GROMACS: `.log` structure, `.edr` binary format (via `panedr`), error/warning taxonomy, theory-implementation boundary (`.mdp` theory params vs. implementation params, `grompp` as compilation step), walk-through of a constraint failure
- VASP: `vasprun.xml` structure (parseable via pymatgen), OUTCAR/OSZICAR content, failure signaling (SCF non-convergence, ionic relaxation failure), DFT-specific theory-implementation boundary (INCAR theory tags vs. parallelization tags), closed-source instrumentation constraints
- Deliverables: `dsl-evaluation/gromacs-trace-analysis.md`, `dsl-evaluation/vasp-trace-analysis.md`

**Session 3** — Cross-framework synthesis
- Build comparison matrix: rows = trace elements (state snapshots, energy series, convergence metrics, error messages, parameter echo, execution metadata, trajectory data); columns = OpenMM / GROMACS / VASP with format, access method, and theory/implementation layer tag
- Produce failure mode taxonomy per framework (implementation / methodology / theory / ambiguous)
- Assess trace completeness: what fraction of execution state is capturable? What's hidden?
- Deliverable: `dsl-evaluation/cross-framework-synthesis.md`, FINDINGS.md log entry for full Step 1

> **DECISION GATE 1**: If theory-implementation boundary is NOT cleanly API-enforced in any framework, flag for adversarial review. May need to narrow target DSL set or add instrumentation wrappers.

#### Step 2: Existing IR Designs

**Session 1 (parallel)** — RCA + Formal Verification IRs
- Sources: LLM-based RCA papers (arxiv:2403.04123, arxiv:2601.22208), Chain-of-Event FSE24, LLVM IR (SSA), MLIR dialects, Boogie/Why3 verification IRs, DRAT proofs, program analysis IRs (Soot, WALA)
- Key questions: How do RCA IRs represent causal chains? How do verification IRs encode specification-vs-implementation? How do counter-example traces structure failure witnesses? What IR decisions affect queryability?

**Session 2 (parallel)** — Provenance + Scientific Workflow IRs
- Sources: W3C PROV-DM, ProvONE, provenance query literature, FAIR provenance, process mining
- Key questions: Can Entity-Activity-Agent model represent theory-implementation distinction? How do provenance query languages handle causal reasoning? Scalability for megabyte-scale traces? How is expected-vs-actual outcome represented?

**Session 6** — Comparative IR synthesis (after sessions 1-2 complete)
- Catalog transferable patterns: counter-example traces, Entity-Activity-Agent, event chains, SSA data flow, multi-level abstraction, spec-implementation separation, causal dependency graphs
- Map each pattern to LFI audit stage requirements
- Identify anti-patterns (post-mortem-only designs, spec-implementation conflation)
- Deliverable: `dsl-evaluation/ir-pattern-catalog.md`, FINDINGS.md log entry for full Step 2

> **DECISION GATE 2**: If PROV-DM's Entity-Activity-Agent model fits well, it becomes the IR foundation. If nothing transfers, we need a fully novel design (higher risk). Document explicitly.

---

### Session 4: Map LFI Audit to IR Requirements (Step 3a)

Work backward from ARCHITECTURE.md 5.3 to derive minimum semantic distinctions:

**Stage 1 (Implementation Audit)** — IR must represent:
1. Execution event (timestamped, completion status)
2. Exception event (type, location, stack equivalent)
3. Data validation event (input name, expected vs. actual)
4. Numerical status event (precision mode, NaN/overflow flag, location)
5. Resource status (platform, memory, device state)

**Stage 2 (Methodology Audit)** — IR must represent:
6. Observable measurement (variable, method, values)
7. Intervention specification (parameter, range, controls)
8. Sampling metadata (count, distribution, power)
9. Controlled variable set (what held constant, how)
10. DAG linkage (IR elements <-> DAG nodes/edges)

**Stage 3 (Theory Evaluation)** — IR must represent:
11. Prediction record (hypothesis, predicted value/distribution)
12. Observation record (actual value/distribution)
13. Comparison result (effect size, divergence, confidence interval)

**Cross-cutting**:
14. Provenance chain (IR element -> raw trace source)
15. Layer tag (theory / implementation)
16. Temporal ordering (causal sequence preservation)
17. Queryability (lookup by layer, event type, time range, variable, DAG node)

Deliverable: FINDINGS.md log entry with 17+ minimum semantic distinctions.

---

### Session 5: Characterize 21% Baseline (Step 4)

- Trace the 21% Top@1 figure to its source paper (likely "Empowering Practical Root Cause Analysis" or similar LLM-RCA evaluation)
- Understand methodology: what datasets, what "unstructured traces" means, candidate set size
- Identify structural properties that improve accuracy: temporal ordering, causal annotations, event type taxonomies, severity levels, typed parameters
- Estimate DSL-specific improvement factors: known schema, API-enforced layer separation, deterministic execution, typed parameters
- Identify residual hard cases that structure alone doesn't solve

Deliverable: FINDINGS.md log entry with baseline characterization and ranked structural properties.

> **DECISION GATE 3**: If 21% figure is from a non-transferable domain (cloud microservices, not scientific computing), reframe the baseline. The question becomes: what accuracy is reasonable for structured DSL traces? If no existing data, this becomes empirical for Step 5.

---

### Session 7: Requirements Refinement (Step 3b)

Map the 17+ minimum requirements against actual DSL trace content from Step 1:

- **Coverage matrix**: For each semantic distinction x each framework: directly available / derivable / unavailable
- **Gap analysis**: For each gap: fillable by instrumentation? By pre-execution analysis? By inference? Or fundamentally unobservable?
- **Feasibility assessment**: Per LFI stage, can it produce deterministic answers? Under what conditions?

Deliverable: FINDINGS.md log entry with refined requirements and feasibility assessment.

> **DECISION GATE 4**: If ANY LFI stage is infeasible given trace data, escalate. Options: (a) narrow target DSL set, (b) require instrumentation wrappers (per VISION.md Open Question #2), (c) weaken deterministic audit to probabilistic. Each has different architectural implications.

---

### Sessions 8-9: Draft Candidate IR Schemas (Step 5a)

Three candidate designs with explicit tradeoffs:

**Candidate 1: Layered Event Log (LEL)** — Flat sequence of typed, layer-tagged, causally-linked events. Inspired by PROV-DM + RCA event chains.
- (+) Simple, Rust zero-copy friendly, easy querying
- (-) Limited complex causal structure, layer assignment at parse time, weak prediction-vs-observation support

**Candidate 2: Dual-Graph IR (DGR)** — Parallel specification graph (what should happen) and execution graph (what did happen), with divergence detection.
- (+) Natural Stage 3 support, divergence is core operation
- (-) Complex construction, graph alignment is non-trivial, harder for Rust

**Candidate 3: Typed Assertion Log (TAL)** — Sequence of typed logical assertions (precondition/postcondition/invariant/prediction), each verified or violated.
- (+) Directly models LFI as assertion checking, streamable, Rust-friendly
- (-) Needs assertion compiler, continuous data doesn't decompose naturally to booleans

Each evaluated against: 17+ requirements from Step 3, DSL trace characteristics from Step 1, Rust compatibility (ADR 001), hidden confounder litmus test compatibility.

Deliverable: FINDINGS.md log entry with all three designs and tradeoff analysis.

---

### Session 10: Prototype + Evaluation (Step 5b)

Build throwaway prototype for the most promising candidate (Python, per ADR 001 research-phase flexibility):

1. **Mock trace generator** — Synthetic traces mimicking one DSL framework for three scenarios: implementation failure (NaN from precision), methodology failure (wrong sampling), theory failure (wrong parameters)
2. **IR parser prototype** — Parse mock traces into candidate IR
3. **LFI query prototype** — Execute three-stage audit against IR, demonstrate correct classification per scenario

Evaluation criteria:
- Three failure types produce distinguishable IR representations
- Three-stage audit expressible as deterministic queries
- Queries are efficient (linear or better in trace size)
- Compatible with hidden confounder litmus test scenario
- Amenable to eventual Rust implementation

Deliverables:
- 3 prototype scripts in `research/trace-semantics/prototypes/`
- Prototype Index entries in FINDINGS.md
- Final FINDINGS.md log entry with evaluation results and design recommendation
- Updated Accumulated Findings (What We Know / What We Suspect / What We Don't Know)

---

## Critical Files

| File | Role |
|---|---|
| `research/trace-semantics/FINDINGS.md` | Primary output — all log entries, findings, prototype index |
| `research/trace-semantics/dsl-evaluation/` | Step 1+2 analysis documents |
| `research/trace-semantics/prototypes/` | Step 5b throwaway code |
| `ARCHITECTURE.md` §4.5, §5.3, §8.1 | Source of truth for component def, LFI audit, risk |
| `VISION.md` §4.1, Open Q#1 | 21% baseline claim, "Semantic Language of Failure" framing |
| `evaluation/hidden-confounder/README.md` | Downstream constraint — IR must support confounder discovery |
| `decisions/001-python-rust-core.md` | Rust target for parsing throughput; Python OK for prototypes |

## Verification

After each session:
1. FINDINGS.md has a new investigation log entry (Scope, Method, Findings, Implications, Open Threads)
2. Deliverable files exist and are referenced from FINDINGS.md
3. Accumulated Findings sections updated with evidence-backed claims

After Step 5:
4. Prototype correctly classifies all three failure scenarios
5. Design recommendation documented with rationale
6. Coverage matrix shows no critical unfillable gaps (or gaps are explicitly escalated)
7. IR design is compatible with hidden confounder litmus test requirements
