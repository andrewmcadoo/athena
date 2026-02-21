# Session Handoff: Trace Semantics IR — Phase 2 Prototyping

> Generated: 2026-02-21 | Handoff #7 | Previous: handoff_006_2026-02-21_close-step-5a-open-threads.md

---

## Continuation Directive

Push trace semantics into Phase 2 prototyping: extend the LEL prototype crate toward the Hybrid LEL+DGR architecture. The three concrete tasks are (1) add `by_id` index to `EventIndexes`, (2) implement the `CausalOverlay` struct, and (3) implement one real query pattern (R14 confounder detection) against the overlay. This transitions from "IR design validated" to "IR implementation started."

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** Extend the LEL IR prototype from a validated Stage 1 data structure into a working Hybrid LEL+DGR prototype with actual CausalOverlay construction and at least one Stage 2 query (R14 confounder detection).

**Success criteria:** (a) `CausalOverlay` struct exists and is constructable from a `LayeredEventLog` via O(n) pass, (b) R14 confounder query is implementable against the overlay, (c) all existing tests still pass plus new tests for overlay construction and querying.

**Constraints:** Per CLAUDE.md — research artifacts only (prototypes directory), no production code, append-only FINDINGS.md log, cite evidence for all claims. Per ADR 001 — Rust for performance-critical components.

## Key Decisions & Rationale

1. **Hybrid LEL+DGR is the IR architecture (94/100)**
   - **Rationale:** LEL streaming for Stage 1, DGR graph traversal for Stages 2-3. Only candidate that passes all 9 anti-patterns.
   - **Alternatives rejected:** LEL standalone (82, weak R14/R18), DGR standalone (82, unnecessary graph overhead for Stage 1), TAL (deferred to query interface layer)

2. **LEL events carry DGR-compatible references from day one**
   - **Rationale:** Step 5c confirmed: "from day one" is the safer default. Deferred resolution is escape hatch via O(n) reference map at Stage 1→2 boundary. Fields compile/serialize cleanly with None/empty defaults.

3. **Overlay construction is a single O(n) pass at Stage 1→2 boundary**
   - **Rationale:** Step 5c benchmark confirmed: 80.53ms at 10^6 events. Linear scaling. ~10.7MB overlay memory. Tractable on commodity hardware.

4. **Vec-first for overlay allocation (no arena)**
   - **Rationale:** Batch O(n) construction pattern means `Vec::with_capacity(n)` achieves same cache locality as arena. Arena benefits only interleaved allocations. Deferred to Phase 2 benchmarking.

5. **Lightweight OverlayEntity (event reference + graph edges) is sufficient**
   - **Rationale:** Step 5c analyzed R14, R17, R18 query patterns. All three work with lightweight indirection. One prerequisite: `by_id: HashMap<EventId, usize>` index on EventIndexes for O(1) event lookup.

## Current State

### Completed
- **Steps 1-5c all complete.** Full investigation chain from DSL surveys → candidate schemas → LEL prototype → open thread resolution.
- **LEL prototype crate** at `research/trace-semantics/prototypes/lel-ir-prototype/`: 11/11 tests pass, clippy clean. Validates event typing, layer tagging, spec separation, Hybrid upgrade fields, serde roundtrip.
- **Benchmark binary** at `src/bench.rs`: measures overlay construction at 4 scales (10^3-10^6). Confirms O(n) tractability.
- **All 5 Step 5a open threads resolved/narrowed/deferred** with evidence. See FINDINGS.md Step 5c log entry.

### In Progress
- Nothing — clean state for Phase 2 prototyping.

### Blocked / Open Questions
- **WDK #35:** `ContractTerm` may need `value: Option<Value>` for VASP Stage 3 — non-blocking for current work.
- **WDK #36:** `Value` enum may need `KnownMatrix` variant for VASP spectral data — non-blocking.
- **WDK #37:** `EventIndexes` needs `by_id: HashMap<EventId, usize>` — **this is the first task for Phase 2**.
- **WDK #38:** Arena allocation deferred — benchmark Vec at Phase 2 scale.

## Key Code Context

