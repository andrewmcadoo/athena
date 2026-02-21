# Trace Semantics Engine: IR Design

## Research Question

What intermediate representation (IR) can translate raw DSL trace logs from structured simulation frameworks (OpenMM, GROMACS, VASP) into semantic failure representations suitable for three-way causal fault classification? The IR must preserve enough structure for the Lakatosian Fault Isolator to deterministically distinguish implementation-layer failures from methodology-layer failures from theory-layer contradictions. Success criteria: an IR specification that, given a trace log containing a known planted fault, enables correct fault classification at a rate significantly exceeding the 21% Top@1 baseline reported for general unstructured traces. This investigation blocks LFI effectiveness and is therefore the highest-priority research dependency.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.5 (Trace Semantics Engine) | Component definition, inputs/outputs, role in analysis pipeline |
| ARCHITECTURE.md | 5.3 (Fault Isolation Decision Tree) | Three-stage audit the IR must support: implementation, methodology, theory |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: High. IR design is unsolved. DSL constraint improves tractability. |
| VISION.md | Open Question #1 | "Semantic Language of Failure" — building the IR is a critical research problem |
| VISION.md | Section 4.1 (LFI) | LFI requires trace logs parseable into causal narratives |
| Constraint | DSL-Only Environments | IR design is bounded to structured DSL output, not arbitrary Python |

## Status

IN PROGRESS — Next Step 2 complete (RCA/formal verification IR survey). Next Steps 1, 3, 4, 5 remain.

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### Entry 1 — 2026-02-20: RCA and Formal Verification IR Survey

**Scope.** Next Step 2: Survey existing IR designs in RCA and formal verification. Identify design patterns that transfer to ATHENA's trace semantics problem (translating DSL trace logs into structured representations for three-stage fault classification).

**Method.** Surveyed intermediate representations across four categories:
1. **LLM-based RCA:** arxiv:2403.04123 (LLM agents for RCA), arxiv:2601.22208 (reasoning failures in LLM RCA).
2. **Structured RCA:** Chain-of-Event (FSE 2024, typed event chains for microservice RCA), Jaeger/Zipkin (OpenTelemetry span-based distributed tracing).
3. **Formal verification IRs:** LLVM IR (SSA form), MLIR (multi-level dialect system), Boogie (specification-implementation contracts), Why3 (ghost state, theories, refinement), DRAT (machine-checkable refutation proofs), AIGER (counter-example witness traces).
4. **Program analysis:** Clang Static Analyzer (path-sensitive bug reports), Soot/WALA (JVM analysis IRs), Facebook Infer (compositional bi-abductive analysis).

Evaluated each IR against: spec-vs-execution separation, causal ordering representation, queryability, root cause ranking, and compatibility with Rust zero-copy/streaming parsing. Produced a transferable patterns catalog (13 patterns), an anti-patterns catalog (6 anti-patterns), and a prioritized recommendation.

**Findings.**

*Primary structural insight:* MLIR's dialect system is the most directly transferable pattern. It maps naturally to ATHENA's core requirement of separating theory-layer and implementation-layer trace events. Defining three dialects (theory, methodology, implementation) would give the LFI structural routing to the correct audit stage. The multi-level coexistence property means a single IR can carry all three layers simultaneously, linked by explicit lowering relationships that encode how theory-level specifications were realized by implementation-level execution.

*Second key insight:* Boogie/Why3 specification-implementation contracts provide the mechanism for the LFI's sequential audit. An experiment specification becomes a contract (requires/ensures/modifies). Stage 1 checks execution against implementation-level contract terms. Stage 2 checks whether the contract is adequate to test the hypothesis. Stage 3 checks whether contract-satisfying execution contradicts predictions. This three-level contract checking maps to the three-stage audit in ARCHITECTURE.md 5.3.

*Third key insight:* The failure modes cataloged in arxiv:2601.22208 (Stalled, Biased, Confused) map directly to IR requirements. "Stalled" requires explicit observability boundaries (Boogie's `havoc` for unobserved state). "Biased" requires evidence-backed causal chains (CoE's typed event chains with evidence links). "Confused" requires isolation of parallel causal paths within the IR structure.

