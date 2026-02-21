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

IN PROGRESS — Steps 1-6 and all synthesis steps (1d, 2c, 3b) complete. Step 5a (candidate IR schemas) complete: Hybrid LEL+DGR recommended (94/100). Step 5b (LEL prototype) complete. Step 5c (open thread resolution) complete: 5/5 threads resolved/narrowed/deferred with evidence. Step 6 (Hybrid LEL+DGR Phase 2 prototype) complete: `by_id` index implemented, `CausalOverlay` + R14 confounder query implemented, crate at 29/29 passing tests, clippy clean.

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### 2026-02-21: Hybrid LEL+DGR Phase 2 Prototype — CausalOverlay + R14 Query (Step 6)

**Scope:** Implement and validate the graph-traversal half of the Hybrid architecture in the Rust prototype: `EventIndexes.by_id`, `CausalOverlay` construction/traversal, and R14 confounder detection over the overlay.

**Method:** Direct implementation in `prototypes/lel-ir-prototype/` following the approved dependency order (Task 1→5): extend indexes, add overlay module with index-only entity mapping, add R14 query method, migrate benchmark to real overlay construction path, and validate each step with `cargo test` + strict clippy.

**Findings:**

1. **`EventIndexes.by_id` is implemented and serialized.** `by_id: HashMap<EventId, usize>` now records event position at insert time. Builder wiring uses `self.events.len()` before push. Added tests for population, position correctness, and serde roundtrip.

2. **`CausalOverlay` now exists as a first-class prototype artifact (`src/overlay.rs`).** Construction is a single O(n) pass with `Vec::with_capacity(n)`, 1:1 entity mapping (`event_idx == log.events index`), `dag_node: Option<String>`, and `causal_parents` resolved through `log.indexes.by_id` using `filter_map` (dangling refs skipped).

3. **Graph traversal APIs are implemented and validated.** Accessors (`len`, `is_empty`, `entity`) plus `transitive_ancestors` (on-demand BFS, start node excluded) are covered by empty, linear-chain, diamond, and dangling-reference tests.

4. **R14 confounder detection query is implemented on the overlay.** `detect_confounders` performs variable existence guard, event-position resolution, transitive ancestor set intersection, controlled/intervention filtering, and dag-node grouping into `ConfounderCandidate` outputs. Added 7 targeted tests (all-controlled, uncontrolled-detected, intervention-excluded, no-common-ancestor, unknown-variable, multiple confounders, transitive chain).

5. **Benchmark now exercises real overlay construction, not ad-hoc HashMaps.** `src/bench.rs` uses `CausalOverlay::from_log(&log)` and reports overlay-backed counts. Observed at 10^6 events: log construction 2130.33ms, overlay construction 251.82ms, 1,000,000 overlay entities, 199,998 derivation edges, 50 DAG-node groups.

6. **Prototype quality gates passed after each task boundary.** Final crate state: 29/29 tests passing, strict clippy (`--all-targets --all-features -- -D warnings`) passes with zero warnings.

**Implications:**
- Hybrid Phase 2 is now concretely prototyped: LEL event stream can be lifted to an index-only causal overlay with O(n) construction and on-demand graph traversal.
- Phase 3 query work is unblocked for confounder-oriented causal analysis (R14 path now executable end-to-end in prototype form).
- Thread #37 is closed (implemented). Thread #38 is narrowed with empirical support: Vec-first allocation remains adequate at current scale; arena remains optional only if future profiling indicates measurable allocation overhead.

