# Prompt: LEL CausalOverlay and R14 Confounder Query

> Generated: 2026-02-21 | Prompt #1 | Framework: RISEN

---

## Session Goal

Extend the LEL IR prototype crate with a `CausalOverlay` struct (the DGR half of the Hybrid LEL+DGR architecture), including a `by_id` index prerequisite, O(n) overlay construction, and an R14 confounder detection query — transitioning the research from "IR design validated" to "IR implementation started."

## Framework Selection

- **Chosen:** RISEN (Role, Instructions, Steps, End Goal, Narrowing)
- **Rationale:** Complex multi-step implementation with strict dependency chain (5 sequential tasks), clear methodology (Rust prototype extension), explicit constraints (research artifacts only, append-only FINDINGS.md), and hard boundaries (no production code, no arena allocation)
- **Alternatives considered:** TIDD-EC (good for constraints but the sequential dependency chain dominates), Chain of Thought (task is procedural, not a reasoning problem)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Unambiguous goal, exact file paths and struct definitions |
| Specificity | 9/10 | Quantitative criteria (11+18 tests, zero clippy), exact type signatures |
| Context | 9/10 | Full architecture context, prior work, key decisions, code references |
| Completeness | 9/10 | Covers what/why/how, verification steps, FINDINGS.md protocol |
| Structure | 9/10 | Clear RISEN hierarchy, numbered steps with substeps, dependency chain |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a Rust systems programmer extending a research prototype for ATHENA's Trace Semantics Engine. You have expertise in IR (intermediate representation) design, causal graph structures, and scientific computing trace analysis. You are working within an existing crate (`research/trace-semantics/prototypes/lel-ir-prototype/`) that has 11 passing tests, clean clippy, and validates the LEL (Layered Event Log) data model. Your job is to add the CausalOverlay — the graph-traversal half of the Hybrid LEL+DGR architecture.

INSTRUCTIONS:
- Follow the approved implementation plan and execute tasks in strict dependency order (Task 1 → 2 → 3 → 4 → 5)
- Each task must compile, pass all tests, and produce zero clippy warnings before moving to the next
- Reuse existing patterns: `TraceEventBuilder` fluent builder, `reset_event_id_counter()` in test setup, `test_provenance()`/`test_spec()`/`test_experiment_ref()` helpers from `src/tests/mod.rs`
- Use 1:1 entity mapping for the overlay (entity index == event index in `log.events`). Events without `dag_node_ref` get `dag_node: None` but still participate in causal chains
- Index-only design: `CausalOverlay` stores `usize` positions, not references to the log. The log is passed to query methods as `&LayeredEventLog`
- Use `filter_map` when resolving `causal_refs` via `by_id` — silently skip dangling EventId references. Post-construction, all `causal_parents` indices are guaranteed valid
- Compute transitive ancestors on-demand via BFS, not pre-computed closure. The `transitive_ancestors` method returns all nodes reachable via `causal_parents` edges (transitive closure). Do not include the start node in results. Cycles are impossible in a well-formed DAG
- All new types must derive `Debug, Clone, Serialize, Deserialize` (matching existing crate pattern)

STEPS:

1. **Task 1 — Add `by_id: HashMap<EventId, usize>` to `EventIndexes`** (`src/lel.rs`)
   a. Add `pub by_id: HashMap<EventId, usize>` field to the `EventIndexes` struct (EventId already derives Hash+Eq in common.rs:8)
   b. Initialize `by_id: HashMap::new()` in `EventIndexes::new()`
   c. Change `index_event` signature to `(&mut self, event: &TraceEvent, position: usize)` and add `self.by_id.insert(event.id, position)` at the top
   d. Update `LayeredEventLogBuilder::add_event` to pass `self.events.len()` as position before pushing
   e. Add 3 tests: `test_by_id_index_populated`, `test_by_id_index_correct_positions`, `test_by_id_serde_roundtrip`
   f. Run `cargo test && cargo clippy` — verify 14+ tests pass, zero warnings

2. **Task 2 — Create `CausalOverlay` struct** (new `src/overlay.rs`)
   a. Create `src/overlay.rs` with:
      - `OverlayEntity { event_idx: usize, dag_node: Option<String>, causal_parents: Vec<usize> }`
      - `CausalOverlay { entities: Vec<OverlayEntity>, entity_by_dag_node: HashMap<String, Vec<usize>> }`
   b. Implement `CausalOverlay::from_log(log: &LayeredEventLog) -> Self` — single O(n) pass, `Vec::with_capacity(n)`, resolve `causal_refs` via `log.indexes.by_id`
   c. Implement accessors: `len()`, `is_empty()`, `entity(idx) -> Option<&OverlayEntity>`
   d. Implement `transitive_ancestors(start_idx: usize) -> Vec<usize>` — BFS with `HashSet` visited tracking, excludes start node
   e. Add `pub mod overlay;` to `src/lib.rs`
   f. Add 8 tests: empty log, 1:1 mapping, dag_node index, causal_parents resolution, dangling ref skip, linear chain ancestors, diamond ancestors, serde roundtrip
   g. Run `cargo test && cargo clippy`

