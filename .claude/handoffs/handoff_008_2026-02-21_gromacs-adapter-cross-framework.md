# Session Handoff: Trace Semantics IR — GROMACS Adapter

> Generated: 2026-02-21 | Handoff #8 | Previous: handoff_007_2026-02-21_phase2-hybrid-prototyping.md

---

## Continuation Directive

Implement Step 9 — a GROMACS adapter for cross-framework generalization. This is the first real-DSL adapter for the LEL IR prototype, parsing actual GROMACS simulation output (`.log`, `.mdp`, `.edr` files) into `LayeredEventLog` events. The goal is to stress-test the IR types against trace data they weren't specifically shaped around.

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** Build a GROMACS adapter that converts real GROMACS simulation traces into `LayeredEventLog` instances, proving the IR generalizes beyond the mock OpenMM adapter. This validates Architectural Constraint #1 (DSL-only environments).

**Success criteria:** (a) GROMACS adapter parses at least `.mdp` (parameter files) and `.log` (execution logs) into typed TraceEvents, (b) parsed events pass through `CausalOverlay::from_log` and support R14/R17/R18 queries, (c) all existing 44 tests still pass plus new adapter-specific tests.

**Constraints:** Per CLAUDE.md — research artifacts only (prototypes directory), no production code, append-only FINDINGS.md log. Per ADR 001 — Rust for performance-critical components.

## Key Decisions & Rationale

1. **Hybrid LEL+DGR is the IR architecture (94/100)**
   - **Rationale:** LEL streaming for Stage 1, DGR graph traversal for Stages 2-3. Only candidate that passes all 9 anti-patterns.
   - **Alternatives rejected:** LEL standalone (82), DGR standalone (82), TAL (deferred)

2. **`prediction_id` String→SpecElementId mismatch resolved at query time**
   - **Rationale:** `prediction_id.parse::<u64>().ok().map(SpecElementId)` in `compare_predictions`. Avoids cascading changes to `event_kinds.rs`. Deferred to production ADR.

3. **Existing adapter pattern: `adapter.rs` mock OpenMM**
   - **Rationale:** The mock adapter demonstrates the adapter→LEL conversion pattern. A GROMACS adapter should follow the same structural approach (function taking raw input → returning `LayeredEventLog`).

4. **All three Stage 2-3 queries are implemented and tested**
   - R14 confounder detection, R17 prediction-observation comparison, R18 causal implication traversal
   - This means the GROMACS adapter just needs to produce valid `LayeredEventLog` instances — all query infrastructure already works.

## Current State

### Completed
- **Steps 1-7 all complete.** Full chain: DSL surveys → candidate schemas → LEL prototype → overlay + R14 → R17 + R18.
- **LEL prototype crate** at `research/trace-semantics/prototypes/lel-ir-prototype/`: 44/44 tests pass, clippy clean.
- **CausalOverlay** with three query methods: `detect_confounders` (R14), `compare_predictions` (R17), `implicate_causal_nodes` (R18).
- **Full Stage 2-3 query surface validated.** Three-way fault classification (Theory/Methodology/Implementation) demonstrated end-to-end.

### In Progress
- Nothing — clean state for Step 9.

### Blocked / Open Questions
- **WDK #35:** `ContractTerm` may need `value: Option<Value>` for VASP Stage 3 — non-blocking.
- **WDK #36:** `Value` enum may need `KnownMatrix` variant for VASP spectral data — non-blocking.
- **WDK #39 (new):** `prediction_id` String vs SpecElementId type mismatch — deferred to production ADR.
- **Open:** What GROMACS trace files to target? `.mdp` (input parameters) and `.log` (execution output) are the minimum viable set. `.edr` (energy records) would exercise `EnergyRecord` EventKind but is binary format (requires `xdrfile` or equivalent parsing).

## Key Code Context