**Open Threads:**
- VASP Stage 3 representation gaps remain open (#35 `ContractTerm.value`, #36 matrix/function value support).

---

### 2026-02-21: Open Thread Resolution (Step 5c)

**Scope:** Resolve or narrow 5 open threads from Step 5a (candidate IR schemas) using LEL prototype evidence and analytical reasoning.

**Method:** Empirical benchmark (#31: overlay construction cost), analytical reasoning from prototype evidence (#32: references from day one, #34: OverlayEntity sufficiency), document-driven analysis against DSL survey findings (#33: ExperimentSpec sufficiency), theoretical analysis (#35: arena allocation).

**Findings:**

1. **Thread #31 RESOLVED: Overlay construction cost is empirically bounded.** Benchmark (`src/bench.rs`) measures O(n) HashMap-building pass over synthetic LEL events at 4 scales. Results at 10^6 events: overlay construction 80.53ms, log construction 488.96ms, ~10.7MB overlay memory, ~300K overlay entities, ~200K derivation edges. Linear scaling confirmed: 10^5→10^6 scales ~9x for overlay (8.97→80.53ms). The O(n) pass is tractable for megabyte-scale traces on commodity hardware. [bench.rs benchmark, release mode]

2. **Thread #32 NARROWED: "From day one" is the safer default; deferred resolution is a viable escape hatch.** Prototype evidence: (a) `dag_node_ref`, `spec_ref`, `causal_refs` compile and serialize with None/empty values (`test_hybrid_upgrade_fields_present`, `test_serde_roundtrip`); (b) mock adapter in adapter.rs leaves `dag_node_ref`/`spec_ref` as None — adapters can defer without structural penalty; (c) `EventIndexes.by_dag_node` index populates incrementally during `index_event()` (lel.rs:141-146) — works whether references are upfront or via deferred pass. Deferred resolution is viable via a parallel reference map (`HashMap<EventId, (Option<String>, Option<SpecElementId>)>`) applied at Stage 1→2 boundary as an O(n) pass. Remaining question narrowed to: is the two-phase adapter protocol acceptable complexity for specific adapters? This is an adapter API design decision, not an IR correctness question. [LEL prototype: adapter.rs, lel.rs:141-146, tests]

3. **Thread #33 NARROWED: ExperimentSpec sufficient for all three frameworks at Stage 1; two specific VASP Stage 3 gaps identified.** Analysis against each framework's adapter needs using DSL survey findings: OpenMM — sufficient (`createSystem()` chain is adapter-internal, not spec). GROMACS — sufficient (.mdp → `controlled_variables`/`interventions`, grompp → trace events). VASP — two gaps: (a) `ContractTerm` (common.rs:94-99) has only `description: String`, needs `value: Option<Value>` for machine-readable precondition checking (e.g., POTCAR family = PBE); (b) `PredictionRecord.predicted_value: Value` cannot represent spectral data (band structure over k-points), would need `KnownMatrix` or function variant in `Value` enum. Both gaps are non-blocking for current scope (OpenMM Stage 1). [DSL surveys: OpenMM, GROMACS, VASP; common.rs:94-99, common.rs:102-108]

4. **Thread #34 NARROWED: Lightweight OverlayEntity sufficient for Stage 2-3 queries; one missing index identified.** Analysis of three actual query patterns: R14 (confounder) traverses `causal_ancestors` → common ancestors → `dag_node_ref` against controlled variables → event lookup — OverlayEntity fields sufficient. R17 (comparison) uses `spec_ref` for prediction, `event_id` for observation — sufficient. R18 (causal implication) traverses derivation edges → `dag_node_ref` — sufficient. One gap: `EventIndexes` lacks `by_id: HashMap<EventId, usize>` for O(1) event lookup by ID. Currently `events` is `Vec<TraceEvent>` with no ID→index mapping. OverlayEntity's `event_id` field requires this to avoid O(n) linear search. Small addition (~8 bytes/event). [LEL prototype: lel.rs:88-96 EventIndexes, lel.rs:51-85 TraceEvent]

5. **Thread #35 DEFERRED with concrete guidance: Vec-first, benchmark at Phase 2.** The Hybrid's overlay construction is a single batch O(n) pass — all OverlayEntities allocated in one sweep. For batch allocation, `Vec<OverlayEntity>` with `Vec::with_capacity(n)` achieves the same cache locality as an arena allocator. Arena provides benefit only when allocations are interleaved with other work (preventing heap fragmentation) — not the Hybrid's pattern. Recommendation: start with Vec, benchmark at 10^6 scale during Phase 2, add arena crate (`bumpalo` or `typed-arena`) only if allocation overhead is measurable. [Theoretical analysis; benchmark confirms batch pattern at scale]

6. **Benchmark artifact produced.** `src/bench.rs` as `[[bin]]` target, zero new dependencies (uses `std::time::Instant`). Tests construction at 4 scales (10^3, 10^4, 10^5, 10^6) with realistic event distributions (70/20/10 layer split, 30% `dag_node_ref`, 10% `causal_refs`). Reports wall-clock time, entity/edge counts, memory estimates. [bench.rs]

**Implications:**
- All 5 open threads from Step 5a are now resolved (1), narrowed (3), or deferred with concrete guidance (1). No thread remains open-ended.
- The Hybrid LEL+DGR architecture's key performance claim (O(n) overlay construction at megabyte scale) is now empirically validated.
- Two concrete tasks for future work identified: (a) add `by_id: HashMap<EventId, usize>` to `EventIndexes` for Phase 2 CausalOverlay; (b) add `value: Option<Value>` to `ContractTerm` for VASP Stage 3.
- The deferred reference resolution strategy provides a viable escape hatch for adapters where per-event entity resolution is costly, without requiring IR structural changes.
- Arena allocation is deferred with clear trigger: benchmark Vec at Phase 2 scale; adopt arena only if measurable overhead.

**Open Threads:**
- None. All threads resolved, narrowed, or deferred with concrete guidance.

---

### 2026-02-20: Candidate IR Schemas with Hybrid LEL-DGR Recommendation (Step 5a)

**Scope:** Synthesize all accumulated evidence (R1-R29 requirements, coverage matrix, pattern catalog, cross-framework synthesis) into concrete IR schema designs. Evaluate candidates against requirements, anti-patterns, streaming constraints, and stage-specific performance. Produce a recommendation for Step 5b prototyping.

**Method:** Schema design driven by three inputs: (1) requirements-coverage-matrix.md (R1-R29 coverage codes, gap analysis, three-input data flow architecture), (2) ir-pattern-catalog.md (7 transferable patterns, 9 anti-patterns, candidate previews, unified architecture), (3) cross-framework-synthesis.md (adapter contract, failure modes, boundary parameters). Candidates evaluated against a 7-criterion weighted framework (R1-R29 coverage 25%, anti-pattern compliance 20%, streaming 15%, Stage 1 efficiency 15%, Stage 2-3 causal reasoning 15%, implementation complexity 5%, incremental adoptability 5%). Key design decision: 2 candidates + 1 hybrid, replacing TAL standalone with LEL-DGR Hybrid based on coverage matrix conclusions.

**Findings:**

1. **TAL deferred to query-layer role, replaced by LEL-DGR Hybrid.** The coverage matrix (requirements-coverage-matrix.md §8) concluded TAL "works better as a query interface layer than a data representation." TAL has the highest novelty risk (no close precedent), weakest causal graph traversal support, and its core strength (sequential assertion checking) functions identically as a query interface over LEL or DGR substrates. TAL's assertion-checking pattern is preserved as the recommended LFI query interface. The Hybrid candidate addresses open questions #2 (incremental path) and #5 (LEL→DGR viability), which are more architecturally informative than a high-risk novelty candidate. [candidate-ir-schemas.md §0]

2. **A common structural foundation shared by all candidates was defined.** Seven shared types: `Layer` enum (Theory/Methodology/Implementation) for R19, `BoundaryClassification` enum (PrimaryLayer/DualAnnotated/ContextDependent) resolving OQ4, `ObservationMode` enum for R28, `Value` enum with `Havoc` variant (Boogie P6) for R26, `TemporalCoord` struct (simulation_step/wall_clock_ns/logical_sequence) for R21, `ProvenanceAnchor` struct for R20, `ExperimentRef` struct for R22/R29, and `ConfidenceMeta` struct for R25. These types ensure consistent semantics regardless of which candidate is chosen. [candidate-ir-schemas.md §1]

3. **LEL (Layered Event Log) scores 82/100.** STRONG for Stage 1 (7/7 requirements), streaming (pure append-only), and implementation simplicity. WEAK for R14 (confounder query — requires multi-way joins unsupported by flat log) and R18 (causal implication — requires transitive causal ancestry unsupported without graph traversal). PARTIAL on AP7 (implicit causal ordering — causal_refs are optional best-effort). [candidate-ir-schemas.md §2, §5, §6, §9]

4. **DGR (Dual-Graph IR) scores 82/100.** STRONG for all R1-R29 requirements (full coverage including R14 and R18 via graph traversal) and Stages 2-3 causal reasoning. PARTIAL on AP2 (post-mortem-only — spec_graph pre-built before trace, acceptable). MODERATE for streaming (graph construction from streaming data requires forward-reference management) and Stage 1 efficiency (graph construction overhead for the most common classification path). Same total as LEL but with inverted strengths/weaknesses. [candidate-ir-schemas.md §3, §5, §6, §9]

5. **Hybrid (LEL core + DGR overlay) scores 94/100.** Captures LEL's strengths (streaming, Stage 1 efficiency) and DGR's strengths (causal reasoning, R1-R29 coverage). PASS on all 9 anti-patterns (the only candidate with no PARTIAL ratings). Stage 1 operates as pure LEL (append-only, early termination if implementation fault found). CausalOverlay built at Stage 1→2 boundary via single O(n) pass over events. Key constraint: LEL events must carry `dag_node_ref` and `spec_ref` from the start to enable overlay construction. [candidate-ir-schemas.md §4, §5, §6, §9]

6. **R17 (prediction-observation comparison) resolved structurally via ComparisonResult + DivergenceMeasure.** The IR provides a structural container with six divergence measure variants (AbsoluteDifference, ZScore, BayesFactor, KLDivergence, EffectSize, Custom). The comparison method is pluggable — the IR stores results, not logic. The LFI selects the appropriate measure per prediction type. The comparison formalization research question is now scoped to LFI logic, not IR structure. [candidate-ir-schemas.md §3, §8 OQ1]

7. **The LEL→DGR incremental path is viable.** The Hybrid demonstrates viability by construction. Key constraint identified: LEL events must include DGR-compatible references (dag_node_ref, spec_ref, causal_refs) from day one. If these are omitted during initial implementation, overlay construction requires re-parsing. Implication for Step 5b: the LEL prototype must include these fields even though Stage 1 does not use them. [candidate-ir-schemas.md §4, §8 OQ2]

8. **Causal reasoning substrate is per-stage.** Stage 1: sequential search sufficient (filter-and-inspect on implementation-tagged events, O(n) with early termination). Stages 2-3: graph traversal required (transitive causal ancestry for R14 confounder queries, structural path finding for R18 causal implication). This per-stage answer directly motivates the Hybrid design. [candidate-ir-schemas.md §8 OQ3]

9. **BoundaryClassification enum resolves the boundary parameter representation question.** Three variants: PrimaryLayer (unambiguous), DualAnnotated (primary layer for routing + secondary layer annotation, e.g., GROMACS dt), ContextDependent (default layer + context note, e.g., VASP ALGO). Avoids both a fourth "boundary" layer and entity duplication. [candidate-ir-schemas.md §1, §8 OQ4]

10. **Step 5b recommendation: LEL prototype on OpenMM traces.** Scope: R1-R7 + R19 + R20 + R21. Target: validate event typing, layer tagging, specification separation with minimal complexity. Critical: include dag_node_ref/spec_ref/causal_refs fields for Hybrid upgrade path. Evolution: LEL → Hybrid (overlay) → full DGR as Stages 2-3 mature. [candidate-ir-schemas.md §10]

**Implications:**
- The IR design question is now resolved to a recommended architecture (Hybrid LEL+DGR) with a concrete prototyping plan (LEL first, OpenMM target).
- Step 5b can proceed immediately with a well-scoped prototype: LEL core on OpenMM traces, Stage 1 requirements only, with Hybrid upgrade path preserved.
- The common structural foundation (Section 1 types) should be implemented first and shared across any candidate, ensuring consistent semantics regardless of which IR representation is used.
- The R17 comparison formalization is now scoped to LFI logic, not IR structure — it can be researched independently of IR prototyping.
- TAL as a query-layer interface should be designed alongside the LFI, not as an IR component.

**Open Threads:**
- DGR overlay construction cost at the Stage 1/2 boundary for megabyte-scale traces (10^5-10^6 events). The O(n) pass is theoretically fast but untested empirically. Performance validation needed during Step 5b or an early Hybrid prototype.
- Whether HybridIR events need full DGR-compatible references (dag_node_ref, spec_ref, causal_refs) from day one, or whether a deferred reference-resolution pass is acceptable. The current recommendation is "from day one" for safety, but this pushes entity resolution complexity into the adapter during Stage 1, when it's not needed.
- Arena allocation strategy for the CausalOverlay. The overlay entities reference back to LEL events — the allocation pattern and cache friendliness of this indirection need benchmarking.
- Whether the ExperimentSpec struct is sufficient for all three frameworks or whether framework-specific extensions are needed. The current design is generic; adapter-specific spec fields may be needed.
- The OverlayEntity is lightweight (wraps an LEL event reference + graph relationships). Whether this indirection is sufficient for Stage 2-3 queries or whether richer overlay entities (carrying computed fields, derived values) are needed.

---

### 2026-02-20: Requirements Coverage Matrix and Gap Analysis (Step 3b)

**Scope:** Cross-reference R1-R29 requirements against the trace capability matrix from Step 1d (cross-framework-synthesis.md). For each requirement × framework cell, classify data availability using six codes (DA/DI/ER/FU/NT/DE). Perform gap analysis for all non-DA cells. Assess per-stage feasibility. Evaluate Decision Gate 4.

**Method:** Systematic assessment of each R1-R29 requirement against OpenMM, GROMACS, and VASP trace capabilities. Evidence drawn from cross-framework-synthesis.md (trace capability matrix §1, boundary assessment §2, failure modes §3, completeness §4, adapter contract §5.3, coverage implications §7.1), ir-pattern-catalog.md (pattern coverage annotations, candidate designs §6), and evaluation/hidden-confounder/README.md (R27-R29 context). Requirements assessed in order: Stage 1 (R1-R7), Cross-cutting (R19-R29), Stage 2 (R8-R14), Stage 3 (R15-R18). Each cell classified with code + evidence note + confidence + source reference.

**Findings:**

1. **Stage 1 (R1-R7) is fully satisfiable for all three frameworks.** OpenMM requires the most custom instrumentation (4 DI cells vs. 0 for GROMACS/VASP) because it lacks built-in parameter echo and requires API queries for specification/resource data. GROMACS has the best default Stage 1 coverage (5 DA cells). VASP has good structured output but exit code unreliability for SCF non-convergence (R1 caveat). [requirements-coverage-matrix.md §1]

2. **31% of requirements (9 of 29) are NT — external to the Trace Semantics Engine.** R9, R10, R11, R15, R18, R22, R23, R28 come from experiment specification, hypothesis, or DAG. R29 (cycle_id) comes from the workflow controller. This confirms the IR is a composite multi-source structure, not a pure trace-log derivative. [requirements-coverage-matrix.md §5.1 Strategy C]

3. **R19 (layer tag) has the widest framework variance: OpenMM=DA, GROMACS=DI+ER, VASP=ER.** OpenMM's API-enforced boundary yields clean layer tags. GROMACS needs a moderate classification table (~10 boundary params). VASP needs an extensive table (~200-300 INCAR tags) with context-dependent ambiguity for ~5-10 tags. This is the only cross-cutting requirement with framework-dependent difficulty. [requirements-coverage-matrix.md §2]

4. **Stage 2 (R8-R14) is the weakest stage, limited by external context rather than trace data.** The IR contributes only R8 (observable values) and R12 (sampling metadata) to Stage 2. The remaining 5 requirements are NT (from experiment spec/hypothesis/DAG) or DE (computed from other elements + DAG). This is consistent with the accumulated finding that methodology failures are invisible to all frameworks. [requirements-coverage-matrix.md §3]

5. **Stage 3 (R15-R18) is feasible but blocked on one research element.** R17 (prediction-observation comparison) is DE (computable) but the quantitative comparison method — effect size measures, divergence metrics, tolerance thresholds for scientific predictions — is novel research not yet formalized. All other Stage 3 requirements are satisfiable. [requirements-coverage-matrix.md §4]

6. **FU cells are narrowly scoped and below the 10% threshold.** No full requirement is FU for any framework. Partial FU exists only for R6 (sub-component numerical internals) in all three frameworks: OpenMM ~5% (GPU precision), GROMACS ~5% (constraint solver internals), VASP ~5-10% (FFT/PAW internals). The surface-level metrics are available (DA/DI). [requirements-coverage-matrix.md §5.1 Strategy E]

7. **Decision Gate 4: PASS.** No LFI stage has FU requirements blocking >10% of expected failure classifications. Four conditions: (a) OpenMM custom reporter required, (b) VASP INCAR classification table required, (c) VASP 20-30% degraded confidence for ambiguous params accepted per DG1, (d) R17 comparison method requires formalization. [requirements-coverage-matrix.md §7]

8. **DGR (Dual-Graph IR) is the recommended primary candidate for Step 5a.** The coverage matrix reveals the IR is fundamentally a three-input composite (trace data + external context + domain rules). DGR's graph structure naturally represents entities from all three sources with qualified relationships. LEL is strongest for Stage 1 (high DA density). TAL works better as a query interface layer than as a standalone IR. [requirements-coverage-matrix.md §8]

**Implications:**
- Step 5a (candidate IR schemas) is now unblocked. The coverage matrix provides: (a) concrete data availability per requirement per framework, (b) gap fill strategies with complexity estimates, (c) the three-input data flow architecture as an organizing principle, (d) candidate-specific coverage pattern analysis.
- The IR's three-input architecture (trace + external + domain rules) should be the organizing principle for candidate evaluation, not just trace parsing capability.
- OpenMM adapter requires the most engineering (10 DI cells) but has the cleanest structural foundation (R19=DA). VASP adapter requires the most domain knowledge (R19=ER, ~200-300 tag table) but has good default trace output.
- The prediction-observation comparison formalization (R17) is a discrete, well-scoped research problem that should be elevated as a prerequisite for Stage 3 capability.

**Open Threads:**
- Per-force-group energy decomposition overhead in OpenMM (R6 DI) — untested, affects custom reporter design decisions. [What We Don't Know #2]
- Quantitative prediction-observation comparison method — the single unresolved research element blocking Stage 3. Related to DRAT propositional-to-statistical adaptation. [ir-pattern-catalog.md §7 Open Thread]
- Whether the LEL→DGR incremental path is viable — start with LEL for Stage 1 prototype, evolve toward DGR. Depends on whether adding graph structure is incremental or requires redesign. [ir-pattern-catalog.md §7 Question 5]
- VASP INCAR classification table completeness and validation — needed before VASP adapter design. [cross-framework §6.4]

---

### 2026-02-20: Comparative IR Synthesis (Step 2c)

**Scope:** Synthesis of RCA/formal verification IR survey and provenance/workflow IR survey into a unified pattern catalog. Resolution of the MLIR-dialects vs. PROV-DM-hybrid tension. Decision Gate 2 assessment.

**Method:** Systematic comparison of 20 patterns across both surveys, distilled into 7 pattern categories. Each pattern evaluated against LFI audit stage requirements, R1-R29 coverage, and Rust/streaming compatibility. Anti-patterns cataloged from both surveys with severity ratings. Tension resolution through compositional analysis (MLIR for routing, PROV-DM for provenance).

**Findings:**

1. **Seven transferable pattern categories identified with stage mappings.** Counter-example traces (MEDIUM), Entity-Activity-Agent (HIGH data model / LOW tech stack), typed event chains (HIGH), SSA data flow (MEDIUM-HIGH), multi-level dialects (HIGH), spec-implementation contracts (HIGH), causal dependency/conformance (MEDIUM). Patterns 5 (dialects) and 6 (contracts) are the highest-transferability patterns. [ir-pattern-catalog.md §1]
2. **Stage 2 (methodology audit) is the weakest stage across all patterns.** No surveyed system provides native methodology adequacy checking. Patterns provide structural scaffolding for encoding methodology metadata, but domain-specific adequacy rules are external to IR design. This is consistent with the DSL trace finding that methodology failures are invisible to all frameworks. [ir-pattern-catalog.md §2]
3. **MLIR dialects and PROV-DM are complementary, not contradictory.** Dialects answer "WHERE does an element belong?" (classification/routing). PROV-DM answers "HOW are elements causally related?" (causal structure). The unified architecture uses dialect structure as primary organization with PROV-DM-like causal graphs within each layer. [ir-pattern-catalog.md §4]
4. **Decision Gate 2: Hybrid adaptation, MEDIUM risk.** ~65-70% transfers from existing systems (12 specific patterns). ~30-35% requires novel design: three-way layer typing vocabulary, fault classification ontology, quantitative prediction-observation comparison formalization, methodology detection rules. [ir-pattern-catalog.md §5]
5. **Nine anti-patterns cataloged with avoidance guidance.** CRITICAL: specification-implementation conflation. HIGH: post-mortem-only design, full-granularity recording, binary pass/fail, lossy compression without principled selection. [ir-pattern-catalog.md §3]
6. **Three candidate IR designs mapped to pattern sources.** LEL (Layered Event Log) is simplest, strongest for Stage 1. DGR (Dual-Graph IR) is the natural synthesis of both surveys, strongest for Stages 2-3. TAL (Typed Assertion Log) is most ATHENA-specific and highest-novelty-risk. [ir-pattern-catalog.md §6]

**Implications:**
- The IR structural foundation is now defined: MLIR-style dialect tags for layer routing + PROV-DM-inspired causal graphs within layers + Boogie-style contracts for specification.
- The technology stack is resolved: Rust-native implementation, no RDF/SPARQL, per ADR 001.
- Four novel elements flagged as requiring original research (not available from surveyed systems).
- Step 5a (candidate IR schemas) can now proceed with clear structural foundation and pattern-to-candidate mapping.

**Open Threads:**
- Quantitative prediction-observation comparison formalization — DRAT is propositional, scientific refutation is quantitative. Bridging mechanism undefined.
- How to handle events spanning multiple dialects simultaneously (e.g., VASP's PREC parameter).
- Which causal reasoning substrate (log search, graph traversal, assertion chains) best matches LFI's actual query patterns — requires enumeration of specific queries derived from R1-R29.
- Whether the unified architecture can be incrementally implemented (start with LEL, evolve toward DGR).

---

### 2026-02-20: Cross-Framework Trace Synthesis (Step 1d)

**Scope:** Comparative analysis of OpenMM, GROMACS, and VASP trace output systems. Production of trace capability matrix, failure mode taxonomy, trace completeness assessment, and Decision Gate 1 assessment.

**Method:** Systematic cross-referencing of the three DSL trace analysis documents. Seven trace element categories compared across frameworks with format, access method, and layer classification. Failure modes harmonized into a cross-framework taxonomy. Theory-implementation boundary assessed per-framework with boundary parameter catalog.

**Findings:**

1. **Trace capability matrix completed across 7 categories.** State snapshots, energy series, convergence metrics, error/warning messages, parameter echo, execution metadata, and trajectory data compared across all three frameworks with specific file formats, access methods, and layer tags. [cross-framework-synthesis.md §1]
2. **Theory-implementation boundary: OpenMM CLEAN, GROMACS SEMI-CLEAN, VASP DIRTY.** OpenMM has API-enforced separation at `ForceField.createSystem()`. GROMACS has .mdp separation with ~10 boundary parameters (dt, nsteps, rlist, etc.) requiring dual-annotation. VASP has flat INCAR namespace requiring external classification of ~200-300 tags. Twenty boundary parameters cataloged across all three frameworks. [cross-framework-synthesis.md §2]
3. **49 failure modes taxonomized across three frameworks.** OpenMM: 17 modes (5 impl, 5 methodology, 4 theory, 3 ambiguous). GROMACS: 16 modes. VASP: 16 modes. 8 common cross-framework patterns identified (numerical overflow, constraint/convergence failure, memory exhaustion, etc.) plus 7 DSL-specific modes. [cross-framework-synthesis.md §3]
4. **Trace completeness varies: OpenMM 30-40% default / 70-80% max, GROMACS 60-70% / 75-85%, VASP 50-60% / 50-60% ceiling.** VASP hits a hard ceiling due to closed-source constraints. All frameworks require custom instrumentation for three-way fault classification. [cross-framework-synthesis.md §4]
5. **Seven common IR core elements generalize across all frameworks:** timestamped events, energy time series, parameter records, error events, state snapshots, convergence trajectories, data absence records. Framework-specific elements require DSL-specific adapters. [cross-framework-synthesis.md §5]
6. **Decision Gate 1: VASP FAILS the clean-boundary test but should be accepted.** External classification table is finite, static, and domain-knowledge-based (not novel research). Dropping VASP loses the DFT domain. 70-80% of standard VASP calculations classifiable with full confidence; 20-30% have degraded confidence from ambiguous parameters. Five items flagged for adversarial review. [cross-framework-synthesis.md §6]
7. **Adapter contract defined: 7 mandatory + 7 optional methods.** The adapter interface establishes the boundary between DSL-specific parsing and common IR construction. [cross-framework-synthesis.md §5.3]

**Implications:**
- The IR cannot be a universal schema — it must be common core + adapter extensions.
- The temporal axis must be generic (step intervals for MD, SCF/ionic iterations for DFT).
- Error classification requires IR-imposed taxonomy through pattern matching (no framework provides structured error codes).
- Crash-state is unreliable across all frameworks — IR must work with "last known state" semantics.
- Step 3b (requirements refinement) can now cross-reference R1-R29 against the trace capability matrix.

**Open Threads:**
- INCAR classification table needs domain expert review before VASP adapter design is finalized.
- The "ambiguous for pathological systems" threshold for VASP parameters (ALGO, PREC) needs empirical validation.
- Whether classification tables can be partially automated (LLM-assisted documentation analysis) or are inherently manual — affects ATHENA's generalizability claim.
- Closed-source ceiling practical impact needs stress-testing with real VASP failure cases.

---

### 2026-02-20: 21% RCA Baseline Characterization

**Scope:** Source tracing of the 21% Top@1 figure cited in VISION.md Open Question #1; analysis of structural properties that improve RCA accuracy; assessment of transferability to DSL-constrained environments.

**Method:** Literature review of LLM-based and traditional RCA evaluation papers. Web access (WebFetch, WebSearch, curl) was unavailable during this session. Findings below draw on training knowledge of the RCA literature through early 2025. All claims are labeled by evidence quality: **(A)** = number from a specific paper with dataset and methodology identified, **(B)** = estimate extrapolated from training knowledge of multiple sources, **(C)** = speculation or inference without direct evidence. A follow-up session with web access is needed to verify specific numbers against primary sources.

**Findings:**

#### Source of the 21% Figure

The 21% Top@1 figure in VISION.md (line 129) is **uncited**. The sentence reads: "While state-of-the-art root cause analysis models achieve a mere 21% Top@1 accuracy on general, unstructured execution traces, this accuracy improves substantially within constrained environments." Unlike most other claims in VISION.md, this sentence carries no reference number. This is itself a significant finding: the anchoring number for ATHENA's value proposition is unsourced in the document.

**Candidate source papers (from training knowledge):**

1. **"Exploring LLM-based Agents for Root Cause Analysis" (arxiv:2403.04123, Roy et al., 2024).** **(B)** This paper evaluates LLM-based agents on RCA tasks in cloud/microservice environments. It uses the RCACopilot benchmark and related AIOps datasets. The paper reports that LLM agents struggle significantly on unstructured, heterogeneous incident data, with Top@1 accuracies in the low-to-mid 20% range on the hardest configurations. The 21% figure is plausibly derived from this paper or its evaluation context, though I cannot confirm the exact number without web access. The domain is cloud operations / AIOps, not scientific computing.

2. **"Empowering Practical Root Cause Analysis by Large Language Models for Cloud Incidents" (Microsoft Research, Li et al., 2024).** **(B)** This paper introduces RCACopilot and evaluates GPT-4-based RCA on real Microsoft cloud incidents. It reports varying accuracy across incident categories, with some categories showing Top@1 accuracy in the 20-30% range when the candidate set includes all possible root causes (not a small pre-filtered set). The unstructured nature of cloud incident logs -- mixing free-text alerts, metrics, and heterogeneous telemetry -- is a key difficulty driver.

3. **"Stalled, Biased, and Confused: LLMs for Root Cause Analysis" (arxiv:2601.22208, 2025/2026).** **(B)** This more recent paper systematically evaluates LLMs on RCA benchmarks and finds that models frequently stall, exhibit positional bias in candidate ranking, and produce confused reasoning chains on unstructured traces. Based on training knowledge, this paper likely reports Top@1 numbers in the 15-30% range depending on model and dataset, consistent with the 21% figure but I cannot confirm a specific 21% number.

4. **"Chain-of-Event: Interpretable Root Cause Analysis for Microservices through Automatically Learning Weighted Event Causal Graph" (FSE 2024).** **(B)** This paper works on microservice failure RCA using event-based causal graphs. It provides baseline comparisons where non-graph-based methods achieve low accuracy on complex failure scenarios. The structured graph approach improves significantly over unstructured baselines.

**Assessment:** The 21% figure most likely originates from evaluations of LLM-based RCA on cloud/microservice incident datasets (AIOps domain), where incident reports combine free-text descriptions, heterogeneous log fragments, metric anomalies, and alert streams. The specific number may come from the RCACopilot benchmark or a related Microsoft/cloud operations evaluation. **(C)** It may also be a rounded or representative number synthesized from multiple papers rather than a single precise measurement.

**What "Top@1" means in this context:** **(B)** In RCA benchmarks, Top@1 (also written Top@1 or A@1) means the model's highest-ranked root cause candidate matches the ground-truth root cause. The candidate set size varies dramatically across benchmarks:
- In cloud incident RCA (likely source domain), the candidate set can range from ~20 to 500+ possible root causes (services, components, configuration changes, etc.)
- Top@1 out of 20 candidates (~5% random baseline) is fundamentally different from Top@1 out of 500 candidates (~0.2% random baseline)
- The 21% figure, if from cloud/AIOps, likely operates over a candidate set of 50-200+ root causes **(C)**, making 21% approximately 10-40x above random chance -- not negligible, but far from usable for autonomous decision-making.

#### Why Unstructured Traces Are Hard

The following properties of unstructured execution traces degrade RCA accuracy, ranked by estimated impact:

1. **Free-text mixing and heterogeneous formats (Impact: Critical).** **(B)** Cloud/AIOps traces interleave natural language alerts, JSON-structured metrics, stack traces, configuration diffs, and human-written incident notes. No consistent schema governs what information appears where. LLMs must parse multiple formats simultaneously, and critical causal information can be buried in any format. Source: consistent finding across RCACopilot evaluations and AIOps benchmark papers.

2. **Missing causal ordering (Impact: Critical).** **(B)** Timestamps in distributed systems are unreliable (clock skew, batched logging, asynchronous propagation). Events that are causally related may appear out of order, or causal relationships may not be inferrable from timestamps alone. Without reliable causal ordering, the model cannot distinguish cause from effect from coincidence. Source: distributed systems observability literature; explicitly discussed in microservice RCA papers.

3. **Log spam and signal-to-noise ratio (Impact: High).** **(B)** Production systems generate enormous volumes of routine log entries. The causally relevant entries for any particular failure are a tiny fraction of the total trace. Alert fatigue and log flooding mean the model must find a needle in a haystack. Studies of cloud incident logs show signal-to-noise ratios of 1:100 to 1:10000 for relevant log lines. Source: AIOps and log analysis literature.

4. **Ambiguous error messages (Impact: High).** **(B)** Error messages in general-purpose systems are often generic ("connection timed out", "internal server error", "null pointer exception") and do not encode the causal mechanism of the failure. The same error message can arise from dozens of different root causes. Without domain-specific error taxonomies, the model must disambiguate based on context that is often absent. Source: common finding in incident analysis research.

5. **Missing context and incomplete observability (Impact: High).** **(B)** Real-world traces frequently lack the information needed to identify root causes: uninstrumented services, swallowed exceptions, missing metrics, network partitions that prevent log delivery. The model reasons from incomplete evidence. Source: VISION.md Section 6.4 explicitly identifies this as an architectural risk.

6. **No layer separation (Impact: Medium-High).** **(B)** In unstructured environments, there is no API-enforced distinction between theory-layer and implementation-layer operations. A Python traceback mixes framework internals, library calls, user code, and OS-level errors in a single stack. Determining which layer is responsible requires understanding the entire software stack. Source: this is exactly the problem ATHENA's DSL constraint addresses; discussed in the AIOps RCA context as "blast radius" determination difficulty.

7. **Absence of severity/priority taxonomies (Impact: Medium).** **(B)** Unstructured traces often lack consistent severity labels. A warning might be more important than an error in context, but without a taxonomy, the model treats all events as equally weighted or falls back on keyword heuristics. Source: log analysis and anomaly detection literature.

8. **Temporal coupling without causal coupling (Impact: Medium).** **(B)** In distributed systems, failures cascade rapidly. Events that are temporally proximate may have no causal relationship (independent failures coinciding), or a single root cause may produce effects with variable delay. Temporal proximity is a misleading heuristic. Source: microservice failure analysis papers.

#### Structural Properties That Improve Accuracy

From the literature, the following structural properties improve RCA accuracy when present in trace data:

| Property | Evidence Source | Estimated Improvement | Mechanism |
| :--- | :--- | :--- | :--- |
| **Temporal/causal ordering** | **(B)** Microservice tracing papers (Jaeger, Zipkin-based studies); Chain-of-Event (FSE 2024) | +15-25% Top@1 over unstructured baselines | Eliminates reverse-causation and coincidence hypotheses; enables chain reconstruction |
| **Event type taxonomies** | **(B)** RCACopilot evaluation categories; structured incident management systems | +10-20% Top@1 | Reduces ambiguity by pre-classifying events into semantic categories (error, state change, metric anomaly, etc.) |
| **Schema conformance** | **(B)** OpenTelemetry-based RCA studies; structured logging research | +10-20% Top@1 | Enables systematic parsing; eliminates free-text ambiguity; every field has defined semantics |
| **Causal annotations / dependency graphs** | **(B)** Chain-of-Event; service dependency graph-based RCA | +20-35% Top@1 over non-graph methods | Directly encodes which components can affect which; constrains the search space for root causes |
| **Severity levels** | **(B)** Incident management literature | +5-10% Top@1 | Enables prioritized attention; distinguishes critical signals from informational noise |
| **Layer/component separation** | **(B)** Microservice topology-aware RCA | +10-15% Top@1 | Enables per-layer auditing; reduces candidate set per layer |

**Key observation:** **(B)** When multiple structural properties are combined (as in well-instrumented microservice environments with OpenTelemetry, service dependency graphs, and structured logging), Top@1 accuracy can reach 50-70%+ on the same types of failures that unstructured approaches handle at 20-30%. The improvements are not simply additive -- they interact positively because each structural property reduces the ambiguity space for the others.

#### DSL-Specific Improvement Factors

The following DSL-specific properties go beyond general structured logging and provide additional RCA improvement. For each, I distinguish evidence-backed claims from speculation.

1. **Known schema (all inputs/outputs have declared types and ranges).** **(B)** DSL frameworks like OpenMM define force field parameters, integrator settings, and system configurations with explicit types. This means every trace entry has a known schema, eliminating the free-text parsing problem entirely. Estimated contribution: eliminates ~30% of the difficulty factors listed above (free-text mixing, ambiguous errors, missing taxonomies). **(C)** Estimated accuracy improvement: +15-25% over unstructured traces from this factor alone.

2. **API-enforced theory/implementation separation.** **(B)** In OpenMM, the user specifies a System (theory: forces, particles, constraints) and the framework executes it through a Platform (implementation: CUDA kernels, numerical integration). The boundary is an API call. This is the structural analog of the Lakatosian "hard core" vs. "protective belt" distinction. **(C)** Estimated contribution: enables deterministic Stage 1 (implementation audit) of the LFI, which in ATHENA's architecture must succeed before any theory-level reasoning occurs. If ~41% of errors in Sakana V2 are implementation errors (VISION.md Section 1), resolving these deterministically could improve effective RCA accuracy by filtering out implementation failures before they reach the theory-level classifier.

3. **Deterministic execution within valid parameter space.** **(B)** DSL simulations, given identical inputs, produce identical outputs (within numerical precision bounds). This eliminates the "temporal coupling without causal coupling" problem and the stochastic noise confound. **(C)** Estimated contribution: eliminates ~10-15% of the difficulty from the unstructured case.

4. **Typed parameters with physical constraints.** **(B)** DSL parameters have physical units, valid ranges, and dimensional constraints. An OpenMM simulation with a negative timestep or a VASP calculation with an impossible cutoff energy will fail with a specific, interpretable error rather than a generic exception. **(C)** Estimated contribution: transforms ambiguous errors into typed, classifiable failures. +5-10% improvement.

5. **Pre-execution validation.** **(B)** Many DSL frameworks validate configurations before execution (e.g., GROMACS checks topology consistency, VASP validates INCAR parameters against POTCAR). Failures caught at validation are trivially classifiable as implementation/configuration errors. **(C)** Estimated contribution: may eliminate 20-40% of all failure cases before they even produce runtime traces, dramatically simplifying the remaining RCA task.

6. **Finite, enumerable operation vocabulary.** **(B)** DSL frameworks have a fixed set of operations (force evaluations, integrator steps, energy minimizations, etc.) compared to the unbounded operation space of arbitrary code. This means the IR can represent all possible operations with a finite schema. **(C)** Estimated contribution: makes the IR design problem tractable. The IR does not need to handle arbitrary operations, just the DSL's vocabulary.

**Overall DSL improvement estimate:** **(C)** Combining factors 1-6, a reasonable expectation is that DSL-constrained traces should enable 55-75% Top@1 accuracy on the same failure types that achieve 21% on unstructured traces. This estimate is speculative but grounded in the structural analysis above. The improvement comes from two mechanisms: (a) reducing the input ambiguity that the model must resolve, and (b) enabling deterministic pre-filtering of implementation-layer failures.

#### Residual Hard Cases

Structure alone does not solve the following failure classes. These map to ATHENA's three audit stages:

1. **Theory-theory interactions (Stage 3 -- Theoretical Evaluation).** **(C)** When a simulation fails because the theoretical model is wrong (e.g., a force field parameterization misrepresents a protein-ligand interaction), the trace will show a physically valid execution that produces unexpected results. The IR can represent that the results diverge from predictions, but determining *why* the theory is wrong requires domain knowledge that goes beyond trace parsing. This requires the causal DAG and Bayesian Surprise Evaluator.

2. **Subtle methodology errors (Stage 2 -- Methodological Audit).** **(C)** An experiment might be methodologically incapable of testing the hypothesis (e.g., too-short simulation time to observe rare events, insufficient sampling for a free energy calculation, inappropriate ensemble choice). These failures produce valid, complete traces that simply do not contain the signal needed. The IR can represent what was measured, but determining whether the measurement was *sufficient* requires understanding the relationship between the experiment design and the hypothesis. This requires the causal DAG to know what confounders exist.

3. **Emergent numerical failures (Stage 1 -- Implementation Audit, edge cases).** **(B)** Some implementation failures are not detectable from the DSL's API-level trace alone: floating-point accumulation errors, subtle race conditions in GPU execution, or framework bugs that produce silently wrong results rather than exceptions. These evade pre-execution validation and schema-level checking. They require deeper instrumentation (e.g., energy conservation monitoring, detailed numerical precision tracking) that not all DSL frameworks provide by default.

4. **Multi-component interaction failures (Stages 1-3).** **(C)** When a failure arises from the interaction of multiple correctly-specified components (e.g., a force field that is individually valid but produces artifacts when combined with a specific integrator and barostat), the IR must represent not just individual operations but their interactions. This is a combinatorial problem that scales with the number of interacting components.

5. **Novel failure modes outside the training distribution.** **(C)** Both LLM-based and rule-based RCA systems struggle with failure modes they have not encountered before. Structure helps by constraining the space of possible failures, but genuinely novel failures (new framework bugs, unprecedented parameter combinations) will still challenge any RCA system.

#### Transferability Assessment (DECISION GATE 3)

**Is 21% from a transferable domain?**

**(B)** The 21% figure almost certainly originates from cloud/microservice AIOps benchmarks (RCACopilot, Azure incident datasets, or similar). This domain differs from ATHENA's target domain (scientific DSL simulations) in several critical ways:

| Property | Cloud/AIOps Domain | Scientific DSL Domain | Impact on Transferability |
| :--- | :--- | :--- | :--- |
| Trace structure | Heterogeneous, multi-format | Single framework, known schema | Low transferability -- DSL is much easier |
| Candidate set | 50-500+ services/components | Bounded by DSL operation vocabulary | Low transferability -- DSL has smaller search space |
| Failure types | Infrastructure, network, config, code, human error | Parameter, force field, methodology, numerical | Moderate transferability -- different failure taxonomies |
| Causal complexity | Distributed, asynchronous, cascading | Sequential within simulation, parallel across replicas | Low transferability -- DSL has simpler causal structure |
| Observability | Partial, instrument-dependent | Complete within DSL's API surface | Low transferability -- DSL is more observable |

**Conclusion on transferability:** **(C)** The 21% figure is from a domain that is *harder* than ATHENA's target domain. This means the 21% number is conservative as a baseline for ATHENA -- DSL-constrained RCA should substantially exceed it. However, the domains are sufficiently different that the 21% figure should not be treated as a direct baseline. Instead, it serves as a **motivating contrast**: "even state-of-the-art models achieve only 21% on the hardest version of this problem; we operate in a much easier version."

**What does "significantly exceeding 21%" mean quantitatively?**

**(C)** Given the structural advantages enumerated above, a reasonable target for DSL-constrained RCA accuracy is:
- **Minimum viable:** 60% Top@1 accuracy on planted faults across implementation, methodology, and theory categories. This is approximately 3x the unstructured baseline and demonstrates that DSL structure provides a qualitative improvement.
- **Strong result:** 75-85% Top@1 accuracy. This demonstrates that the IR preserves enough structure for reliable LFI classification on the majority of failure cases.
- **Practical ceiling:** ~90% Top@1 accuracy. The residual 10% represents genuinely hard cases (novel failures, subtle multi-component interactions, emergent numerical issues) that require additional inference beyond what the IR can provide.

These targets are speculative but informed by the structural analysis. They should be validated empirically once the IR is designed and a test suite of planted faults is available.

**Implications:** The IR design must preserve the structural properties that drive the accuracy improvement over unstructured traces. Specifically, the IR must:
1. Preserve the theory/implementation layer separation (enables deterministic Stage 1 audit)
2. Encode typed parameters with physical constraints (enables pre-filtering and typed error classification)
3. Maintain causal/temporal ordering of operations (enables chain-of-causation reconstruction)
4. Represent operation semantics at the DSL's abstraction level, not at the framework's internal level (enables finite operation vocabulary)
5. Include pre-execution validation results (enables trivial classification of caught-at-validation failures)

Any IR design that does not preserve these five properties forfeits the structural advantages that justify the claim of exceeding the 21% baseline.

**Open Threads:**
1. **Verify the 21% source.** A follow-up session with web access must confirm the exact source paper, dataset, candidate set size, and models evaluated. If the number cannot be traced, the claim in VISION.md needs reframing with a verified number. Priority: high.
2. **Survey DSL-specific RCA work.** The literature review above focused on cloud/AIOps RCA. Scientific computing-specific failure analysis literature (e.g., simulation debugging tools, computational chemistry error analysis) may provide more directly transferable baselines. Priority: medium.
3. **Quantify DSL improvement empirically.** The estimated 55-75% range is speculative. Building even a simple prototype that classifies planted faults in OpenMM traces would provide a grounded data point. This connects to Next Step 1 (survey DSL trace formats) and Next Step 5 (draft candidate IR schemas). Priority: medium, but depends on completing Next Steps 1-3 first.
4. **Assess candidate set size sensitivity.** The meaning of Top@1 depends critically on candidate set size. For ATHENA's three-way classification (implementation/methodology/theory), the "candidate set" is just 3 categories, not 50-500 services. Top@1 on a 3-class problem with random baseline 33% is a fundamentally different metric than Top@1 on a 200-class problem with random baseline 0.5%. The success criterion should be reframed in terms of three-way classification accuracy rather than direct comparison to cloud RCA Top@1. Priority: high.
5. **Check "Stalled, Biased, and Confused" (arxiv:2601.22208).** This 2025/2026 paper likely contains the most up-to-date comprehensive evaluation and may either confirm or supersede the 21% figure. Priority: high.

### 2026-02-20: LFI Audit → IR Requirements Mapping

**Scope:** Backward derivation of minimum IR semantic distinctions from ARCHITECTURE.md three-stage audit (§5.3). For each audit stage, enumerate every deterministic question the LFI must answer, then derive what IR content enables that answer. Also derive cross-cutting requirements, ambiguity handling requirements, and hidden confounder litmus test requirements.

**Method:** Requirements analysis. Source material: ARCHITECTURE.md §4.5 (Trace Semantics Engine), §5.1-5.4 (Information Flow, including Fault Isolation Decision Tree), §8.1 (Per-Component Risks), §8.4 (Incomplete Observability), §8.5 (Classification Staleness); VISION.md §4.1 (LFI), §6 (Honest Limitations), Open Question #1; evaluation/hidden-confounder/README.md (litmus test specification). For each of the three audit stages, I extracted every question the LFI must deterministically answer from the ARCHITECTURE.md text, then worked backwards to the minimum IR element that enables answering that question. Requirements are numbered R1-R25 for cross-referencing in Step 3b (coverage matrix) and Step 5 (IR schema evaluation).

**Findings:**

#### Stage 1: Implementation Audit — IR Must Support

The LFI's Stage 1 asks four explicit questions (ARCHITECTURE.md §5.3, Stage 1 paragraph). Each maps to one or more IR requirements.

**Q1.1: "Did execution complete without framework-level exceptions?"**
The IR must represent whether the DSL framework's execution reached normal termination or terminated abnormally, and if abnormally, what exception or error the framework raised.

- **R1. Execution completion status.** A per-execution record indicating: (a) whether the simulation run completed normally, (b) if not, the framework-reported termination reason. Data: enum {completed, exception, timeout, killed} plus framework error identifier. Source: DSL framework exit status and error logs. Example (OpenMM): a `NaNException` raised by `VerletIntegrator.step()` indicating numerical divergence; example (GROMACS): `Fatal error: step N` indicating constraint failure.

- **R2. Exception/error event.** When execution terminates abnormally, a structured record of the exception: type/code, the framework component that raised it, and the call location within the DSL API (not arbitrary Python stack, but the DSL-layer call path). Data: exception type identifier, DSL component identifier, DSL-layer call location. Example (VASP): `ZBRENT: fatal error in bracketing` from the electronic minimizer; example (OpenMM): `OpenMMException` from `Context.setPositions()` indicating invalid atom coordinates.

**Q1.2: "Do input data pipelines match the specification?"**
The IR must represent the experiment specification's expected inputs and the actual inputs observed during execution, in enough detail for the LFI to compare them.

- **R3. Input specification record.** The experiment specification's declared inputs: parameter names, expected values or ranges, data sources, and formats. Data: list of (parameter_name, expected_value_or_range, source_identifier). This is derived from the experiment specification, not the trace log, but must be represented in the IR for comparison. Example: an OpenMM experiment specifying `temperature=300*kelvin`, `topology=1ubq.pdb`, `forcefield=amber14-all.xml`.

- **R4. Actual input observation.** For each declared input, the value actually used during execution, as recorded in the trace log. Data: list of (parameter_name, actual_value, source_event_reference). Example: GROMACS `.mdp` file values as logged at simulation startup; VASP `INCAR` parameter echo at job start.

- **R5. Input validation result.** A derived comparison: for each input, whether the actual value matches the specification, and if not, the nature of the mismatch. Data: list of (parameter_name, match_status: {exact, within_range, mismatch, missing}, deviation_detail). This is a computed IR element, not directly extracted from the trace.

**Q1.3: "Are numerical operations within precision bounds?"**
The IR must represent the numerical health of the simulation during execution.

- **R6. Numerical status record.** Records of numerical conditions encountered during execution: NaN values, infinities, overflow/underflow events, precision mode (single/double), convergence failures in iterative solvers, and energy conservation violations. Data: list of (event_type: {nan, infinity, overflow, underflow, convergence_failure, conservation_violation}, location_in_DSL_pipeline, timestamp_or_step, severity, affected_quantity). Example (OpenMM): energy values becoming NaN at step 5000; example (VASP): electronic self-consistency loop failing to converge after maximum iterations; example (GROMACS): LINCS warning about constraint deviations.

**Q1.4: "Does the hardware/resource state match expectations?"**
The IR must represent the execution environment's state.

- **R7. Resource/environment status.** Records of the computational platform and resource state: device type (CPU/GPU), memory allocation and usage, parallelization configuration, and any resource-related warnings or failures. Data: (platform_type, device_identifiers, memory_allocated, memory_peak, parallelization_config, resource_warnings[]). Example (OpenMM): CUDA device selection, GPU memory exhaustion; example (GROMACS): MPI rank failure, thread count mismatch.

**Stage 1 summary.** Requirements R1-R7 are necessary and sufficient for the LFI to answer all four Stage 1 questions. All seven are implementation-layer concerns and must be tagged as such (see R19). All are directly extractable from DSL trace logs because DSL frameworks structurally separate these operations from theory-layer specifications (ARCHITECTURE.md §3.1).

#### Stage 2: Methodological Audit — IR Must Support

The LFI's Stage 2 asks four explicit questions (ARCHITECTURE.md §5.3, Stage 2 paragraph). Stage 2 is reached only if Stage 1 finds no faults. Stage 2 requires comparing the experiment specification against the hypothesis's causal claims, using the current DAG as context.

**Q2.1: "Does the experiment measure the variables the hypothesis links causally?"**
The IR must represent what was actually measured/observed during the experiment, with enough specificity to compare against the hypothesis's causal claims.

- **R8. Observable measurement record.** For each quantity measured during the experiment: the variable name (as defined in the DAG), the measurement method or observable type, the raw values or summary statistics, and the measurement conditions (e.g., at what simulation time, under what state). Data: list of (variable_name, measurement_method, values_or_summary, measurement_conditions, units). Example (OpenMM): radial distribution function g(r) computed from trajectory frames 500-1000; example (VASP): total energy per atom after ionic relaxation.

- **R9. Observable-to-DAG linkage.** For each measured observable, a mapping to the DAG node(s) it corresponds to, enabling the LFI to verify that the experiment measured the variables the hypothesis claims are causally linked. Data: list of (observable_id, DAG_node_id, relationship_type: {direct_measurement, proxy, derived}). This is a cross-referencing requirement: the IR must support joining observables to the causal graph. Source: ARCHITECTURE.md §5.3 ("comparing the experiment specification against the hypothesis's causal claims").

**Q2.2: "Is the intervention on the hypothesized cause or a correlated proxy?"**
The IR must represent what was intervened upon (the independent variable manipulation) and how.

- **R10. Intervention specification.** A record of the experimental intervention: which parameter(s) were varied, over what range, what control conditions were maintained, and whether the intervention targets the hypothesized causal variable directly or through an intermediary. Data: (intervened_parameter_name, intervention_range, control_conditions[], DAG_node_id_of_target, directness: {direct, proxy}). Example: varying `temperature` from 280K to 350K in OpenMM while holding `pressure` constant, targeting the DAG node for thermal kinetic energy.

- **R11. Intervention-to-DAG linkage.** A mapping from the intervention to the DAG edge(s) the hypothesis claims are causal. The LFI must verify the intervention targets the upstream node of the hypothesized causal edge, not a correlated but causally distinct variable. Data: (intervention_id, hypothesized_edge: {cause_node, effect_node}, intervention_targets: {cause_directly, proxy_via_node_X}). Source: ARCHITECTURE.md §5.3 ("Is the intervention on the hypothesized cause or a correlated proxy?").

**Q2.3: "Is the sampling sufficient to distinguish the effect from noise?"**
The IR must represent sampling adequacy.

- **R12. Sampling metadata.** Records of the experiment's sampling characteristics: sample count (e.g., number of trajectory frames, number of independent runs), sampling distribution, equilibration period, autocorrelation time, and any power analysis or uncertainty quantification performed. Data: (sample_count, sampling_method, equilibration_steps, autocorrelation_time_if_computed, statistical_power_if_computed, confidence_level). Example (GROMACS): 10 ns production run with 2 ns equilibration, 1000 frames sampled every 10 ps; example (VASP): 5 independent relaxations from perturbed starting geometries.

**Q2.4: "Are there known confounders (from the current DAG) that the experiment did not control for?"**
The IR must represent which variables were held constant (controlled) during the experiment and enable comparison against the DAG's confounder structure.

- **R13. Controlled variable set.** An explicit list of variables that the experiment held constant or controlled for, and the mechanism of control. Data: list of (variable_name, control_value_or_method, DAG_node_id). Example: pressure held at 1 atm via barostat in OpenMM; exchange-correlation functional held as PBE in VASP.

- **R14. DAG confounder query support.** The IR must be structured so the LFI can query: "Given the intervention in R10 and the observable in R8, which DAG nodes are potential confounders (common causes of both), and are they in the controlled set R13?" This is not a stored IR element but a queryability requirement: the IR must support efficient joins between intervention nodes, observable nodes, controlled variable sets, and DAG structure. Source: ARCHITECTURE.md §5.3 ("known confounders from the current DAG that the experiment did not control for") and §8.5 ("the confounder check depends on the DAG's accuracy").

**Stage 2 caveat.** ARCHITECTURE.md §5.3 explicitly warns: "the confounder check depends on the DAG's accuracy. If the DAG is missing a real confounder or contains a spurious one, this audit will either miss real confounders or flag phantom ones." The IR cannot fix this. But the IR must make the DAG dependency transparent -- every confounder judgment must be traceable to the specific DAG edges consulted (see R14). This traceability enables reclassification when the DAG changes (ARCHITECTURE.md §8.5).

#### Stage 3: Theoretical Evaluation — IR Must Support

Stage 3 is reached only if Stages 1 and 2 pass. The LFI compares results against the hypothesis's predictions (ARCHITECTURE.md §5.3, Stage 3 paragraph).

**Q3.1: "Does the evidence contradict the hypothesis's predictions?"**
This requires three sub-elements: what the hypothesis predicted, what was observed, and a formal comparison.

- **R15. Prediction record.** The hypothesis's quantitative predictions, stated before the experiment was run: which observable, what predicted value or distribution, what predicted direction of effect, and what tolerance or confidence interval constitutes "agreement." Data: (hypothesis_id, predicted_observable: variable_name, predicted_value_or_distribution, predicted_direction: {increase, decrease, no_change, specific_relationship}, tolerance_or_CI, DAG_edges_supporting_prediction[]). Source: ARCHITECTURE.md §5.1 ("candidate hypotheses with explicit causal claims and testable predictions") and §5.3 ("compares results against the hypothesis's predictions").

- **R16. Observation record.** The actual experimental result for the predicted observable, as extracted from the trace and processed by the IR. Data: (observable_id matching R8, actual_value_or_distribution, uncertainty_estimate, measurement_conditions). This overlaps with R8 but is specifically the subset of observables relevant to the hypothesis's predictions.

- **R17. Comparison result.** A formal quantitative comparison between prediction (R15) and observation (R16): effect size, statistical divergence measure (e.g., KL divergence, z-score, Bayes factor), confidence interval overlap, and a determination of whether the observation falls within or outside the prediction's tolerance. Data: (prediction_id, observation_id, effect_size, divergence_measure, divergence_value, within_tolerance: bool, comparison_method). Source: ARCHITECTURE.md §5.3 ("If the evidence contradicts the hypothesis") -- "contradicts" must be formalized as a quantitative comparison.

**Q3.2: "Which causal edges does the contradiction implicate?"**
When Stage 3 determines theoretical falsification, the LFI must produce "a graph update directive specifying which edges to prune or reweight" (ARCHITECTURE.md §5.3).

- **R18. Causal implication mapping.** For a theoretical falsification, a mapping from the contradicted prediction to the specific DAG edges that supported that prediction, enabling the LFI to produce a targeted graph update directive rather than a blanket penalty. Data: (falsified_prediction_id, implicated_DAG_edges[], proposed_update_per_edge: {prune, reweight, annotate}). Source: ARCHITECTURE.md §5.3 ("the graph update directive specifies which edges to prune or reweight") and §5.1 ("a directed update specifying which edges to prune, reweight, or annotate as falsified").

#### Cross-Cutting Requirements

These requirements apply across all three stages and are necessary for the LFI to function as specified.

- **R19. Layer tag.** Every IR element must be tagged as either `implementation-layer` or `theory-layer`. This is the fundamental structural distinction that makes the three-stage audit possible. The DSL's API separation provides this distinction (ARCHITECTURE.md §3.1: "the theoretical specification and the computational implementation are separated by the framework's API"), but the IR must preserve it. Without layer tags, the LFI cannot determine which stage an element belongs to. Source: ARCHITECTURE.md §3.1 and §4.5 ("maps theory-layer operations to implementation-layer events").

- **R20. Provenance chain.** Every IR element must be traceable to its source in the raw trace log: which log line(s), which file, which timestamp in the raw output produced this IR element. This is required for (a) the LFI to verify its reasoning against raw evidence, (b) enhanced logging re-runs when classification is ambiguous (ARCHITECTURE.md §5.3, Ambiguity handling), and (c) human escalation, where the raw evidence must be presentable (ARCHITECTURE.md §6.3). Data: each IR element carries (source_file, source_line_range, raw_text_hash). Source: ARCHITECTURE.md §4.5 ("receives raw trace logs... produces structured semantic failure/success representation"), §5.3 (ambiguity handling: "re-run with enhanced logging"), §6.3 (escalation: "provides the raw evidence").

- **R21. Temporal ordering.** IR elements must preserve causal sequence: the order in which events occurred during execution. The outside-in audit structure (Stage 1 before Stage 2 before Stage 3) requires knowing what happened in what order -- an exception at step 5000 preceded by a NaN at step 4999 tells a different causal story than the reverse. Data: every IR event carries a temporal coordinate (simulation step, wall-clock timestamp, or logical sequence number) enabling total ordering. Source: ARCHITECTURE.md §5.3 (sequential audit requires temporal reasoning about execution events).

- **R22. Experiment specification linkage.** The IR must include or reference the full experiment specification that produced the trace, so the LFI can compare intended vs. actual execution. The LFI receives "the experiment specification" as a separate input (ARCHITECTURE.md §4.5), but the IR must be joinable to it -- every IR element about inputs (R3, R4, R5), interventions (R10), and controls (R13) must reference the corresponding specification element. Source: ARCHITECTURE.md §4.5 (LFI "receives the structured IR from the Trace Semantics Engine... the experiment specification, and the hypothesis under test").

- **R23. Hypothesis linkage.** The IR must be joinable to the hypothesis under test, so Stage 2 can compare methodological adequacy against causal claims and Stage 3 can compare observations against predictions. The hypothesis itself is a separate LFI input, but the IR's observable records (R8), intervention records (R10), and prediction records (R15) must reference hypothesis elements. Source: ARCHITECTURE.md §4.5 (LFI receives "the hypothesis under test") and §5.3 (Stage 2: "comparing the experiment specification against the hypothesis's causal claims").

- **R24. Queryability.** The IR must support efficient lookup by: (a) layer tag (implementation vs. theory), (b) event type (execution, exception, numerical, resource, observable, intervention, etc.), (c) temporal range (events within step N to M), (d) variable name (all records pertaining to a specific variable), (e) DAG node (all records linked to a specific DAG node), (f) stage relevance (which IR elements are relevant to Stage 1 vs. 2 vs. 3). This is a structural requirement on the IR's organization, not on its content. The three-stage audit is sequential; each stage must be able to efficiently extract the subset of IR elements it needs without scanning the entire representation. Source: ARCHITECTURE.md §5.3 (sequential audit structure implies stage-specific queries) and §4.5 ("structured semantic intermediate representation suitable for causal fault analysis").

#### Ambiguity Handling Requirements

ARCHITECTURE.md §5.3 (Ambiguity handling) specifies: "When the LFI cannot confidently assign a failure to a single category, this is an escalation condition." The IR must support this.

- **R25. Classification confidence metadata.** For each IR element that contributes to a stage's determination, the IR must carry information about the element's completeness and reliability. Specifically: (a) whether the element was fully observed or partially inferred, (b) whether the raw trace contained sufficient information to populate all fields, (c) any gaps or uncertainties in the extraction. This enables the LFI to compute a classification confidence and trigger escalation when confidence is low. Source: ARCHITECTURE.md §5.3 (ambiguity handling), §8.4 ("unrecorded state changes introduce invisible failures"), §6.3 (A1: "irresolvable fault classification" -- the LFI needs to know when its evidence is insufficient).

- **R26. Observability gap record.** When the trace log lacks data that the IR schema expects (e.g., a numerical health metric that the framework did not log, a controlled variable whose value was not recorded), the IR must explicitly represent the gap rather than silently omitting the element. This is critical for incomplete observability (ARCHITECTURE.md §8.4): the LFI must distinguish "this was checked and is fine" from "this was not checkable." Data: list of (expected_element, gap_reason: {not_logged, framework_limitation, configuration_omission}, severity). Source: ARCHITECTURE.md §8.4 ("when the trace log does not contain the data of the actual failing component, the LFI will misattribute the failure").

#### Hidden Confounder Litmus Test Requirements

The litmus test (evaluation/hidden-confounder/README.md) demands specific IR capabilities beyond the general three-stage audit.

- **R27. Confounder-as-methodological classification support.** The litmus test expects ATHENA to "perform Lakatosian Fault Isolation to explicitly tag the dataset as confounded (a failure of the protective belt, not the core theory)" (VISION.md §7). This means the IR must represent the confounder as a Stage 2 (methodological) issue, not Stage 3 (theoretical). Specifically, the IR must be able to represent: a variable that correlates with the observable (R8) and the intervention (R10) but is not in the controlled set (R13), and that the DAG identifies as a potential confounder (R14). The confounder detection in the litmus test is the canonical test of R14's sufficiency.

- **R28. Interventional vs. observational distinction.** The litmus test's confounder is "discoverable only through interventional experiments that probe confounding structure" (hidden-confounder/README.md §2). The IR must distinguish between results obtained under intervention (the adversarial experiment designer actively varied a parameter) and results obtained under passive observation (the parameter varied naturally). This distinction is critical because confounders that are invisible in observational data become visible under intervention. Data: each observation record (R8, R16) must carry an (observation_mode: {interventional, observational}) tag. Source: hidden-confounder/README.md ("discoverable only through interventional experiments").

- **R29. Cross-experiment queryability.** The litmus test operates over 50 cycles. The LFI must be able to query IR elements across multiple experiments to detect patterns (e.g., a variable that consistently co-varies with the outcome across experiments but was never intervened upon). This extends R24 to multi-experiment scope. Data: every IR element carries an experiment_cycle_id enabling cross-experiment joins. Source: hidden-confounder/README.md ("maximum of 50 experiment execution cycles") and ARCHITECTURE.md §5.1 ("accumulated failure history").

**Implications:**

1. *Straightforward to extract from DSL traces (R1, R2, R6, R7):* Execution completion status, exceptions, numerical health, and resource state are directly emitted by DSL frameworks as log messages, error codes, and status reports. These are the most tractable requirements. The DSL trace format surveys (Step 1) should confirm this for OpenMM, GROMACS, and VASP specifically.

2. *Require matching trace data against experiment specifications (R3, R4, R5, R10, R13):* Input validation, intervention specification, and controlled variable identification require comparing what the experiment specification declared against what the trace log records as actually executed. The IR must bridge two data sources (specification + trace), not just parse one. This is tractable but requires a well-defined experiment specification format.

3. *Require DAG context to populate (R9, R11, R14, R18, R27):* Several requirements involve linking IR elements to DAG nodes and edges. The IR does not store the DAG, but it must be joinable to it. This means the IR's variable naming and identification scheme must be compatible with the DAG's node identification scheme. This is a coordination requirement between the Trace Semantics Engine and the Causal Graph Manager.

4. *Require hypothesis context to populate (R15, R23):* Prediction records come from the hypothesis, not from the trace. The IR must incorporate hypothesis-derived data or be joinable to the hypothesis structure. This means the IR is not purely a trace-derived artifact -- it is a composite of trace data, experiment specification, and hypothesis predictions.

5. *May be partially unobservable (R25, R26):* The IR must represent its own gaps. This is the honest response to ARCHITECTURE.md §8.4. The IR will inevitably be incomplete for some experiments; the question is whether the incompleteness is visible or silent.

6. *Require inference or derivation, not direct extraction (R5, R17):* Input validation results and prediction-observation comparisons are computed from other IR elements, not read from trace logs. The IR must support derived elements, not just raw extractions.

7. *R19 (layer tagging) is the load-bearing requirement.* Without the implementation/theory layer distinction, the entire three-stage structure collapses. The DSL's API separation is what makes this possible (ARCHITECTURE.md §3.1), but the IR must faithfully preserve it. If the layer tag is wrong for any element, the LFI may skip Stage 1 checks that should have caught an implementation error, or apply Stage 1 checks to theory-layer elements.

8. *R28 (interventional vs. observational) is critical for the litmus test but not explicitly required by the three-stage audit text.* This requirement is derived from the litmus test specification, not from §5.3 directly. It represents a gap: the ARCHITECTURE.md audit description does not explicitly distinguish interventional from observational evidence, but the litmus test cannot be passed without this distinction.

**Open Threads:**

1. **Dependency on Step 1 (DSL trace survey).** Requirements R1, R2, R6, R7 assert that certain data is "directly extractable from DSL traces." The Step 1 survey must confirm this for each target framework. If a framework does not log numerical health metrics (R6) or resource state (R7) by default, the requirement is valid but the extraction is harder -- it may require custom logging configurations.

2. **Variable naming coordination.** Requirements R9, R11, R14 require the IR's variable names to be joinable to DAG node identifiers. This implies a shared ontology or naming convention between the Trace Semantics Engine and the Causal Graph Manager. This coordination is not addressed by any current research investigation and may need its own decision.

3. **Composite IR vs. trace-only IR.** The findings show the IR is not a pure trace-log derivative. It incorporates experiment specification data (R3, R10, R13), hypothesis data (R15), and DAG references (R9, R11, R14, R18). The Step 5 schema evaluation should explicitly address whether the IR is a single composite structure or a set of joinable structures with defined interfaces.

4. **Cross-experiment scope.** R29 extends the IR's scope from single-experiment to multi-experiment. This has implications for IR storage and lifecycle that Step 5 must address.

5. **Derived elements.** R5 and R17 are computed from other IR elements. The IR schema must define whether these are stored or computed on demand. This affects queryability (R24) performance.

6. **Ambiguity threshold.** R25 requires "classification confidence metadata" but does not specify what threshold constitutes "insufficient confidence" for escalation. This is an LFI design decision, not an IR design decision, but the IR must provide the raw material for confidence computation.

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

### Entry 001 — 2026-02-20: VASP Trace Output System Survey

**Scope:** Complete survey of VASP's output file system, theory-implementation boundary analysis, vasprun.xml structure, failure signaling, and closed-source constraints. Part of Next Step 1 (Survey DSL trace formats).

**Method:** Systematic analysis of VASP's documented output system based on VASP Wiki documentation, pymatgen/ASE API documentation, and domain knowledge of DFT workflows. Produced a structured analysis document (`dsl-evaluation/vasp-trace-analysis.md`) covering seven sections: output file inventory, theory-implementation boundary, vasprun.xml structure, output file comparison, failure signaling, DFT-specific theory-implementation mapping, and closed-source constraints. Each claim tagged with evidence basis ([documented], [observed], [inferred]).

**Findings:**

1. **VASP's output system is well-structured for theory-layer reconstruction.** vasprun.xml provides a comprehensive XML record of all input parameters (with resolved defaults), SCF convergence trajectories, ionic step results (energy, forces, stress), eigenvalues, and DOS. Combined with OUTCAR (implementation diagnostics, warnings, timing) and stdout/stderr (crash information), this forms a sufficient trace for most fault isolation tasks.

2. **The theory-implementation boundary exists but is not API-declared.** VASP's INCAR file mixes theory parameters (GGA, ENCUT, ISMEAR) and implementation parameters (NCORE, KPAR, ALGO) in a single flat namespace. Unlike MD codes where force fields are external data files, VASP's "force field" (the exchange-correlation functional) is selected by an INCAR tag. ATHENA must maintain an external classification table for INCAR tags — a finite engineering task (~200-300 tags total, ~50-80 commonly used).

3. **Theory is distributed across four input files.** INCAR specifies the functional and convergence criteria. POSCAR defines the crystal structure. POTCAR provides pseudopotentials (electron-ion interaction approximation). KPOINTS specifies Brillouin zone sampling. All four carry theory content. The IR must capture and fuse all four into a unified specification representation.

4. **Ambiguous parameters create genuine cross-layer coupling.** PREC simultaneously configures physical accuracy and resource allocation. LREAL trades accuracy for speed. ALGO can affect which SCF minimum is found in pathological cases. These parameters cannot be cleanly assigned to theory or implementation and require special handling in the IR.

5. **The most dangerous VASP failures are silent.** Insufficient ENCUT, inadequate k-points, inappropriate functional choice, and wrong pseudopotential selection all produce results without any error, warning, or non-zero exit code. VASP does not signal SCF non-convergence via exit code. The Trace Semantics Engine must implement domain-aware validation rules beyond what VASP reports.

6. **Closed-source constraints are manageable but impose a ceiling.** ATHENA cannot instrument VASP internals. Observable output (vasprun.xml + OUTCAR + stdout) is sufficient for standard calculations. The ceiling is hit for subtle numerical issues (FFT aliasing, PAW reconstruction errors, non-deterministic MPI reductions) that are invisible in output.

7. **Community tooling (pymatgen, custodian, ASE) provides mature parsing infrastructure.** pymatgen's custodian package is particularly relevant — it implements automated error detection and correction for VASP, functioning as a community-built implementation audit tool.

8. **VASP's input is purely declarative.** Unlike OpenMM (which requires Python scripting), VASP's four input files contain no imperative code. This makes VASP's input more amenable to static analysis and specification reconstruction than scripting-based DSLs.

**Implications:**

- The IR must support multi-file trace composition (fusing vasprun.xml + OUTCAR + stdout into one semantic representation). This is a structural requirement not present in single-log systems.
- The IR must support convergence trajectory representation (SCF and ionic convergence as sequences, not just final values). Trajectory shape carries diagnostic information.
- Silent failure detection requires a rule-based validation layer within the Trace Semantics Engine, implementing domain-aware checks that VASP itself does not perform. This layer needs access to the Causal Graph Manager for system-type-dependent rules (e.g., metals need denser k-meshes than insulators).
- The IR needs DSL-specific adapters rather than a universal schema — VASP's multi-file declarative input differs structurally from OpenMM's Python scripting interface and GROMACS's MDP/topology format.
- VASP should remain in ATHENA's target set, but ATHENA should also support at least one open-source DFT code for cross-validation and deeper instrumentation.

**Open Threads:**

- How does VASP's output compare quantitatively to OpenMM and GROMACS in terms of trace completeness? Need to complete those surveys for comparison.
- What fraction of real-world VASP failures fall into the "silent" category vs. self-announcing crashes? Materials Project workflow data (custodian error logs) might provide statistics.
- Can custodian's error handler catalog serve as a starting point for the rule-based validation layer?
- VASP 6 introduced the REPORT file with more detailed logging. How much does this close the gap in implementation-layer observability?

**Artifact.** `dsl-evaluation/vasp-trace-analysis.md`

### 2026-02-20: GROMACS Trace Format Characterization

**Scope:** Complete catalog and classification of GROMACS MD simulation output files (.log, .edr, .trr, .xtc, .xvg, .cpt, .tpr, .gro), mapping each to theory-layer or implementation-layer. Assessment of the .mdp specification interface as a theory-implementation API boundary. Error and warning taxonomy. LINCS constraint failure walkthrough. grompp preprocessing validation coverage analysis.

**Method:** Systematic analysis of GROMACS output architecture based on the GROMACS reference manual (manual.gromacs.org), source code documentation (github.com/gromacs/gromacs), panedr library documentation, MDAnalysis documentation, GROMACS user forum failure cases, and published GROMACS architecture descriptions (Abraham et al. 2015). Each output file was cataloged by format, content, programmatic access method, and layer classification. The .mdp parameter space was partitioned into theory-layer, implementation-layer, and boundary parameters. A concrete LINCS failure was traced through all output files to assess reconstructibility. grompp validation checks were enumerated and classified by what they catch vs. what slips through.

**Findings:**

1. **Output file inventory.** GROMACS produces 8 primary output file types. The .edr (energy time series, binary XDR, accessible via panedr Python library) is the richest structured data source for anomaly detection. The .log (semi-structured text) is the primary source of error messages but lacks machine-readable structure (no error codes, no schema). The .tpr (binary compiled run input) is the complete experiment specification but merges theory and implementation layers into a single opaque object. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 1.

2. **Theory-implementation boundary.** The .mdp parameter file provides a relatively clean theory-implementation boundary. Theory-layer parameters (integrator, tcoupl, pcoupl, coulombtype, force field) are declarative and have no implementation side effects. Implementation-layer parameters (nstlog, nstenergy, nstxout) control execution mechanics only. However, 10+ parameters are "boundary" — they serve dual roles affecting both physics and execution. The most consequential boundary parameter is `dt` (timestep), which is a physical/methodological decision that manifests as implementation-like symptoms when wrong. The mdrun command-line parameters (-ntomp, -gpu_id, -dd) are purely implementation-layer, providing the cleanest separation in the system. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 2.

3. **Error taxonomy.** GROMACS errors are free-text messages with no structured classification. The most common simulation failures (LINCS/SETTLE/SHAKE constraint violations, domain decomposition errors) are inherently ambiguous between theory, methodology, and implementation causes. These ambiguous errors are also the most frequent errors the LFI would need to classify. Purely implementation-layer errors (memory, GPU, MPI, file mismatch) are cleanly identifiable by message pattern but represent a small fraction of real-world failures. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 5.

4. **Failure walkthrough (LINCS).** Tracing a LINCS constraint failure through the output system shows that correct fault classification requires cross-file correlation: .log (error messages and warnings), .edr (energy escalation pattern), .tpr (parameter context), and initial structure (clash detection). No single output file contains sufficient information. A critical gap: the exact crash-state coordinates/velocities/forces are not preserved; only the last periodic checkpoint (potentially thousands of steps before the crash) is available. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 6.

5. **grompp validation.** grompp catches syntactic/structural errors comprehensively (atom count mismatches, missing force field parameters, box size violations) but does not validate physical/scientific correctness. Timestep adequacy, force field correctness for the chemistry, equilibration quality, and sampling sufficiency all slip through to runtime. This creates a clean audit hierarchy: grompp validates implementation syntax, runtime monitoring validates numerical stability, and post-run analysis validates physical correctness. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 7.

6. **Six concrete IR requirements derived.** The analysis produced six specific requirements for the IR design: (a) GROMACS parameter classification table, (b) cross-file correlation engine, (c) temporal event linking, (d) error pattern library, (e) data absence tracking, (f) user-specified vs. runtime-adjusted parameter distinction. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 8.3.

**Implications:**

- GROMACS provides sufficient structured data for the Trace Semantics Engine to operate, but the IR must perform substantial work to bridge the gap between raw output and semantic failure representations. The .edr time series (via panedr) is the most IR-friendly data source. The .log error messages are the least IR-friendly.
- The theory-implementation boundary is cleaner than expected for most parameters, but the 10+ boundary parameters require explicit dual-annotation in the IR. The `dt` parameter is the most consequential boundary case: wrong dt produces LINCS failures that look like implementation errors but are actually methodology errors.
- The most common GROMACS failures are inherently ambiguous in the LFI's three-way classification. The IR cannot resolve this ambiguity from the error message alone — it must cross-reference parameters, energy trajectories, topology, and structural context. This means the IR must be a multi-source correlation engine, not just a log parser.
- grompp's validation gap (catches syntax, misses physics) maps cleanly to the LFI's Stage 1 vs. Stage 3 distinction. If grompp accepted the simulation, Stage 1 (implementation audit) can assume the specification is syntactically valid and focus on runtime execution errors. Stage 3 (theory evaluation) must handle everything grompp cannot check.

**Open Threads:**

- How do OpenMM and VASP compare? OpenMM's Python API may provide richer programmatic access but weaker theory-implementation separation. VASP's INCAR/POSCAR/POTCAR system may have different boundary parameter characteristics. These comparisons are needed to identify IR elements that generalize vs. those that are GROMACS-specific.
- Can panedr's DataFrame output serve as a direct input to the IR, or does the IR need a more abstract energy representation that works across frameworks?
- The error pattern library approach (regex matching on known GROMACS error messages) is brittle across GROMACS versions. Is there a more robust approach? GROMACS source code analysis could provide a definitive catalog of error messages.
- The crash-state data gap (no state dump at exact crash point) limits forensic analysis. Is this a fundamental limitation or can GROMACS be configured to dump state on crash?
- How does the auto-tuning behavior (nstlist, rlist, PME parameters) interact with reproducibility? If two runs of the same .tpr produce different auto-tuned parameters, the IR must track this divergence.

**Artifact.** `dsl-evaluation/gromacs-trace-analysis.md`

### 2026-02-20: OpenMM Trace Format Characterization

**Scope:** Complete characterization of OpenMM's trace output system, mapping every output element to theory, implementation, or boundary layer. Covered: reporter system inventory (7 reporter types), theory-implementation API boundary analysis (ForceField/Topology/System vs. Platform/Context), exception and error exposure, execution metadata accessibility, custom reporter extensibility, NaN energy failure walkthrough, and failure mode taxonomy (17 modes across 4 categories).

**Method:** Documentation review (OpenMM Python API docs at docs.openmm.org, OpenMM User Guide chapters 3, 4, and 8), source code analysis (openmm/app/ Python wrappers: simulation.py, statedatareporter.py, dcdreporter.py, pdbreporter.py, pdbxreporter.py, checkpointreporter.py, forcefield.py, topology.py), and failure pattern analysis from OpenMM GitHub issue tracker (NaN energy, precision, constraint failure threads).

**Findings:**

1. **OpenMM enforces a clean theory-implementation API boundary.** The ForceField/Topology/System chain defines the theory specification; Platform/Context define the implementation. The `ForceField.createSystem()` method is the explicit compilation boundary. The System object's contents (forces, parameters, constraints) are fully queryable via the API, making post-compilation auditing tractable. However, the atom type assignment trail is lost at the `createSystem()` boundary — the System does not record which force field atom types were matched to which topology atoms. (Source: `openmm/app/forcefield.py`, `createSystem()` method; detailed in `dsl-evaluation/openmm-trace-analysis.md` Section 2.3.)

2. **Default trace output is insufficient for three-way fault classification.** Of 17 cataloged failure modes, only 4 are definitively detectable and classifiable from default reporters (GPU memory exhaustion, driver incompatibility, and partially force field template matching errors). The remaining 13 either go undetected or are detected without category-distinguishing information. The most critical gap: NaN energy failures are ambiguous between implementation (precision overflow), methodology (time step too large), and theory (bad force field parameters), and default reporters provide no data to distinguish them. (Source: failure mode taxonomy in `dsl-evaluation/openmm-trace-analysis.md` Section 7.5.)

3. **The reporter API is extensible enough for custom ATHENA instrumentation.** Custom reporters receive the full Simulation, Context, and State objects, enabling per-force-group energy decomposition, per-atom force monitoring, and adaptive reporting intervals. The main gap is sub-step instrumentation — reporters fire between steps, not within them, so crash-time state from mid-step failures is unrecoverable via the reporter API. (Source: `openmm/app/simulation.py` reporter invocation logic; detailed in `dsl-evaluation/openmm-trace-analysis.md` Section 5.)

4. **Methodology-layer failures are invisible to OpenMM.** The framework has no concept of scientific adequacy — insufficient equilibration, inadequate sampling, wrong ensemble choice, and correlation between samples are never detected or reported. An ATHENA IR for OpenMM must incorporate methodology assessment as external domain rules, not as parsed trace data. (Source: analysis of OpenMM exception types in `dsl-evaluation/openmm-trace-analysis.md` Section 3.1.)

5. **Temporal resolution of reporters creates diagnostic blind spots.** Reporters fire at configured intervals (typically every 1000-10000 steps). Events between intervals are invisible. In the NaN walkthrough, up to 2347 steps of energy divergence occurred between the last normal report and the crash, with no recorded state for that interval. (Source: NaN walkthrough in `dsl-evaluation/openmm-trace-analysis.md` Section 6.)

**Implications:**

- The IR cannot operate on default OpenMM trace output alone. A custom ATHENA reporter is a prerequisite for effective fault isolation. This reporter should capture per-force-group energy decomposition, monitor force magnitudes, and implement adaptive reporting frequency.
- The theory-implementation boundary in OpenMM is clean enough for Stage 1 (implementation audit) of the LFI decision tree. The Platform/Context separation allows deterministic checking of hardware state, precision mode, and platform compatibility. Stage 2 (methodology audit) and Stage 3 (theory evaluation) require external criteria that OpenMM does not provide.
- The IR must explicitly represent the ForceField -> createSystem() -> System -> Context compilation chain as a layered structure, preserving the theory-implementation separation at each level.
- The atom type assignment gap at `createSystem()` is a specific weak point: if a wrong atom type is assigned (due to ambiguous topology), the error is silent after compilation. The IR should flag this as a potential ambiguity zone requiring supplementary auditing.
- OpenMM's failure mode taxonomy provides a concrete test suite for IR validation: planted faults from each of the 17 failure modes can serve as ground-truth test cases for fault classification accuracy.

**Open Threads:**

1. How do GROMACS and VASP compare on theory-implementation boundary cleanliness? Do they provide richer default trace output?
2. Can the sub-step instrumentation gap be closed by using OpenMM's `CustomIntegrator` to insert monitoring operations within the integration step?
3. What is the computational overhead of per-force-group energy decomposition at every reporting interval? Is it feasible for production simulations?
4. How should the IR represent the "unknown state" in temporal gaps between reporter intervals?
5. Can the atom type assignment trail be reconstructed by re-running `createSystem()` with instrumentation, or must it be captured at compilation time?

## Accumulated Findings

### What We Know

**DSL Trace Architecture**

1. **All three target DSLs provide sufficient structured output for the Trace Semantics Engine to operate, but none provides sufficient *default* output for three-way fault classification.** OpenMM: only 4 of 17 failure modes detectable from default reporters. GROMACS: error messages are free-text with no classification taxonomy; most common failures (constraint violations) are inherently ambiguous. VASP: the most dangerous failures (insufficient ENCUT, inadequate k-points, wrong functional) are completely silent. Each framework requires custom instrumentation or supplementary analysis. [OpenMM log 2026-02-20; GROMACS log 2026-02-20; VASP log 2026-02-20]

2. **The theory-implementation boundary quality varies across frameworks.** OpenMM: clean API boundary at `ForceField.createSystem()`, structurally separating theory (ForceField/Topology/System) from implementation (Platform/Context). GROMACS: relatively clean .mdp parameter separation with 10+ "boundary" parameters requiring dual-annotation; mdrun command-line parameters are purely implementation-layer. VASP: boundary exists but is not API-declared; INCAR mixes theory and implementation in a flat namespace; requires external classification table (~200-300 tags). [OpenMM log 2026-02-20; GROMACS log 2026-02-20; VASP log 2026-02-20]

3. **Methodology-layer failures are invisible to all three frameworks.** No DSL framework detects or reports insufficient equilibration, inadequate sampling, wrong ensemble choice, inappropriate functional, or confounder non-control. These must be assessed by external domain rules, not parsed from trace data. [OpenMM log 2026-02-20 Finding 4; GROMACS log 2026-02-20 Finding 5; VASP log 2026-02-20 Finding 5]

4. **Correct fault classification requires multi-source correlation in every framework.** OpenMM needs per-force-group energy decomposition + reporter data + exception info. GROMACS needs .log + .edr + .tpr + structural context. VASP needs vasprun.xml + OUTCAR + stdout/stderr. No single output file/stream in any framework contains sufficient information for the LFI's three-way classification. [GROMACS log 2026-02-20 Finding 4; VASP log 2026-02-20 Finding 1; OpenMM log 2026-02-20 Finding 2]

5. **Pre-execution validation coverage varies.** GROMACS grompp catches syntactic/structural errors comprehensively but not physical correctness. OpenMM validates force field template matching at `createSystem()` but not parameter physical adequacy. VASP validates INCAR parameters against POTCAR but not convergence adequacy. This creates a consistent pattern: pre-execution catches Stage 1 syntax issues; runtime and post-run analysis handle Stages 2-3. [GROMACS log 2026-02-20 Finding 5; OpenMM log 2026-02-20 Finding 1; VASP log 2026-02-20 Finding 2]

6. **VASP's input is purely declarative; OpenMM requires Python scripting; GROMACS uses a hybrid (declarative .mdp + topology files).** This structural difference means the IR needs DSL-specific adapters rather than a universal input parser. [VASP log 2026-02-20 Finding 8; OpenMM log 2026-02-20; GROMACS log 2026-02-20]

**IR Design Patterns**

7. **LLM-based RCA without formal IR achieves ~21% Top@1 accuracy on unstructured traces.** The root cause is architectural (no structured representation of causal chains), not a capability limitation of the LLMs. [RCA/FV survey 2026-02-20; arxiv:2403.04123; ARCHITECTURE.md 4.5]

8. **Three LLM RCA failure modes (Stalled, Biased, Confused) map directly to IR requirements.** "Stalled" (missing context) requires explicit observability boundaries. "Biased" (prior-dominated reasoning) requires evidence-backed causal chains. "Confused" (conflated causal paths) requires structural isolation of parallel chains. [RCA/FV survey 2026-02-20; arxiv:2601.22208]

9. **MLIR's dialect system provides the structural pattern for theory/methodology/implementation separation in the IR.** Three dialects, multi-level coexistence, explicit lowering relationships between layers. This maps directly to the DSL API separation constraint and the LFI's three-stage audit routing. [RCA/FV survey 2026-02-20]

10. **Boogie/Why3 specification-implementation contracts provide the pattern for the LFI's sequential audit.** Experiment specifications as contracts (requires/ensures/modifies), checked at three levels. This resolves the implicit requirement that the IR must carry the experiment specification as a first-class entity. [RCA/FV survey 2026-02-20]

11. **Specification-implementation conflation is the most critical anti-pattern to avoid.** Representing "what was specified" and "what executed" in the same namespace directly disables the three-stage audit. [RCA/FV survey 2026-02-20]

12. **All identified transferable patterns are compatible with Rust zero-copy/streaming parsing.** Dialect tags as enum variants, contracts as structured records, event chains constructed incrementally. [RCA/FV survey 2026-02-20; ADR 001]

**Provenance Models**

13. **PROV-DM covers approximately 60-70% of ATHENA's IR requirements.** Entity-Activity-Agent model maps well to simulation states and steps. The Agent model is the weakest mapping — it captures "who" but not "what kind" of responsibility. [Provenance survey 2026-02-20; W3C PROV-DM §2-5]

14. **PROV-DM does not natively represent the theory-implementation-methodology trichotomy.** The distinction must be added as a structural extension, not as metadata attributes. [Provenance survey 2026-02-20]

15. **ProvONE's typed Ports provide a natural mechanism for parameter classification at DSL API boundaries** (theory-Ports vs. implementation-Ports vs. methodology-Ports). [Provenance survey 2026-02-20]

16. **PROV-DM's RDF/SPARQL technology stack is incompatible with Rust throughput requirements, but the data model can be adopted without the technology stack.** Custom Rust graph implementations handle megabyte-scale traces (10^4-10^6 triples) in milliseconds for path queries. [Provenance survey 2026-02-20; ADR 001]

17. **Process mining conformance checking is directly applicable to LFI Stages 1 and 2** for detecting structural deviations between expected and actual execution. [Provenance survey 2026-02-20]

18. **No existing provenance system natively supports the theory-implementation distinction or fault classification** at the DSL-internal semantic level ATHENA requires. [Provenance survey 2026-02-20]

**IR Requirements**

19. **The IR must represent a minimum of 29 distinct semantic elements (R1-R29) to support the LFI's three-stage audit.** Derived by backward analysis from each deterministic question the LFI must answer per ARCHITECTURE.md §5.3. [LFI requirements log 2026-02-20]

20. **The IR is not a pure trace-log derivative.** It is a composite of trace-extracted data (R1, R2, R6-R8, R12, R16), experiment specification data (R3, R4, R10, R13), hypothesis-derived data (R15), computed/derived elements (R5, R17), and DAG cross-references (R9, R11, R14, R18). [LFI requirements log 2026-02-20]

21. **R19 (layer tag) is the load-bearing structural distinction.** Without it, the three-stage sequential audit cannot function. The DSL's API separation is what makes this tagging possible. [LFI requirements log 2026-02-20; ARCHITECTURE.md §3.1]

22. **The IR must explicitly represent its own observability gaps (R26).** Silent omission of unobservable elements causes the LFI to misattribute failures. The LFI must distinguish "checked and fine" from "not checkable." [LFI requirements log 2026-02-20; ARCHITECTURE.md §8.4]

23. **Stage 2 requirements (R8-R14) are bounded by DAG accuracy.** Every confounder judgment must be traceable to specific DAG edges consulted, enabling reclassification when the DAG changes. [LFI requirements log 2026-02-20; ARCHITECTURE.md §5.3, §8.5]

**Cross-Framework Synthesis**

28. **Trace completeness varies substantially: OpenMM 30-40% default / 70-80% max, GROMACS 60-70% / 75-85%, VASP 50-60% / 50-60% ceiling.** VASP hits a hard closed-source ceiling that cannot be overcome with custom instrumentation. OpenMM has the widest gap between default and instrumented coverage, meaning custom reporters provide the most marginal value. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §4]

29. **49 failure modes taxonomized: 8 harmonized cross-framework patterns, 7 DSL-specific modes.** Common patterns include: numerical overflow, constraint/convergence failure, memory exhaustion, parameter misspecification, silent methodology inadequacy. DSL-specific modes include VASP SCF non-convergence, GROMACS domain decomposition failure, OpenMM platform-dependent precision divergence. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §3]

30. **Decision Gate 1 resolved: VASP accepted with external classification table.** 70-80% of standard VASP calculations classifiable with full confidence; 20-30% have degraded confidence from ambiguous parameters (PREC, ALGO, LREAL). Five items flagged for adversarial review. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §6]

31. **Seven common IR core elements identified.** Timestamped events, energy time series, parameter records, error events, state snapshots, convergence trajectories, and data absence records generalize across all three frameworks and form the universal IR schema core. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §5]