3. **Task 3 — Implement R14 confounder detection query** (add to `src/overlay.rs`)
   a. Add `ConfounderCandidate { dag_node: String, observable_ancestor_events: Vec<usize>, intervention_ancestor_events: Vec<usize> }`
   b. Implement `CausalOverlay::detect_confounders(&self, log: &LayeredEventLog, observable_var: &str, intervention_var: &str) -> Vec<ConfounderCandidate>`
   c. Algorithm (in this order):
      1. If observable_var or intervention_var not in `log.indexes.by_variable`, return `Vec::new()`
      2. Resolve variable EventIds to positions via `log.indexes.by_id`
      3. BFS transitive ancestors for all observable event positions
      4. BFS transitive ancestors for all intervention event positions
      5. Compute intersection of ancestor sets
      6. For each ancestor in intersection:
         - SKIP if `dag_node` is None
         - SKIP if `dag_node` matches intervention_var
         - SKIP if `dag_node` is in `spec.controlled_variables` parameter names
      7. Group remaining ancestors by `dag_node` → one `ConfounderCandidate` per unique dag_node
   d. Add `debug_assert_eq!(self.entities.len(), log.events.len())` at method entry
   e. Add 7 tests: all controlled, uncontrolled detected, intervention excluded, no common ancestors, unknown variable (returns empty), multiple confounders, transitive chain
   f. Run `cargo test && cargo clippy`

4. **Task 4 — Update `bench.rs`** (`src/bench.rs`)
   a. Replace ad-hoc HashMap overlay construction (lines 122-141) with `CausalOverlay::from_log(&log)`
   b. Update reporting to use `overlay.len()`, `overlay.entity_by_dag_node.len()`, and edge count from `overlay.entities.iter().map(|e| e.causal_parents.len()).sum::<usize>()`
   c. Run `cargo run --bin bench` — verify O(n) scaling at 10^6

5. **Task 5 — Update `FINDINGS.md`** (`research/trace-semantics/FINDINGS.md`)
   a. Append Step 6 investigation log entry at top of Investigation Log (reverse chronological)
   b. Scope: Hybrid LEL+DGR Phase 2 prototype — CausalOverlay struct, by_id index, R14 query
   c. Include: Method (direct implementation), Findings (what worked, performance), Implications (what this unblocks for Phase 3), Open Threads
   d. Update Prototype Index table with `src/overlay.rs` entry
   e. Update WDK items: close #37 (by_id index implemented), note #38 status (Vec allocation validated)
   f. Update Accumulated Findings: move relevant items to What We Know

END GOAL:
- `CausalOverlay` struct exists in `src/overlay.rs` and is constructable from any `LayeredEventLog` via `CausalOverlay::from_log()` in a single O(n) pass
- R14 confounder detection query works against the overlay, correctly identifying uncontrolled causal influences by traversing the causal graph
- All 11 existing tests still pass (no regressions) plus ~18 new tests pass
- `cargo clippy` produces zero warnings across the entire crate
- `cargo run --bin bench` uses real `CausalOverlay::from_log` and confirms O(n) scaling
- FINDINGS.md updated with Step 6 entry documenting the Phase 2 prototype work

NARROWING:
- Do NOT write production code — this is a research prototype in `prototypes/` per CLAUDE.md
- Do NOT modify `src/common.rs` or `src/event_kinds.rs` — no type changes needed for this work
- Do NOT use arena allocation (deferred per WDK #38 — benchmark Vec first)
- Do NOT pre-compute transitive closure in CausalOverlay — compute on-demand via BFS
- Do NOT use lifetime parameters on CausalOverlay or OverlayEntity — index-only design avoids this
- Do NOT add external dependencies to Cargo.toml — only `serde` and `serde_json` needed
- Out of scope: WDK #35 (ContractTerm value field for VASP), WDK #36 (KnownMatrix variant)
- Out of scope: Selective entity mapping optimization — 1:1 mapping is correct for prototype
- Do NOT edit previous FINDINGS.md investigation log entries — append-only protocol
- Do NOT soften limitations or skip documenting known constraints in FINDINGS.md
```

---

## Review Findings

### Issues Addressed
- **Warning #1 (Algorithm filtering):** Expanded R14 algorithm in Task 3c to explicit 7-step pipeline with filtering precedence and grouping-by-dag_node behavior
- **Warning #2 (Unknown variable):** Added explicit "return `Vec::new()`" as step 1 of the algorithm
- **Suggestion #2 (BFS spec):** Added "excludes start node" and "cycles impossible in well-formed DAG" to Instructions
- **Suggestion #4 (Invariant):** Added "Post-construction, all `causal_parents` indices are guaranteed valid" to Instructions

### Remaining Suggestions
- ConfounderCandidate serde roundtrip test not explicitly listed (could add to Task 3 tests)
- FINDINGS.md Step 6 could specify whether to update ARCHITECTURE.md implications (low priority)
- ConfounderCandidate grouping dedup behavior (observable/intervention ancestor event lists) could be more explicit

## Usage Notes

- **Best used with:** Claude Code in an ATHENA research session with the LEL prototype crate already built and passing
- **Adjust for:** If using as a Codex prompt, include the full file contents inline rather than referencing paths
