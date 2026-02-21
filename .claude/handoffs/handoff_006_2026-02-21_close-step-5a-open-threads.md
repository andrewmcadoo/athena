# Session Handoff: Trace Semantics IR — Close Step 5a Open Threads

> Generated: 2026-02-21 | Handoff #6 | Previous: handoff_005_2026-02-21_codex-prompt-5b-lel-prototype.md

---

## Continuation Directive

Close the 5 open threads from Step 5a (candidate IR schemas) using the now-validated LEL prototype as empirical evidence. The open threads are in FINDINGS.md "What We Don't Know" #31-34 and the Step 5a investigation log. Some can be resolved analytically from the prototype; others require targeted experiments (benchmarks, framework-specific spec analysis). Update FINDINGS.md accumulated findings as threads are closed.

---

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** Resolve the 5 open threads left from Step 5a (Hybrid LEL+DGR IR schema design) that were deferred to "during prototyping." The LEL prototype crate is now available as empirical evidence.

**Success criteria:** Each thread either (a) resolved with evidence and moved to "What We Know," (b) partially resolved with narrowed scope, or (c) confirmed as requiring further investigation with a concrete next step. FINDINGS.md updated with an investigation log entry documenting the analysis.

**Constraints:** Per CLAUDE.md — research artifacts only, no production code, append-only investigation log, cite evidence for all claims.

## Key Decisions & Rationale

1. **Hybrid LEL+DGR is the recommended IR architecture (94/100)**
   - **Rationale:** Captures LEL streaming efficiency for Stage 1 and DGR causal reasoning for Stages 2-3
   - **Alternatives rejected:** LEL standalone (82, weak R14/R18), DGR standalone (82, unnecessary graph overhead for Stage 1), TAL (deferred to query layer)

2. **LEL events carry DGR-compatible references from day one**
   - **Rationale:** Enables CausalOverlay construction without re-parsing. The prototype validates this works — `dag_node_ref`, `spec_ref`, `causal_refs` compile and serialize cleanly with `Option`/`Vec` defaults.
   - **This is open thread #32 — partially answered, needs full resolution**

3. **Step 5b scope: LEL-only with Hybrid upgrade fields preserved**
   - **Rationale:** Validate foundation before graph overlay. Prototype confirms LEL types, indexing, and serde roundtrip all work.

## Current State

### Completed
- **Steps 1-5b all complete.** Full investigation chain from DSL surveys through candidate schemas to working prototype.
- **LEL prototype crate** at `research/trace-semantics/prototypes/lel-ir-prototype/`: 7 source files, `cargo build` clean, 11/11 tests pass, `cargo clippy -- -D warnings` clean.
- **Codex prompt** at `research/trace-semantics/prototypes/codex-prompt-5b-lel-prototype.md`: produced the crate successfully on first attempt.
- **FINDINGS.md** fully updated: Status, Prototype Index (2 entries), Next Steps all reflect Step 5b completion.

### In Progress
- Nothing — clean state for open thread resolution.