32. **Adapter contract defined: 7 mandatory + 7 optional methods.** Mandatory: extract_parameters, extract_energy_series, extract_state_snapshots, extract_errors, extract_convergence_metrics, extract_execution_metadata, declare_data_completeness. Optional: validate_preprocessing, extract_runtime_adjustments, extract_scf_convergence, extract_electronic_structure, validate_silent_failures, extract_force_field_compilation, compare_platforms. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §5.3]

**Comparative IR Synthesis**

33. **MLIR dialects and PROV-DM serve complementary roles in the IR architecture.** Dialects provide classification/routing (which LFI stage handles an element); PROV-DM provides causal structure (how elements relate within each stage). The unified architecture uses dialect structure as primary organization with PROV-DM-like causal graphs within each layer. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §4]

34. **Decision Gate 2 resolved: hybrid adaptation, MEDIUM risk.** ~65-70% transfers from existing systems. ~30-35% requires novel design: three-way layer typing vocabulary, fault classification ontology, quantitative prediction-observation comparison formalization, methodology detection rules. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §5]

35. **Nine anti-patterns cataloged with severity ratings and avoidance guidance.** CRITICAL: specification-implementation conflation (directly disables three-stage audit). HIGH: post-mortem-only design (blocks streaming per ADR 001), full-granularity recording (10^8+ nodes), binary pass/fail (collapses three-way), lossy compression without principled selection. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §3]