*Negative finding:* LLM-based RCA systems (arxiv:2403.04123) use no formal IR — chain-of-thought reasoning serves as an implicit, non-queryable, non-reproducible "representation." The ~21% Top@1 accuracy on unstructured traces is consistent with this architectural limitation. The absence of a formal IR is the root cause of low accuracy, not insufficient LLM capability.

*Streaming compatibility:* All 13 identified transferable patterns are compatible with Rust zero-copy/streaming parsing. The three primary patterns (dialects, contracts, typed event chains) are particularly efficient: dialect tags are enum variants, contracts are parsed once from experiment specifications, and event chains are constructed incrementally.

*Anti-pattern identification:* Six anti-patterns identified, with "specification-implementation conflation" (AP2) as the most critical to avoid — it would directly disable the three-stage audit.

**Implications.**

1. The IR design is not a blank-slate research problem. Three well-established patterns from formal verification (MLIR dialects, Boogie contracts, Why3 ghost state) provide structural foundations. The research challenge is adapting these patterns to empirical trace analysis (post-execution, quantitative, streaming) rather than static/deductive verification (pre-execution, logical, batch).
2. The dialect-based layer separation pattern should be the primary structural decision for the IR. It provides the routing mechanism the LFI needs and maps directly to the DSL API separation constraint.
3. The contract pattern resolves a previously implicit requirement: the IR must carry the experiment specification alongside the trace events, as a first-class entity. Without this, Stage 2 (methodology audit) and Stage 3 (theoretical evaluation) cannot function.
4. Six open questions identified for subsequent investigations (see survey document Section 7).

**Open Threads.**

- Dialect boundaries per DSL: How to determine which trace events belong to theory/methodology/implementation for each target DSL. Requires the DSL trace format survey (Next Step 1).
- Contract extraction: Can experiment specification contracts be automatically derived from DSL experiment scripts? Determines practicality of the contract pattern.
- Streaming completeness trade-off: How much trace data must be buffered vs. streamed for each audit stage? Stage 1 may be fully streaming; Stage 3 may require the full trace.
- Quantitative refutation logic: DRAT-style refutation chains need adaptation from propositional to statistical reasoning for Stage 3.
- Ghost state validation: Methodological ghost state (sampling sufficiency, confounder control) depends on DAG quality, connecting to the bootstrapping error risk (ARCHITECTURE.md 8.3).

**Artifact.** `dsl-evaluation/rca-formal-verification-ir-survey.md` — Full survey with 13 transferable patterns, 6 anti-patterns, prioritized recommendations, and open questions.

## Accumulated Findings

### What We Know

1. **LLM-based RCA without formal IR achieves ~21% Top@1 accuracy on unstructured traces.** The root cause is architectural (no structured representation of causal chains), not a capability limitation of the LLMs. [Entry 1; arxiv:2403.04123; ARCHITECTURE.md 4.5]
2. **Three specific LLM RCA failure modes map to IR requirements.** "Stalled" (missing context) requires explicit observability boundaries. "Biased" (prior-dominated reasoning) requires evidence-backed causal chains. "Confused" (conflated causal paths) requires structural isolation of parallel chains. [Entry 1; arxiv:2601.22208]
3. **MLIR's dialect system provides a structural pattern for theory/implementation separation in the IR.** Namespaced operation dialects can represent theory-layer, methodology-layer, and implementation-layer trace events as structurally distinct entity types coexisting in a single IR. This maps directly to the DSL API separation constraint and the LFI's three-stage audit routing. [Entry 1; MLIR documentation]
4. **Boogie/Why3 specification-implementation contracts provide a pattern for the LFI's sequential audit.** Experiment specifications can be represented as contracts (requires/ensures/modifies) checked against trace data at each audit stage. [Entry 1; Boogie, Why3 documentation]
5. **The IR must carry the experiment specification as a first-class entity alongside trace events.** Without the specification, Stage 2 (methodology audit) and Stage 3 (theoretical evaluation) cannot function — the LFI needs to compare what was intended against what executed. [Entry 1; Boogie contract pattern analysis]
6. **All identified transferable patterns are compatible with Rust zero-copy/streaming parsing.** Dialect tags are enum variants, contracts are structured records, event chains are incrementally constructed. No pattern requires full-trace buffering for Stage 1 (implementation audit). [Entry 1; pattern catalog analysis]
7. **Specification-implementation conflation is the most critical anti-pattern to avoid.** Representing "what was specified" and "what executed" in the same namespace directly disables the three-stage audit. [Entry 1; anti-pattern analysis]