**`src/overlay.rs:34-67`** — New R17/R18 result structs:
```rust
pub struct PredictionComparison {
    pub comparison_event_idx: usize,
    pub prediction_id: Option<SpecElementId>,
    pub variable: String,
    pub outcome: ComparisonOutcome,
    pub is_falsified: bool,
    pub dag_node: Option<String>,
}

pub struct ImplicatedNode {
    pub dag_node: String,
    pub layer: Layer,
    pub causal_distance: usize,
    pub ancestor_event_indices: Vec<usize>,
}
```

**`src/adapter.rs`** — Existing mock adapter pattern (follow this structure for GROMACS):
- `mock_openmm_trace() -> LayeredEventLog` — builds spec, events, and indexes via builder pattern.
- Uses `TraceEventBuilder::new().layer(...).kind(...).temporal(...).build()` fluent API.
- Uses `LayeredEventLogBuilder::new(experiment_ref, spec).add_event(e1).add_event(e2).build()`.

**`src/event_kinds.rs:12-114`** — 12 EventKind variants. GROMACS adapter should map to at minimum:
- `ParameterRecord` (from `.mdp` file parameters)
- `ExecutionStatus` (from `.log` completion status)
- `NumericalStatus` (from `.log` energy drift / NaN detection)
- `EnergyRecord` (from `.log` energy decomposition lines)

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master investigation log (53 WK, 39 WDK items) | Updated this session (Step 7 entry) |
| `prototypes/lel-ir-prototype/src/overlay.rs` | CausalOverlay + R14/R17/R18 queries | Modified this session |
| `prototypes/lel-ir-prototype/src/tests/mod.rs` | 44 unit tests | Modified this session |
| `prototypes/lel-ir-prototype/src/lel.rs` | Core LEL types, builders, indexes | Reference |
| `prototypes/lel-ir-prototype/src/common.rs` | Shared types (Layer, Value, EventId, etc.) | Reference |
| `prototypes/lel-ir-prototype/src/event_kinds.rs` | 12 EventKind variants (R1-R17) | Reference |
| `prototypes/lel-ir-prototype/src/adapter.rs` | Mock OpenMM adapter — follow this pattern | Reference |
| `prototypes/lel-ir-prototype/src/bench.rs` | Overlay construction benchmark | Reference |
| `prototypes/lel-ir-prototype/src/lib.rs` | Module declarations | Reference |

## Loop State

N/A — single-session work. No Codex loop.

## Next Steps

1. **Read FINDINGS.md Step 7 log entry** (top of Investigation Log) for full context on what's proven.
2. **Research GROMACS trace formats** — Survey `.mdp` file syntax (INI-like key=value), `.log` file structure (timestep summaries, energy output, performance counters), and optionally `.edr` (XDR binary energy).
3. **Create `gromacs_adapter.rs`** in `prototypes/lel-ir-prototype/src/` — function signature like `parse_gromacs_mdp(content: &str) -> Vec<TraceEvent>` and `parse_gromacs_log(content: &str) -> Vec<TraceEvent>`.
4. **Map GROMACS concepts to EventKind variants** — `.mdp` parameters → `ParameterRecord`, log energy lines → `EnergyRecord`, log completion → `ExecutionStatus`, NaN/energy-drift → `NumericalStatus`.
5. **Assign layers** — `.mdp` parameters are typically Methodology (integrator, thermostat) or Theory (force field, cutoffs). Execution status is Implementation. This tests the three-way classification with real data.
6. **Write tests using sample GROMACS output** — embed small representative `.mdp` and `.log` snippets as string constants in tests.
7. **End-to-end test** — Parse GROMACS trace → build LayeredEventLog → construct CausalOverlay → run R14/R17/R18 queries → verify results make physical sense.
8. **Update FINDINGS.md** — Step 9 log entry documenting what the GROMACS adapter revealed about IR generality.

## Session Artifacts

- Prompt: `.claude/prompts/prompt_003_2026-02-21_r17-r18-overlay-queries.md`
- Bead: athena-c9q (closed — Step 7 complete)
- Previous handoff: `.claude/handoffs/handoff_007_2026-02-21_phase2-hybrid-prototyping.md`
- Commit: `1c1b140` — "Complete Step 7: R17+R18 queries on CausalOverlay"

## Documentation Updated

No documentation updates — all project docs were current.