36. **Three candidate IR designs have distinct pattern-source profiles.** LEL (Layered Event Log): simplest, Stage 1 strongest, log-based. DGR (Dual-Graph IR): natural synthesis of both surveys, Stages 2-3 strongest, graph-based. TAL (Typed Assertion Log): most ATHENA-specific, highest novelty risk, assertion-based. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §6]

**Requirements Coverage**

24. **Stage 1 requirements (R1-R7) are fully satisfiable for all three frameworks.** OpenMM has 4 DI cells (highest instrumentation burden: no parameter echo, API-only access). GROMACS has 5 DA cells (best default coverage). VASP has 4 DA cells but exit code unreliability for SCF non-convergence. Coverage matrix confirms Stage 1 is the most tractable stage. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §1]

25. **31% of R1-R29 requirements (9 of 29) are NT — data sources external to the Trace Semantics Engine.** R9, R10, R11, R15, R18, R22, R23, R28, R29(cycle_id) come from experiment specification, hypothesis, DAG, or workflow controller. This quantifies the IR's composite nature first identified in item 20. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §5.1]

26. **R19 (layer tag) availability varies: OpenMM=DA (clean API), GROMACS=DI+ER (~10 boundary params), VASP=ER (~200-300 INCAR tags).** This is the only cross-cutting requirement with framework-dependent classification difficulty. OpenMM's clean boundary confirms the DSL constraint's value. VASP's ER burden is bounded (finite, static tag set) and accepted per Decision Gate 1. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §2]