### What We Suspect

1. **The dialect boundary definition will be the hardest per-DSL adaptation problem.** Determining which trace events belong to theory, methodology, or implementation for each target DSL (OpenMM, GROMACS, VASP) likely requires deep understanding of each framework's API structure and trace output format. [Entry 1; open question 1]
2. **Stage 3 (theoretical evaluation) may require full-trace buffering, breaking the streaming model.** Theoretical predictions are evaluated against aggregate experimental outcomes, which may require the complete trace. Stages 1 and 2 can likely operate in streaming mode. [Entry 1; open question 5]
3. **Ghost state for methodological metadata inherits DAG quality problems.** If the causal DAG is wrong about confounders, methodological ghost state will encode incorrect confounder-control claims, propagating the bootstrapping error into the IR. This connects to ARCHITECTURE.md 8.3. [Entry 1; open question 4]
4. **Contract extraction from DSL experiment scripts may be partially automatable.** DSL APIs have typed parameter specifications that could serve as automatically derived preconditions. Postconditions (expected outcomes from hypothesis predictions) likely require manual or LLM-assisted specification. [Entry 1; open question 3]

### What We Don't Know

1. **How to adapt quantitative/statistical refutation logic into a machine-checkable chain structure.** DRAT-style refutation chains are propositional; scientific falsification is quantitative and probabilistic. What constitutes a "step" in a scientific refutation chain? [Entry 1; open question 6]
2. **What the actual trace output formats of OpenMM, GROMACS, and VASP look like.** The IR design patterns identified in this survey are structural — their applicability depends on whether real DSL trace data can be mapped into these structures. The DSL trace format survey (Next Step 1) is required to ground the patterns. [Entry 1; open question 1]
3. **What the minimum IR resolution is for each audit stage.** How much trace detail does Stage 1 (implementation audit) need? Stage 2? Stage 3? This is the subject of Next Step 3 (backward mapping from LFI requirements). [Entry 1]
4. **Whether the 21% baseline applies to DSL-constrained environments.** ARCHITECTURE.md 4.5 asserts that accuracy "improves substantially within constrained DSL environments," but this has not been empirically validated in ATHENA's specific context. [Entry 1]
5. **How to handle trace events that span multiple dialects.** Some DSL operations may involve both theory-level and implementation-level concerns simultaneously (e.g., a numerical integrator that embodies both a theoretical choice and an implementation method). The dialect pattern assumes clean separation; reality may be messier. [Entry 1]

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| *None yet* | | | |

## Next Steps

1. **Survey DSL trace formats** — Collect and document the actual trace output structure from OpenMM, GROMACS, and VASP. Identify common elements (timestamped events, state dumps, error codes) and framework-specific elements. Scope: 2-3 sessions.

2. **Survey existing IR designs in RCA and formal verification** — Review intermediate representations used in root cause analysis tools, formal verification (SAT solvers, model checkers), and program analysis. Identify which design patterns transfer to DSL trace parsing. Scope: 2-3 sessions.

3. **Map LFI three-stage audit backwards to minimum IR requirements** — Starting from the three audit stages in ARCHITECTURE.md 5.3, derive the minimum set of semantic distinctions the IR must represent. What must Stage 1 (implementation audit) be able to query? Stage 2 (methodology audit)? Stage 3 (theory evaluation)? Scope: 1-2 sessions.

4. **Characterize the 21% baseline and DSL improvement** — Understand what drives the low accuracy of general RCA on unstructured traces, and what structural properties of DSL environments improve it. Identify the specific features of DSL traces that make them more amenable to semantic parsing. Scope: 1-2 sessions.

5. **Draft candidate IR schemas** — Based on the above, propose 2-3 candidate IR designs with explicit tradeoffs (resolution vs. generality vs. implementation cost). Each should be evaluated against the minimum requirements from step 3. Scope: 2-3 sessions.
