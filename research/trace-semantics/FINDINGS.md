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

IN PROGRESS

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### 2026-02-20 — Provenance Data Models and Scientific Workflow IR Survey

**Scope:** Survey W3C PROV-DM, ProvONE, scientific workflow provenance systems (Kepler, Taverna, VisTrails, Galaxy, CWL), process mining (XES, conformance checking), and provenance query languages (SPARQL over PROV) for applicability to ATHENA's trace semantics IR. Central assessment question: can these models represent the theory-implementation distinction deterministically?

**Method:** Systematic analysis of W3C PROV-DM (§2-5), PROV-O (§3), and PROV-CONSTRAINTS (§5-8) specifications. Mapped PROV-DM's Entity-Activity-Agent model to ATHENA's theory-methodology-implementation trichotomy. Evaluated ProvONE's scientific workflow extensions (Program, Port, Channel, Controller, Workflow) for DSL simulation fit. Assessed provenance query expressiveness (SPARQL path queries for causal chain traversal). Analyzed process mining conformance checking as an expected-vs-actual comparison mechanism. Evaluated scalability at megabyte-scale traces for Rust implementation. Cataloged seven transferable patterns and five anti-patterns.

**Findings:**

1. PROV-DM's Entity-Activity-Agent model provides approximately 60-70% of ATHENA's IR requirements. Entity and Activity map well to simulation states and steps. The Agent model is the weakest mapping — it captures "who is responsible" but not "what kind of responsibility" (theory vs. implementation vs. methodology). This is the central gap.

2. PROV-DM qualified relations (§3) substantially improve resolution. Qualified Usage records *how* entities participate in activities (roles), and qualified Association with Plans provides a mechanism for encoding expected behavior (hypothesis predictions) against which actual execution can be compared. Plans are the closest PROV-DM gets to expected-vs-actual representation, but they are unstructured entities requiring ATHENA-specific formalization.

3. PROV-CONSTRAINTS provides temporal ordering, derivation chain integrity, and uniqueness constraints that can encode *some* LFI audit preconditions (particularly temporal consistency checks for Stage 1 implementation audit). It cannot encode domain-specific constraints (parameter bounds, precision requirements).

4. ProvONE's prospective/retrospective separation is the most directly relevant extension. Prospective provenance (workflow definition) maps to specification; retrospective provenance (execution trace) maps to actual execution. This provides a two-way split (specification vs. execution) rather than ATHENA's required three-way split (theory vs. methodology vs. implementation). The methodology layer is collapsed into the specification layer.

5. ProvONE's typed Ports provide a natural mechanism for parameter classification. Theory-Ports (force field parameters, equation coefficients) vs. implementation-Ports (GPU device, memory allocation) vs. methodology-Ports (sampling frequency, convergence criteria) can structurally encode the three-layer distinction at the API boundary.

6. Process mining conformance checking (alignment-based) is directly relevant to LFI Stages 1 and 2. Comparing expected process models against actual execution traces identifies structural deviations (missing steps, unexpected events) that signal implementation or methodology failures.

7. For Rust implementation: PROV-DM's RDF/SPARQL technology stack is incompatible with the throughput requirement. No mature Rust RDF triple stores exist. However, PROV-DM's *data model* (concepts and relations) can be adopted without its technology stack, implemented as a Rust-native graph structure (petgraph or custom adjacency list) with purpose-built query functions.

8. Scalability assessment: megabyte-scale traces produce 10^4 to 10^6 PROV triples. Custom Rust graph implementations handle this in milliseconds for path queries. RDF triple stores take 10-1000ms. The hybrid approach (PROV-DM concepts in Rust structures) is viable at this scale.

**Implications:**

- Decision Gate 2 outcome: PROV-DM is viable as a *conceptual foundation* but not as a *complete IR*. Three mandatory extensions are needed: (a) three-layer agent/activity/entity typing, (b) fault semantics vocabulary, (c) expected-vs-actual comparison primitives.
- The recommended approach is a hybrid: adopt PROV-DM's data model concepts (Entity, Activity, Agent, derivation chains, qualified relations, Plans) implemented in Rust-native structures, with ATHENA-specific extensions built into the core type system rather than layered as attributes.
- The theory-implementation-methodology distinction must be structural (in the type system), not attributional (in metadata). This is the single most critical design decision for the IR.
- ProvONE's typed Ports provide the most promising mechanism for encoding the three-layer distinction at DSL API boundaries.
- A novel IR designed from scratch would carry higher risk (no existing specification) but could be ATHENA-optimal. The hybrid approach trades some optimality for maturity.

**Open Threads:**

- How should theory/methodology/implementation layer assignments be determined for each DSL's API parameters? This is a per-DSL classification problem that needs investigation.
- Can conformance checking (process mining) be integrated with PROV-DM derivation chains to provide both structural and value-level deviation detection?
- What is the minimum granularity of provenance recording needed for the LFI? Full-granularity is an anti-pattern (10^8+ nodes); DSL-API-call level seems right but needs validation against actual trace data.
- ProvONE's prospective/retrospective split collapses methodology into specification. Can the prospective layer be further split into theory-prospective and methodology-prospective sub-layers? This requires investigation.