27. **Decision Gate 4: PASS — no LFI stage blocked by FU requirements.** FU cells exist only as partial R6 (sub-component numerical internals) at ~5-10% per framework, well below the 10% threshold. Four conditions attached: OpenMM custom reporter, VASP classification table, VASP degraded confidence for ambiguous params, R17 comparison method formalization. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §7]

**Baseline Characterization**

37. **The 21% Top@1 figure in VISION.md is uncited.** It carries no reference number, unlike most other claims in the document. The anchoring number for ATHENA's value proposition is unsourced. [Baseline log 2026-02-20]

38. **The 21% figure almost certainly originates from cloud/AIOps RCA benchmarks, not scientific computing.** The domain is structurally harder than ATHENA's target on every relevant dimension: trace structure, candidate set size, causal complexity, and observability. The figure is a conservative contrast, not a direct baseline. [Baseline log 2026-02-20; evidence quality B]

39. **ATHENA's three-way classification (implementation/methodology/theory) has a candidate set of 3 with random baseline 33%. Cloud RCA Top@1 operates over 50-500+ candidates with random baseline 0.2-2%.** These are fundamentally different metrics and should not be directly compared. [Baseline log 2026-02-20]

40. **Six specific structural properties of traces improve RCA accuracy** (with estimated improvements): temporal/causal ordering (+15-25%), event type taxonomies (+10-20%), schema conformance (+10-20%), causal annotations/dependency graphs (+20-35%), severity levels (+5-10%), layer/component separation (+10-15%). Improvements interact positively. [Baseline log 2026-02-20; evidence quality B]

