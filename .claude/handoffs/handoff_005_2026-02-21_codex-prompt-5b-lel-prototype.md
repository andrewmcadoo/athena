# Session Handoff: Trace Semantics IR — Step 5b Codex Prompt Design

> Generated: 2026-02-21 | Handoff #5 | Previous: handoff_004_2026-02-20_plan-implement-5a-candidate-ir-schemas.md

---

## Continuation Directive

Design a prompt for Codex to implement Step 5b: a Rust prototype of the LEL (Layered Event Log) IR core on OpenMM traces. The prompt must be self-contained, referencing only the candidate-ir-schemas.md document (Section 2: LEL structures, Section 1: common foundation) and ADR 001 (Rust + PyO3). Scope: R1-R7 + R19 + R20 + R21 only. Must include `dag_node_ref`, `spec_ref`, and `causal_refs` fields on TraceEvent to preserve the Hybrid upgrade path.

---

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** Produce a Codex-ready implementation prompt for the LEL IR prototype that validates event typing, layer tagging, and specification separation on OpenMM simulation traces.

**Success criteria:** A prompt that Codex can execute to produce a compilable Rust crate with: (1) the common structural foundation types from candidate-ir-schemas.md §1, (2) the LEL core structures from §2, (3) a minimal OpenMM adapter stub, (4) unit tests demonstrating Stage 1 event construction and layer-based filtering.

**Constraints:**
- Rust crate in `research/trace-semantics/prototypes/` (research artifact, not production)
- PyO3 not required for prototype — pure Rust with serde for serialization
- Must compile. Must have tests. Must pass `cargo test`.
- Per CLAUDE.md: prototypes are throwaway research artifacts referenced from FINDINGS.md

## Key Decisions & Rationale