**Output:** `dsl-evaluation/provenance-workflow-ir-survey.md` — Complete survey with PROV-DM analysis, ProvONE analysis, query language assessment, workflow system survey, process mining assessment, scalability analysis, expected-vs-actual representation patterns, seven transferable patterns, and five anti-patterns.

## Accumulated Findings

### What We Know

- W3C PROV-DM's Entity-Activity-Agent model provides a mature, formally specified data model for provenance with derivation chains, qualified relations, temporal constraints, and a validation framework. It covers approximately 60-70% of ATHENA's IR requirements. [Log: 2026-02-20, Finding 1; W3C PROV-DM §2-5]
- PROV-DM does not natively represent the theory-implementation-methodology trichotomy. Agents, Activities, and Entities are untyped with respect to the three-layer distinction. The distinction must be added via extension. [Log: 2026-02-20, Finding 1; PROV-DM §2.3]
- PROV-DM qualified relations (specifically qualified Association with Plans) provide the closest mechanism to expected-vs-actual comparison, but Plans are unstructured entities requiring formalization. [Log: 2026-02-20, Finding 2; PROV-DM §3.4]
- PROV-CONSTRAINTS temporal ordering and derivation chain constraints can serve as structural preconditions for LFI Stage 1 (implementation audit). They cannot encode domain-specific constraints. [Log: 2026-02-20, Finding 3; PROV-CONSTRAINTS §5-8]
- ProvONE provides prospective/retrospective provenance separation that maps to specification-vs-execution, a two-way split. ATHENA requires a three-way split. [Log: 2026-02-20, Finding 4; ProvONE specification]
- ProvONE's typed Ports provide a natural mechanism for classifying parameters by layer (theory/methodology/implementation) at DSL API boundaries. [Log: 2026-02-20, Finding 5; ProvONE Port concept]
- Process mining conformance checking (alignment-based) is directly applicable to LFI Stages 1 and 2 for detecting structural deviations between expected and actual execution. [Log: 2026-02-20, Finding 6; van der Aalst 2016]
- The PROV-DM RDF/SPARQL technology stack is incompatible with the Rust throughput requirement. The data model can be adopted without the technology stack. [Log: 2026-02-20, Finding 7; ADR 001]
- At megabyte-scale traces (10^4-10^6 triples), Rust-native graph implementations handle path queries in milliseconds, which is adequate for LFI. [Log: 2026-02-20, Finding 8]
- No existing provenance system (Kepler, Taverna, VisTrails, Galaxy, CWL) natively supports the theory-implementation distinction or fault classification. All provide provenance at the workflow/tool level, not at the DSL-internal semantic level ATHENA requires. [Log: 2026-02-20, workflow survey]

### What We Suspect

- The hybrid approach (PROV-DM data model concepts in Rust-native structures with ATHENA-specific extensions) likely offers the best risk/reward tradeoff for Decision Gate 2. It captures the maturity of a W3C standard without the performance costs of the RDF stack. [Log: 2026-02-20, Implications]
- The theory-implementation-methodology distinction should be structural (in the type system) rather than attributional (in metadata) for the IR to support deterministic fault classification. Attribute-based encoding forces every LFI query to filter by metadata, adding complexity and ambiguity. [Log: 2026-02-20, Anti-Pattern 3]
- ProvONE's prospective layer could potentially be split into theory-prospective and methodology-prospective sub-layers to achieve the three-way split, but this requires investigation and may not be clean for all DSL APIs. [Log: 2026-02-20, Open Threads]
- Selective/adaptive provenance recording (fine-grained only in failure-proximate regions) is likely necessary to avoid the full-granularity anti-pattern at simulation scale. [Log: 2026-02-20, Anti-Pattern 1]

### What We Don't Know

- How to determine theory/methodology/implementation layer assignments for each DSL's API parameters. This is a per-DSL classification problem that has not been investigated yet. [Log: 2026-02-20, Open Threads]
- Whether conformance checking can be integrated with PROV-DM derivation chains to provide both structural and value-level deviation detection simultaneously. [Log: 2026-02-20, Open Threads]
- What the minimum granularity of provenance recording is that still enables correct fault classification by the LFI. This requires validation against actual DSL trace data. [Log: 2026-02-20, Open Threads]
- Whether the three-layer typing can be implemented cleanly in Rust's type system (e.g., enum-based layer tags vs. trait-based hierarchies) without sacrificing graph traversal performance. [Log: 2026-02-20, Finding 7]
- Whether actual DSL trace logs from OpenMM, GROMACS, and VASP contain sufficient information to construct PROV-like derivation chains, or whether significant trace enrichment would be needed. This depends on the DSL trace format survey (Next Step 1). [Log: 2026-02-20]

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