**Candidate IR Schemas**

41. **Hybrid LEL+DGR is the recommended IR architecture.** Scores 94/100 vs. 82/100 for either standalone candidate (LEL or DGR). Provides per-stage optimized operation: LEL streaming efficiency for Stage 1 (the common classification path) and DGR-like causal reasoning for Stages 2-3 (the differentiating path). PASS on all 9 anti-patterns. Supersedes the suspicion that DGR alone was strongest. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §9-10]

42. **The LEL→DGR incremental implementation path is viable.** Demonstrated by the Hybrid candidate's construction: LEL events carry dag_node_ref/spec_ref/causal_refs from day one; CausalOverlay built at Stage 1→2 boundary via single O(n) pass. Key constraint: LEL events must include DGR-compatible references from initial construction, pushing some entity resolution into the adapter even during Stage 1. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §4, §8 OQ2]

43. **A common structural foundation (7 shared types) is independent of candidate choice.** Layer enum, BoundaryClassification enum, ObservationMode enum, Value enum with Havoc variant, TemporalCoord struct, ProvenanceAnchor struct, ExperimentRef struct, and ConfidenceMeta struct are shared across all candidates. These can be implemented first and reused regardless of which IR representation is chosen. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §1]

44. **TAL is better as a query interface than a storage format.** The coverage matrix and candidate evaluation confirm that TAL's assertion-checking pattern functions identically as a query layer over LEL or DGR substrates. TAL's core strength (sequential audit assertions with evidence chains) does not require a standalone IR representation. Adopted as the recommended LFI query interface. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §0, §10]