### Blocked / Open Questions
The 5 open threads from Step 5a (FINDINGS.md "What We Don't Know" #31-34 + Step 5a log):

1. **#31: DGR overlay construction cost at Stage 1/2 boundary for megabyte-scale traces (10^5-10^6 events).** O(n) pass is theoretically fast but untested. Needs benchmarking.
2. **#32: Whether HybridIR events need DGR-compatible references from day one.** Prototype shows the fields compile and serialize fine with `Option::None`/empty `Vec`. But the deeper question is adapter complexity — does the adapter need to resolve DAG/spec refs during Stage 1?
3. **#33: Whether ExperimentSpec is sufficient for all three frameworks.** Generic struct may need framework-specific extensions (VASP multi-file fusion, OpenMM createSystem chain, GROMACS grompp results).
4. **#34: Whether lightweight OverlayEntity (reference to LEL event + graph edges) is sufficient for Stage 2-3 queries.** Indirection cost unknown.
5. **Step 5a log thread: Arena allocation strategy for CausalOverlay.** Cache friendliness of overlay→LEL event indirection needs benchmarking.

## Key Code Context

**`research/trace-semantics/prototypes/lel-ir-prototype/src/lel.rs`** — TraceEvent struct with Hybrid upgrade fields:
```rust
pub struct TraceEvent {
    pub id: EventId,
    pub layer: Layer,
    pub boundary: BoundaryClassification,
    pub kind: EventKind,
    pub temporal: TemporalCoord,
    pub causal_refs: Vec<EventId>,        // Hybrid upgrade path
    pub dag_node_ref: Option<String>,     // Hybrid upgrade path
    pub spec_ref: Option<SpecElementId>,  // Hybrid upgrade path
    pub provenance: ProvenanceAnchor,
    pub confidence: ConfidenceMeta,
}
```

**`research/trace-semantics/prototypes/lel-ir-prototype/src/lel.rs`** — EventIndexes (relevant to overlay construction cost):
```rust
pub struct EventIndexes {
    pub by_layer: HashMap<Layer, Vec<EventId>>,
    pub by_kind: HashMap<EventKindTag, Vec<EventId>>,
    pub by_time_range: BTreeMap<u64, EventId>,
    pub by_variable: HashMap<String, Vec<EventId>>,
    pub by_dag_node: HashMap<String, Vec<EventId>>,
}
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master investigation log (47+ accumulated findings, 34 What We Don't Know items) | Updated this session |
| `research/trace-semantics/dsl-evaluation/candidate-ir-schemas.md` | Source of truth for all IR types (§1 common, §2 LEL, §3 DGR, §4 Hybrid) | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/` | Working LEL IR Rust crate (7 files, 11 tests) | Created this session (via Codex) |
| `research/trace-semantics/prototypes/codex-prompt-5b-lel-prototype.md` | Codex prompt that produced the crate | Created this session |
| `decisions/001-python-rust-core.md` | ADR: Rust for Trace Semantics Engine | Reference |

## Loop State

- **Iteration:** 1 (Codex prompt → Codex implementation → verified)
- **Last prompt to Codex:** `codex-prompt-5b-lel-prototype.md` — full crate specification with types, adapter, builders, 11 tests
- **Codex result:** Clean implementation. Minor adjustments for clippy compliance (unused imports, assertion style). All verification gates pass.
- **Claude review findings:** Code review confirmed 100% plan coverage. No missing items.

## Next Steps

1. **Read FINDINGS.md** — specifically the Step 5a investigation log "Open Threads" (line ~66) and "What We Don't Know" #31-34 (lines ~908-914).
2. **For thread #31 (overlay construction cost):** Write a benchmark in the prototype crate that generates 10^3, 10^4, 10^5, 10^6 synthetic events and measures a simulated overlay construction pass (iterate events, build HashMap indexes from dag_node_ref/causal_refs). Report wall-clock time and memory.
3. **For thread #32 (references from day one):** Analyze the prototype's adapter code — quantify what changes if the adapter must populate dag_node_ref/spec_ref during initial construction vs. leaving them None. This is an analytical question, not a benchmark.
4. **For thread #33 (ExperimentSpec sufficiency):** Compare the generic ExperimentSpec fields against what each framework's adapter would actually need to populate. Review OpenMM createSystem, GROMACS grompp, VASP INCAR/POSCAR inputs against the struct.
5. **For threads #34 and arena allocation:** These are blocked until CausalOverlay types exist. Can be analyzed theoretically or deferred to a Phase 2 prototype. Document the analysis either way.
6. **Write investigation log entry** in FINDINGS.md with findings for each thread. Update "What We Don't Know" items as they resolve.

## Session Artifacts

- Codex prompt: `research/trace-semantics/prototypes/codex-prompt-5b-lel-prototype.md`
- LEL prototype crate: `research/trace-semantics/prototypes/lel-ir-prototype/`
- Previous handoff: `.claude/handoffs/handoff_005_2026-02-21_codex-prompt-5b-lel-prototype.md`

## Documentation Updated

| Document | Change Summary | Status |
|----------|---------------|--------|
| `research/trace-semantics/FINDINGS.md` | Status line updated (Steps 1-5b complete). Prototype Index: prompt status→Complete, added crate entry. Next Steps §5: reflects 5b completion with both beads. | Approved |