1. **Hybrid LEL+DGR is the recommended IR architecture (94/100)**
   - **Rationale:** Captures LEL streaming efficiency for Stage 1 (the common classification path — ~41% of failures are implementation-layer) AND DGR causal reasoning for Stages 2-3 (ATHENA's differentiating value)
   - **Alternatives rejected:** LEL standalone (82/100, weak on R14 confounders and R18 causal implication), DGR standalone (82/100, unnecessary graph overhead for Stage 1), TAL standalone (deferred to query-layer role — highest novelty risk, no close precedent)

2. **Step 5b prototypes LEL first, not full Hybrid**
   - **Rationale:** Validate the foundation (event typing, layer tagging, spec separation) with minimal complexity before adding graph overlay. Stage 1 is the most tractable stage — all three frameworks provide sufficient data.
   - **Critical constraint:** LEL events MUST include `dag_node_ref`, `spec_ref`, `causal_refs` fields even though Stage 1 doesn't use them. These enable future CausalOverlay construction without re-parsing.

3. **OpenMM as target framework for prototype**
   - **Rationale:** Cleanest API boundary (R19=DA, API-enforced separation at `ForceField.createSystem()`). Best structural foundation for validating layer tagging. Highest instrumentation burden (10 DI cells) but most extensible reporter API.

4. **TAL adopted as LFI query interface, not storage format**
   - **Rationale:** Coverage matrix showed TAL's assertion-checking pattern functions identically as a query layer over LEL/DGR substrates. Its core strength (sequential audit assertions with evidence chains) doesn't require standalone IR storage.

5. **BoundaryClassification enum with 3 variants for boundary parameters**
   - **Rationale:** Avoids both a fourth "boundary" layer (breaks three-stage audit) and entity duplication. PrimaryLayer/DualAnnotated/ContextDependent covers the full spectrum from unambiguous to context-dependent parameters.

6. **R17 comparison resolved as pluggable structural container**
   - **Rationale:** ComparisonResult + DivergenceMeasure enum (6 variants) provides the slot. The comparison method is LFI logic, not IR structure. Decouples IR prototyping from the R17 formalization research.

## Current State

### Completed
- **Step 5a** (bead athena-axc, CLOSED): `candidate-ir-schemas.md` created (1,171 lines, 11 sections). Three candidates evaluated. Hybrid recommended. All 4 open questions resolved. All 29 requirements mapped. All 9 anti-patterns evaluated.
- **FINDINGS.md** updated: investigation log entry (10 findings), 7 new What We Know items (#41-47), 3 What We Don't Know items resolved (#23, #24, #29), 4 new What We Don't Know items added (#31-34), Status and Next Steps updated.
- **All prerequisites:** Steps 1-4, synthesis steps 1d/2c/3b, Decision Gates 1/2/4 all PASS.

### In Progress
- Nothing — clean state for Codex prompt design.

### Blocked / Open Questions
- **For the Codex prompt specifically:** How much of the common foundation (§1) to include? All 7 types are needed for a complete prototype, but some (DivergenceMeasure, ComparisonResult) are Stage 3 and could be stubbed.
- **Prototype scope boundary:** Should the Codex prototype include a real OpenMM trace parser, or just an adapter trait + mock data? Real parsing requires pymatgen/MDAnalysis knowledge. Mock data is simpler and sufficient for structural validation.

## Key Code Context

**`candidate-ir-schemas.md` §1 — Common Foundation Types (the types Codex must implement):**

Core types: `Layer` (3-variant enum), `BoundaryClassification` (3-variant enum), `ObservationMode` (2-variant enum), `Value` (4-variant enum with `Havoc`), `HavocReason` (5-variant enum), `TemporalCoord` (struct: simulation_step, wall_clock_ns, logical_sequence), `ProvenanceAnchor` (struct: source_file, source_location, raw_hash), `SourceLocation` (4-variant enum), `ExperimentRef` (struct: experiment_id, cycle_id, hypothesis_id), `ConfidenceMeta` (struct: completeness, field_coverage, notes), `Completeness` (4-variant enum).

**`candidate-ir-schemas.md` §2 — LEL Core Structures (Codex must implement these):**

Core types: `LayeredEventLog` (struct: experiment_ref, spec, events, indexes), `ExperimentSpec` (struct: preconditions, postconditions, predictions, interventions, controlled_variables, dag_refs, provenance), `TraceEvent` (struct: id, layer, boundary, kind, temporal, causal_refs, dag_node_ref, spec_ref, provenance, confidence), `EventKind` (12-variant enum mapping to R1-R7, R8, R12, R16, R17 + convergence/state/energy), `EventIndexes` (struct: by_layer, by_kind, by_time_range, by_variable, by_dag_node).

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/dsl-evaluation/candidate-ir-schemas.md` | **Primary input** — candidate schemas, common foundation, LEL structures | Created this session |
| `research/trace-semantics/FINDINGS.md` | Master investigation log + accumulated findings (47 items) | Updated this session |
| `research/trace-semantics/dsl-evaluation/ir-pattern-catalog.md` | Pattern sources for the schemas (7 patterns, 9 anti-patterns) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/requirements-coverage-matrix.md` | R1-R29 coverage codes per framework | Input (unchanged) |
| `decisions/001-python-rust-core.md` | ADR: Rust for Trace Semantics Engine | Reference |

## Loop State

- **Iteration:** 1 (first Codex prompt)
- **Last prompt to Codex:** Not yet written — this handoff is for designing it
- **Codex result:** N/A
- **Claude review findings:** N/A

## Next Steps

1. **Read `candidate-ir-schemas.md` §1 and §2** — these are the exact type definitions the Codex prompt must reference.
2. **Design the Codex prompt** with these elements:
   - Crate structure: `research/trace-semantics/prototypes/lel-ir-prototype/` with `Cargo.toml`, `src/lib.rs`, module files
   - Common foundation types from §1 (all of them — they're foundational)
   - LEL core structures from §2 (LayeredEventLog, ExperimentSpec, TraceEvent, EventKind, EventIndexes)
   - Stage 1 scope: only EventKind variants for R1-R7, R19-R21. Stage 2/3 variants can be stubbed or included as-is (they're defined in the schema)
   - OpenMM adapter trait (trait definition + mock implementation producing sample events)
   - Builder/constructor helpers for TraceEvent
   - Unit tests: construct events, add to log, query by layer, query by event kind, verify temporal ordering
   - `dag_node_ref`, `spec_ref`, `causal_refs` fields present on TraceEvent (populated as None/empty in tests — Hybrid upgrade path)
3. **Reference FINDINGS.md Prototype Index** — the prototype must be added there once created
4. **Include verification criteria** in the prompt: `cargo build`, `cargo test`, `cargo clippy`

## Session Artifacts

- Candidate IR schemas: `research/trace-semantics/dsl-evaluation/candidate-ir-schemas.md`
- Previous handoff: `.claude/handoffs/handoff_004_2026-02-20_plan-implement-5a-candidate-ir-schemas.md`

## Documentation Updated

No documentation updates — all project docs were current.