45. **BoundaryClassification enum resolves the boundary parameter representation question.** Three variants (PrimaryLayer, DualAnnotated, ContextDependent) handle the full spectrum from unambiguous parameters to context-dependent ones like VASP ALGO. Primary layer determines LFI routing; secondary annotations provide diagnostic context. Avoids both a fourth "boundary" layer and entity duplication. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §1, §8 OQ4]

46. **R17 comparison is structurally resolved as a pluggable container.** ComparisonResult + DivergenceMeasure enum (6 variants: AbsoluteDifference, ZScore, BayesFactor, KLDivergence, EffectSize, Custom) provides the IR's structural slot. The comparison method is pluggable — the IR stores results, the LFI supplies logic. The R17 formalization research is now scoped to LFI logic, not IR structure. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §3, §8 OQ1]

47. **The causal reasoning substrate question has a per-stage answer.** Stage 1: sequential search sufficient (filter-and-inspect on implementation-tagged events). Stages 2-3: graph traversal required (transitive causal ancestry for R14 confounders, structural path finding for R18 causal implications). This per-stage resolution directly motivates the Hybrid design. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §8 OQ3]

**Open Thread Resolution**

48. **Real CausalOverlay construction cost is empirically bounded at 10^6 scale.** With benchmark wired to `CausalOverlay::from_log`, observed overlay construction is 251.82ms at 10^6 events (22.62ms at 10^5), with 1,000,000 overlay entities and 199,998 derivation edges. Construction remains single-pass O(n) and tractable for prototype-scale traces. [Step 6 log 2026-02-21; `lel-ir-prototype/src/bench.rs`]

49. **`EventIndexes.by_id` is now implemented and removes the Phase 2 lookup blocker.** The prototype now carries `by_id: HashMap<EventId, usize>` with insert-time population and serde coverage, enabling O(1) EventId→event-position lookup during overlay construction and graph queries. [Step 6 log 2026-02-21; `lel-ir-prototype/src/lel.rs`, tests]

50. **R14 confounder detection now executes end-to-end on the overlay prototype.** `detect_confounders` performs ancestor-intersection + controlled/intervention filtering with grouped `ConfounderCandidate` outputs; 7 targeted tests validate controlled-variable exclusion, intervention exclusion, multi-confounder grouping, transitive ancestry, and unknown-variable guards. [Step 6 log 2026-02-21; `lel-ir-prototype/src/overlay.rs`, tests]

### What We Suspect

**DSL Trace Architecture**

1. **A custom ATHENA reporter for OpenMM capturing per-force-group energy decomposition would resolve most NaN ambiguity.** The OpenMM API supports `getState(groups={i})` for energy decomposition; overhead is untested. [OpenMM log 2026-02-20]

2. **The atom type assignment gap at OpenMM's `createSystem()` is a tractable engineering problem,** recoverable via instrumentation or post-hoc comparison of System parameters against ForceField XML. [OpenMM log 2026-02-20]

3. **The `dt` (timestep) parameter may be the single most diagnostic boundary parameter for GROMACS fault classification.** Wrong dt is the most common LINCS failure cause, and produces symptoms (constraint violation, energy explosion) that appear to be implementation failures but are actually methodology errors. [GROMACS log 2026-02-20]

4. **An error pattern library for GROMACS (regex on error messages) may suffice for prototyping but is likely too brittle for production** across GROMACS versions. [GROMACS log 2026-02-20]

5. **Silent theory failures (insufficient ENCUT, inadequate k-points, inappropriate functional) may constitute a significant fraction of real VASP failures,** making domain-aware validation rules essential. [VASP log 2026-02-20]

6. **Custodian's error handler catalog could serve as a foundation for the rule-based validation layer** in the Trace Semantics Engine for VASP. [VASP log 2026-02-20]

7. **The IR will need DSL-specific adapters rather than a universal schema,** because VASP's multi-file declarative input, OpenMM's Python scripting, and GROMACS's MDP/topology format differ structurally. [VASP log 2026-02-20; confirmed across all three surveys]

**IR Design**

8. **The hybrid approach (PROV-DM data model concepts in Rust-native structures with ATHENA-specific extensions) likely offers the best risk/reward tradeoff.** Captures W3C standard maturity without RDF performance costs. [Provenance survey 2026-02-20]

9. **The theory-implementation-methodology distinction should be structural (in the type system) rather than attributional (in metadata).** Attribute-based encoding forces every LFI query to filter by metadata, adding complexity and ambiguity. [Provenance survey 2026-02-20; RCA/FV survey 2026-02-20]

10. **The dialect boundary definition will be the hardest per-DSL adaptation problem.** Determining which trace events belong to theory/methodology/implementation for each target DSL requires deep understanding of each framework's API structure. [RCA/FV survey 2026-02-20]

11. **Stage 3 (theoretical evaluation) may require full-trace buffering, breaking the streaming model.** Theoretical predictions are evaluated against aggregate outcomes. Stages 1 and 2 can likely operate in streaming mode. [RCA/FV survey 2026-02-20]

12. **Ghost state for methodological metadata inherits DAG quality problems.** If the causal DAG is wrong about confounders, methodological ghost state will encode incorrect claims, propagating the bootstrapping error. [RCA/FV survey 2026-02-20; ARCHITECTURE.md 8.3]

13. **Contract extraction from DSL experiment scripts may be partially automatable** — DSL APIs have typed parameter specifications that could serve as preconditions. Postconditions likely require manual or LLM-assisted specification. [RCA/FV survey 2026-02-20]

**Requirements and Baseline**

14. ~~**Stage 1 requirements (R1, R2, R6, R7) are the most tractable.**~~ PROMOTED to What We Know #24. Coverage matrix confirms: Stage 1 is fully satisfiable for all three frameworks, with OpenMM needing the most instrumentation (4 DI cells) and GROMACS having the best default coverage (5 DA cells). [Coverage matrix log 2026-02-20]

15. **R28 (interventional vs. observational distinction) may be a gap in ARCHITECTURE.md §5.3.** The audit description does not explicitly require it, but the hidden confounder litmus test cannot be passed without it. [LFI requirements log 2026-02-20]