**`src/lel.rs:88-96`** — EventIndexes (needs `by_id` addition):
```rust
pub struct EventIndexes {
    pub by_layer: HashMap<Layer, Vec<EventId>>,
    pub by_kind: HashMap<EventKindTag, Vec<EventId>>,
    pub by_time_range: BTreeMap<u64, EventId>,
    pub by_variable: HashMap<String, Vec<EventId>>,
    pub by_dag_node: HashMap<String, Vec<EventId>>,
}
```

**`src/lel.rs:51-85`** — TraceEvent with Hybrid upgrade fields:
```rust
pub struct TraceEvent {
    pub id: EventId,
    pub layer: Layer,
    pub boundary: BoundaryClassification,
    pub kind: EventKind,
    pub temporal: TemporalCoord,
    pub causal_refs: Vec<EventId>,
    pub dag_node_ref: Option<String>,
    pub spec_ref: Option<SpecElementId>,
    pub provenance: ProvenanceAnchor,
    pub confidence: ConfidenceMeta,
}
```

**`src/bench.rs:113-131`** — Overlay construction pattern (the O(n) pass to replicate structurally):
```rust
let mut entity_by_dag_node: HashMap<String, Vec<u64>> = HashMap::new();
let mut causal_ancestors: HashMap<u64, Vec<u64>> = HashMap::new();
for event in &log.events {
    if let Some(dag_ref) = &event.dag_node_ref {
        entity_by_dag_node.entry(dag_ref.clone()).or_default().push(event.id.0);
    }
    if !event.causal_refs.is_empty() {
        let refs: Vec<u64> = event.causal_refs.iter().map(|e| e.0).collect();
        causal_ancestors.insert(event.id.0, refs);
    }
}
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master investigation log (49 WK items, 38 WDK items) | Updated this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/Cargo.toml` | Crate manifest (added bench binary) | Modified this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/bench.rs` | Overlay construction benchmark | Created this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/lel.rs` | Core LEL types, builders, indexes | Reference — Phase 2 extends this |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/common.rs` | Shared types (Layer, Value, EventId, etc.) | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/event_kinds.rs` | 12 EventKind variants (R1-R17) | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs` | Mock OpenMM adapter | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/tests.rs` | 11 unit tests | Reference — Phase 2 adds more |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/lib.rs` | Module declarations | Reference |
| `research/trace-semantics/dsl-evaluation/candidate-ir-schemas.md` | Source of truth for Hybrid design (§4) | Reference |

## Loop State

N/A — single-session work. No Codex loop this session. The benchmark was written directly in Claude Code.

## Next Steps

1. **Read FINDINGS.md Step 5c log entry** (top of Investigation Log) and WDK items #35-38 for full context on what Phase 2 inherits.
2. **Add `by_id: HashMap<EventId, usize>` to `EventIndexes`** — update `new()`, `index_event()`, and `Default`. This is a prerequisite for CausalOverlay.
3. **Design and implement `CausalOverlay` struct** — likely in a new `overlay.rs` module. Fields: `entities: Vec<OverlayEntity>`, `entity_by_dag_node: HashMap<String, Vec<usize>>`, `causal_ancestors: HashMap<usize, Vec<usize>>`. Construction via `CausalOverlay::from_log(&LayeredEventLog)` using the O(n) pass pattern from bench.rs.
4. **Define `OverlayEntity`** — lightweight: `event_idx: usize` (index into `log.events`), `dag_node: Option<String>`, `causal_parents: Vec<usize>` (indexes into overlay entities).
5. **Implement R14 confounder query** — given a variable name, find events related to it, traverse causal ancestors, check if any ancestor's `dag_node_ref` corresponds to a controlled variable not in the experiment spec's `controlled_variables`. This is the simplest Stage 2 query that exercises the full overlay.
6. **Write tests** for overlay construction (correct entity/edge counts, round-trip against bench expectations) and R14 query (planted confounder detection).
7. **Update FINDINGS.md** — new investigation log entry for Phase 2 prototype work.

## Session Artifacts

- Benchmark binary: `research/trace-semantics/prototypes/lel-ir-prototype/src/bench.rs`
- Previous handoff: `.claude/handoffs/handoff_006_2026-02-21_close-step-5a-open-threads.md`
- Bead: athena-ebb (closed — Step 5c complete)

## Documentation Updated

No documentation updates — all project docs were current. FINDINGS.md was updated as part of the implementation work (not the handoff process).