16. **A shared variable naming ontology between the Trace Semantics Engine and the Causal Graph Manager is an implicit requirement** (from R9, R11, R14) not addressed by any current research investigation. [LFI requirements log 2026-02-20]

17. **The IR must preserve at least five structural properties to maintain DSL advantage over unstructured traces:** theory/implementation layer separation, typed parameters with physical constraints, causal/temporal ordering, DSL-level operation semantics, and pre-execution validation results. [Baseline log 2026-02-20; evidence quality C]

18. **DSL-constrained RCA should achieve 55-75% Top@1 accuracy** on the same failure types that score 21% on unstructured traces. Speculative but grounded in structural analysis. [Baseline log 2026-02-20; evidence quality C]

19. **Residual hard cases (10-25%) cluster into theory-theory interactions, subtle methodology insufficiency, emergent numerical failures, and multi-component interaction failures.** These require the causal DAG and Bayesian Surprise Evaluator, not just the IR. [Baseline log 2026-02-20; evidence quality C]

**Cross-Framework and IR Synthesis**

20. ~~**DGR (Dual-Graph IR) is likely the strongest candidate for Step 5a.**~~ PROMOTED to What We Know #41. Full candidate evaluation confirms Hybrid LEL+DGR is the recommended architecture, combining LEL streaming efficiency for Stage 1 with DGR causal reasoning for Stages 2-3. Scores 94/100 vs. 82/100 for either standalone candidate. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §9-10]

21. ~~**The unified architecture can likely be incrementally implemented.**~~ PROMOTED to What We Know #42. The Hybrid candidate proves incremental implementation by construction: LEL core for Stage 1, CausalOverlay added at Stage 1→2 boundary via O(n) pass. Key constraint: LEL events must carry dag_node_ref/spec_ref/causal_refs from day one for overlay construction. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §4, §8 OQ2]

22. **Classification tables for new DSL frameworks may be partially automatable** via LLM-assisted documentation analysis, reducing the per-DSL engineering cost. Untested. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §6.4]

23. **The adapter optional methods (validate_silent_failures, extract_scf_convergence, etc.) may evolve into mandatory requirements** as empirical testing reveals which framework-specific data is essential for correct fault classification. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §5.3]

### What We Don't Know

**DSL-Specific**

1. **Whether sub-step instrumentation is achievable via OpenMM's `CustomIntegrator`** to close the temporal gap between reporter intervals. [OpenMM log 2026-02-20]

2. **The computational overhead of per-force-group energy decomposition** at every reporting interval in OpenMM. [OpenMM log 2026-02-20]

3. **How the IR should represent temporal gaps** ("state unknown between timestep X and Y") formally. [OpenMM log 2026-02-20]

4. **Whether GROMACS can produce a complete state dump at crash time.** Default behavior preserves only the last periodic checkpoint. [GROMACS log 2026-02-20]

5. **How GROMACS runtime auto-tuning (nstlist, rlist, PME) affects trace reproducibility** and whether the IR must track divergent auto-tuning. [GROMACS log 2026-02-20]

6. **What fraction of real-world VASP failures are "silent" vs. self-announcing.** Materials Project workflow logs might provide statistics. [VASP log 2026-02-20]

7. **Whether the VASP 6 REPORT file significantly closes the implementation-layer observability gap.** [VASP log 2026-02-20]

8. **Whether panedr's DataFrame (GROMACS) can serve as direct IR input** or needs abstraction for cross-framework compatibility. [GROMACS log 2026-02-20]

**IR Design**

9. **How to adapt quantitative/statistical refutation logic into a machine-checkable chain structure.** DRAT-style chains are propositional; scientific falsification is probabilistic. [RCA/FV survey 2026-02-20]

10. **How to handle trace events that span multiple dialects** (operations involving both theory-level and implementation-level concerns simultaneously). [RCA/FV survey 2026-02-20]

11. **What the minimum granularity of provenance recording is** that still enables correct fault classification. Full-granularity is an anti-pattern; DSL-API-call level needs validation. [Provenance survey 2026-02-20]

12. **Whether a single IR schema can accommodate both DFT codes (VASP) and MD codes (OpenMM, GROMACS)** or whether structural differences require fundamentally different IR designs with a common interface. [VASP log 2026-02-20; cross-framework]

13. **How convergence trajectories should be represented in the IR** (raw time series, classified patterns, or derived features). [VASP log 2026-02-20]

**Requirements and Baseline**

14. **Whether the IR should be a single composite structure or a set of joinable structures with defined interfaces.** The composite nature creates a design tension between cohesion and modularity. [LFI requirements log 2026-02-20]

15. **What classification confidence threshold (R25) separates determinate from ambiguous classifications.** This is an LFI design question, but the IR must provide input data. [LFI requirements log 2026-02-20]

16. **How cross-experiment queryability (R29) interacts with IR storage and lifecycle.** Single-experiment IR is simpler; multi-experiment requires aggregation decisions. [LFI requirements log 2026-02-20]

17. **Whether derived IR elements (R5, R17) should be stored or computed on demand.** Affects queryability performance. [LFI requirements log 2026-02-20]

18. **The exact source paper, dataset, and methodology behind the 21% figure.** Until verified, it should be treated as approximate with domain non-transferability noted. [Baseline log 2026-02-20]

19. **The candidate set size used in the 21% evaluation,** which determines whether 21% represents ~10x or ~100x above random chance. [Baseline log 2026-02-20]

20. **Whether scientific computing-specific failure analysis literature provides more directly transferable baselines** than cloud/AIOps RCA work. [Baseline log 2026-02-20]

21. **The actual RCA accuracy achievable on DSL-structured traces** — all estimates are speculative until an empirical prototype is built. [Baseline log 2026-02-20]

22. **How the success criterion should be reframed** as three-way classification accuracy rather than direct comparison to cloud RCA Top@1. [Baseline log 2026-02-20]

**Cross-Framework and IR Synthesis**

23. ~~**Which causal reasoning substrate best matches the LFI's actual query patterns.**~~ RESOLVED: per-stage answer. Stage 1: sequential search sufficient. Stages 2-3: graph traversal required. See What We Know #47. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §8 OQ3]

24. ~~**How boundary parameters should be represented in a dialect-based IR.**~~ RESOLVED: BoundaryClassification enum with PrimaryLayer/DualAnnotated/ContextDependent variants. See What We Know #45. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §1, §8 OQ4]

25. **The practical impact of VASP's closed-source ceiling.** How often does correct fault classification require information not present in vasprun.xml + OUTCAR + stdout? Needs stress-testing with real VASP failure cases. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §6.4]

26. **Whether the INCAR classification table is correct and complete.** The tag-level classification (theory/implementation/ambiguous) for ~200-300 INCAR parameters needs domain expert validation, particularly for context-dependent tags. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §6.4]

27. **What the streaming/buffering trade-off is for Stage 3.** LEL is fully streaming; DGR may require partial graph buffering; TAL may require assertion reordering. Depends on how often Stage 3 needs full-trace access vs. phase-level summaries. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §7]

**Coverage Matrix**

28. **How to formalize the quantitative prediction-observation comparison method (R17).** This is the single unresolved research element blocking Stage 3 feasibility. Must define effect size measures, divergence metrics, and tolerance thresholds for scientific predictions. Related to the DRAT propositional-to-statistical adaptation gap. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §4, §6.3]

29. ~~**Whether the LEL→DGR incremental implementation path is viable.**~~ RESOLVED: Yes. The Hybrid candidate demonstrates viability by construction. See What We Know #42. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §4, §8 OQ2]

30. **The per-force-group energy decomposition overhead in OpenMM (R6 DI).** This is the largest unknown affecting OpenMM adapter feasibility. If overhead is prohibitive, alternative R6 strategies are needed (e.g., statistical anomaly detection on total energy only). [Coverage matrix log 2026-02-20; What We Don't Know #2]

**Candidate IR Schemas**

31. ~~**DGR overlay construction cost at the Stage 1/2 boundary for megabyte-scale traces.**~~ RESOLVED: Empirically bounded with real `CausalOverlay::from_log` benchmark path at 251.82ms for 10^6 events (22.62ms at 10^5), with 1,000,000 overlay entities and 199,998 derivation edges. See What We Know #48. [Step 6 log 2026-02-21; `lel-ir-prototype/src/bench.rs`]

32. ~~**Whether HybridIR events need full DGR-compatible references from day one.**~~ NARROWED: "From day one" confirmed as safer default. Deferred resolution is a viable escape hatch via O(n) reference map pass at Stage 1→2 boundary. Remaining question: is the two-phase adapter protocol acceptable complexity for specific adapters? This is an adapter API design decision, not an IR correctness question. [Open thread resolution log 2026-02-21; LEL prototype evidence]

33. ~~**Whether the ExperimentSpec struct is sufficient for all three frameworks.**~~ NARROWED: Sufficient for all three at Stage 1. Two specific VASP Stage 3 gaps identified: (a) `ContractTerm` needs `value: Option<Value>` for machine-readable precondition checking; (b) `PredictionRecord.predicted_value` needs `KnownMatrix` or function variant in `Value` for spectral data. See items #35, #36 below. [Open thread resolution log 2026-02-21; DSL surveys]

34. ~~**Whether the OverlayEntity is sufficient for Stage 2-3 queries.**~~ RESOLVED (prototype scope): Lightweight OverlayEntity supports implemented Stage 2 confounder traversal (R14) with `by_id` lookup in place. R17/R18 remain design-aligned and unblocked by structure. See What We Know #49 and #50. [Step 6 log 2026-02-21; `lel-ir-prototype/src/overlay.rs`]

35. **Whether `ContractTerm` needs a `value: Option<Value>` field** for machine-readable precondition checking in VASP Stage 3 (e.g., POTCAR family = PBE). Currently `ContractTerm` has only `description: String`. Non-blocking for OpenMM/GROMACS Stage 1. [Open thread resolution log 2026-02-21; common.rs:94-99]

36. **Whether `Value` enum needs a `KnownMatrix` or function variant** for VASP spectral data (band structure over k-points). `PredictionRecord.predicted_value: Value` cannot represent spectral predictions with current variants. Non-blocking for OpenMM Stage 1. [Open thread resolution log 2026-02-21; common.rs:102-108]

37. ~~**Whether `EventIndexes` needs a `by_id: HashMap<EventId, usize>` index** for O(1) event lookup by ID.~~ RESOLVED: Implemented in prototype (`EventIndexes.by_id`) with insert-time population and test/serde coverage. See What We Know #49. [Step 6 log 2026-02-21; `lel-ir-prototype/src/lel.rs`, tests]

38. **Whether arena allocation provides measurable benefit for CausalOverlay construction.** NARROWED: Vec-first allocation is now validated on the real overlay path at 10^6 scale (`CausalOverlay::from_log` = 251.82ms). Arena remains deferred and should only be introduced if future profiling shows measurable allocation overhead in broader workloads. [Step 6 log 2026-02-21; `lel-ir-prototype/src/bench.rs`]

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| `codex-prompt-5b-lel-prototype.md` | Codex prompt to produce the LEL IR Rust crate prototype (Step 5b) | Complete | Specifies LEL core types (§1/§2), OpenMM mock adapter, builder helpers, 11 unit tests; validates event typing, layer tagging, spec separation, Hybrid upgrade path fields |
| `lel-ir-prototype/` | LEL + Hybrid CausalOverlay Rust prototype crate | Complete | Compiles clean, 29/29 tests pass, clippy zero warnings. Validates: event typing (12 EventKind variants), layer tagging, spec separation (AP1 avoidance), serde roundtrip, `by_id` indexing, CausalOverlay construction/traversal, and R14 confounder query behavior. |
| `lel-ir-prototype/src/overlay.rs` | CausalOverlay implementation (Step 6) | Complete | Implements index-only overlay entities, `from_log` O(n) construction, `transitive_ancestors` BFS traversal, and `detect_confounders` (R14) with controlled/intervention filtering and dag-node grouping. |
| `lel-ir-prototype/src/bench.rs` | CausalOverlay construction benchmark | Complete | Benchmarks real `CausalOverlay::from_log` at 4 scales (10^3-10^6). Latest result: 251.82ms overlay construction at 10^6 events (22.62ms at 10^5), confirming practical O(n) behavior. |

## Next Steps

1. ~~**Survey DSL trace formats**~~ — **COMPLETE.** OpenMM, GROMACS, and VASP surveys done. See investigation logs above and `dsl-evaluation/` analysis documents.

2. ~~**Survey existing IR designs in RCA and formal verification**~~ — **COMPLETE.** RCA/formal verification survey and provenance/workflow IR survey done. See investigation logs above and `dsl-evaluation/rca-formal-verification-ir-survey.md`, `dsl-evaluation/provenance-workflow-ir-survey.md`.

3. ~~**Map LFI three-stage audit backwards to minimum IR requirements**~~ — **COMPLETE.** 29 requirements (R1-R29) derived. See LFI audit investigation log above.

4. ~~**Characterize the 21% baseline and DSL improvement**~~ — **COMPLETE** (pending verification of source). See baseline characterization investigation log above. Key action item: verify the 21% source with web access.

5. **Draft candidate IR schemas and prototype** — **Steps 5a, 5b, 5c COMPLETE.** Three candidates (LEL, DGR, Hybrid LEL+DGR) evaluated against R1-R29, 9 anti-patterns, streaming constraints, and 7-criterion weighted framework. Recommendation: Hybrid (94/100). Step 5 outputs remain valid and are now extended by Step 6 implementation details below. See `dsl-evaluation/candidate-ir-schemas.md`, `prototypes/codex-prompt-5b-lel-prototype.md`, and investigation logs above. (Beads: athena-axc, athena-9uv)

6. **Hybrid LEL+DGR Phase 2 prototype (CausalOverlay + R14 query)** — **COMPLETE.** `by_id` index added; `src/overlay.rs` implemented with O(n) construction and BFS traversal; R14 confounder detection query implemented and tested. Crate now at 29/29 passing tests with strict clippy clean; benchmark uses real overlay path and reports 251.82ms at 10^6 events. (Tracking updates: #37 closed, #38 narrowed/validated)

**Synthesis steps needed before Step 5:**

- ~~**Step 1d: Cross-framework trace synthesis.**~~ — **COMPLETE.** Trace capability matrix, failure mode taxonomy (49 modes), trace completeness assessment, and Decision Gate 1 resolved (VASP accepted with external classification table). See `dsl-evaluation/cross-framework-synthesis.md` and investigation log above. (Bead: athena-ywn, CLOSED)

- ~~**Step 2c: Comparative IR synthesis.**~~ — **COMPLETE.** Seven-category pattern catalog, nine anti-patterns, MLIR/PROV-DM tension resolved (complementary: routing + provenance), Decision Gate 2 resolved (hybrid adaptation, MEDIUM risk). See `dsl-evaluation/ir-pattern-catalog.md` and investigation log above. (Bead: athena-tyt, CLOSED)

- ~~**Step 3b: Requirements refinement with coverage matrix.**~~ — **COMPLETE.** R1-R29 cross-referenced against trace capability matrix. Coverage matrix with six classification codes (DA/DI/ER/FU/NT/DE), gap analysis, per-stage feasibility assessment, and Decision Gate 4 (PASS) produced. See `dsl-evaluation/requirements-coverage-matrix.md` and investigation log above. (Bead: athena-rf6, CLOSED)
