# Prompt: R17+R18 CausalOverlay Query Implementation

> Generated: 2026-02-21 | Framework: RISEN

---

## Session Goal

Implement R17 (prediction-observation comparison) and R18 (causal implication traversal) query methods on the CausalOverlay struct in the LEL IR prototype, with 15 new tests, clippy compliance, and a FINDINGS.md update — completing the Stage 2-3 query surface.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation process with clear role (Rust systems programmer), explicit sequential steps (types → R17 → tests → helper → R18 → tests → quality gates → docs), measurable end goal (44/44 tests, zero clippy warnings), and strong narrowing constraints (prototype scope, no cascading changes). RISEN maps perfectly.
- **Alternatives considered:** TIDD-EC (good for precision constraints but the sequential methodology is the dominant structure); Chain of Thought (implicit in coding tasks but doesn't capture role/narrowing dimensions).

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal is unambiguous: add 2 methods, 1 helper, 2 structs, 15 tests |
| Specificity | 10/10 | Exact types, signatures, field names, test names, verification commands |
| Context | 9/10 | Full codebase structure provided; actual type definitions referenced |
| Completeness | 9/10 | Covers what/why/how, output verification, narrowing constraints |
| Structure | 10/10 | RISEN components cleanly separate concerns; steps are sequential with dependencies noted |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a Rust systems programmer implementing query methods on a research prototype for a falsification-driven AI co-scientist (ATHENA). You have expertise in: Rust idioms (BFS graph traversal, BTreeMap grouping, serde-derived structs), the existing LEL IR codebase (CausalOverlay, LayeredEventLog, EventKind, EventIndexes), and the prototype's R14/R17/R18 query requirements. You operate within an append-only research codebase governed by FINDINGS.md protocol.

INSTRUCTIONS:
1. All code changes go in the existing `overlay.rs` and `tests/mod.rs` files — no new files.
2. Follow the existing pattern established by `detect_confounders` and `ConfounderCandidate`: public method on `CausalOverlay` taking `&self` + `&LayeredEventLog`, returning a `Vec<T>` of a serde-derived result struct.
3. The `prediction_id` field in `EventKind::ComparisonResult` is `String`, but `PredictionRecord.id` is `SpecElementId(u64)`. Resolve this mismatch at query time via `prediction_id.parse::<u64>().ok().map(SpecElementId)`. Do not modify `event_kinds.rs`.
4. Do not modify `transitive_ancestors` — it has 4 existing tests. Create a private `ancestors_with_depth` helper that extends the BFS pattern with depth tracking.
5. Run `cargo test` after each batch of tests (Steps 3, 6) and `cargo clippy --all-targets --all-features -- -D warnings` at the end. All 44 tests must pass, zero clippy warnings.
6. For FINDINGS.md: append-only log (new entry at top of Investigation Log), update Accumulated Findings (What We Know / What We Don't Know), update Prototype Index.

STEPS:
1. **Add imports and types to `overlay.rs`** (after `ConfounderCandidate`):
   - Add `use crate::common::{ComparisonOutcome, Layer, SpecElementId};` and `use crate::event_kinds::EventKind;` to the imports section.
   - Add `PredictionComparison` struct with fields: `comparison_event_idx: usize`, `prediction_id: Option<SpecElementId>`, `variable: String`, `outcome: ComparisonOutcome`, `is_falsified: bool`, `dag_node: Option<String>`.
   - Add `ImplicatedNode` struct with fields: `dag_node: String`, `layer: Layer`, `causal_distance: usize`, `ancestor_event_indices: Vec<usize>`.
   - Both structs derive `Debug, Clone, Serialize, Deserialize`.

2. **Implement R17 `compare_predictions` method** on `CausalOverlay`:
   - Signature: `pub fn compare_predictions(&self, log: &LayeredEventLog) -> Vec<PredictionComparison>`
   - Build `HashMap<SpecElementId, &PredictionRecord>` from `log.spec.predictions`.
   - Get ComparisonResult event IDs from `log.indexes.by_kind` using `EventKindTag::ComparisonResult`. This returns `Vec<EventId>`. Guard for absent key with early return of empty Vec.
   - For each `EventId`: resolve to `usize` position via `log.indexes.by_id[&event_id]`, then access `log.events[position]`. Destructure `EventKind::ComparisonResult { prediction_id, observation_id: _, result }`.
   - Parse `prediction_id` String→u64→SpecElementId, lookup matching prediction in the HashMap, build `PredictionComparison` with `is_falsified = !result.agreement`, `variable` from matched prediction (or `"unknown".to_string()` if unresolvable), `dag_node` from the event's `dag_node_ref`.

3. **Write 7 R17 tests** in `tests/mod.rs`:
   - Add a `test_spec_with_predictions(predictions: Vec<PredictionRecord>) -> ExperimentSpec` helper function (not a test itself — clones the existing `test_spec()` pattern and replaces the predictions field).
   - `test_compare_predictions_empty_log` — no events → empty result
   - `test_compare_predictions_no_comparison_events` — events present but no ComparisonResult → empty
   - `test_compare_predictions_matched_agreement` — prediction matches, agreement=true, is_falsified=false
   - `test_compare_predictions_matched_falsified` — prediction matches, agreement=false, is_falsified=true, divergence present
   - `test_compare_predictions_unresolvable_prediction_id` — malformed string → prediction_id=None
   - `test_compare_predictions_multiple_predictions` — two predictions, one passes one fails
   - `test_compare_predictions_with_dag_node_ref` — dag_node forwarded correctly from event
   - Run `cargo test` — expect 36/36 (29 existing + 7 new).

4. **Add private `ancestors_with_depth` helper** to `CausalOverlay`:
   - Signature: `fn ancestors_with_depth(&self, start_idx: usize) -> Vec<(usize, usize)>`
   - Apply the same bounds-checking guard as `transitive_ancestors`: `let Some(start_entity) = self.entities.get(start_idx) else { return Vec::new(); }`.
   - BFS identical to `transitive_ancestors` but uses `HashMap<usize, usize>` (event_idx → depth) for both visited-set deduplication and depth tracking (replaces the `HashSet`). Start node's direct parents are depth 1.
   - Returns `(event_idx, bfs_depth)` pairs. Start node excluded.

5. **Implement R18 `implicate_causal_nodes` method** on `CausalOverlay`:
   - Signature: `pub fn implicate_causal_nodes(&self, log: &LayeredEventLog, comparison: &PredictionComparison) -> Vec<ImplicatedNode>`
   - Call `ancestors_with_depth(comparison.comparison_event_idx)`.
   - For each ancestor: look up `self.entities[idx].dag_node` (skip ancestors without a dag_node), look up `log.events[idx].layer`. Group by dag_node into `BTreeMap<String, ImplicatedNode>`, keeping min depth per node. When multiple events share the same dag_node but have different layers, use the layer of the min-depth event.
   - Sort result by `(layer_rank, causal_distance)` where Theory=0, Methodology=1, Implementation=2. Ties within the same (layer_rank, causal_distance) naturally sort alphabetically by dag_node due to BTreeMap iteration order.

6. **Write 8 R18 tests** in `tests/mod.rs`:
   - `test_implicate_no_ancestors` — isolated event → empty result
   - `test_implicate_theory_layer` — Theory ancestor → layer=Theory
   - `test_implicate_implementation_layer` — Implementation ancestor → layer=Implementation
   - `test_implicate_methodology_layer` — Methodology ancestor → layer=Methodology (completes three-way coverage)
   - `test_implicate_mixed_layers` — Theory+Methodology ancestors → sorted Theory first
   - `test_implicate_depth_ordering` — verify causal_distance matches BFS depth correctly
   - `test_implicate_ancestor_without_dag_node` — ancestor without dag_node_ref → skipped
   - `test_implicate_multiple_events_same_dag_node` — grouped correctly, ancestor_event_indices.len()==2
   - Run `cargo test` — expect 44/44 (36 existing + 8 new).

7. **Quality gates**:
   - `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings
   - If clippy fails, fix and re-run until clean.

8. **Update FINDINGS.md** (use the investigation's step numbering, not this prompt's step numbering):
   - Append a new investigation log entry (at top of Investigation Log section) titled "Step 7: R17+R18 Query Implementation" with: Scope (R17+R18 implementation on CausalOverlay), Method (BFS-based queries reusing overlay infrastructure), Findings (both queries work end-to-end, String→SpecElementId parse-at-query-time pattern viable, three-way layer classification demonstrated), Implications (full Stage 2-3 surface validated with R14+R17+R18), Open Threads (GROMACS adapter for cross-framework generalization, prediction_id type mismatch for production ADR).
   - Update What We Know: R17 comparison executes end-to-end; R18 three-way classification demonstrated; full Stage 2-3 query surface validated.
   - Update What We Don't Know: prediction_id String vs SpecElementId mismatch deferred to production ADR.
   - Update Prototype Index table for `overlay.rs` to note R14+R17+R18 methods.

END GOAL:
When complete, the following must all be true:
- `cargo test` passes 44/44 tests (29 existing + 15 new) with zero failures
- `cargo clippy --all-targets --all-features -- -D warnings` produces zero warnings
- `PredictionComparison` and `ImplicatedNode` structs exist in `overlay.rs` with serde derives
- `compare_predictions` returns correct results for all 7 test scenarios
- `implicate_causal_nodes` returns ImplicatedNodes sorted by (layer_rank, causal_distance) for all 8 test scenarios
- FINDINGS.md has a new Step 7 log entry and updated Accumulated Findings sections
- The prototype demonstrates end-to-end: falsified prediction → implicated causal node with layer classification

NARROWING:
- Do NOT modify `event_kinds.rs` — the prediction_id String→SpecElementId mismatch is resolved at query time
- Do NOT modify `transitive_ancestors` — it has 4 tests; create a new private helper instead
- Do NOT add threshold filtering to `implicate_causal_nodes` — return all ancestors (matches `detect_confounders` pattern)
- Do NOT create new files — all changes go in existing `overlay.rs` and `tests/mod.rs`
- Do NOT edit previous FINDINGS.md log entries — append-only protocol
- Stay within prototype scope — no production code, no ADRs, no architecture changes
- Avoid over-engineering: no trait abstractions, no generic parameters, no feature flags
- Do NOT modify any existing test functions
- The `ancestors_with_depth` helper must be private (`fn`, not `pub fn`)
```

---

## Review Findings

### Issues Addressed
1. **Warning: EventId→position indirection chain** — Clarified the full lookup chain: `by_kind → Vec<EventId> → by_id[&event_id] → usize position → events[position]` in Step 2
2. **Warning: `ancestors_with_depth` bounds checking** — Added explicit instruction to use same `let Some(...) else { return Vec::new() }` guard as `transitive_ancestors` in Step 4
3. **Warning: Layer resolution for multi-layer dag_node grouping** — Specified "use the layer of the min-depth event" in Step 5
4. **Warning: FINDINGS.md step numbering** — Changed from "Step 7-8" (prompt numbering) to "Step 7" (investigation numbering) in Step 8
5. **Warning: HashMap replaces HashSet** — Clarified in Step 4 that HashMap serves both depth tracking and visited-set deduplication

### Remaining Suggestions
- Could add a concrete example of the prediction_id parse chain (`"42"` → `Some(SpecElementId(42))`, `"abc"` → `None`), though the Rust expression is unambiguous
- Sort stability within tied `(layer_rank, causal_distance)` depends on BTreeMap iteration order; this is noted but not enforced beyond natural alphabetical ordering
- The `test_implicate_*` prefix could note it exercises `implicate_causal_nodes` for clarity, but the mapping is evident from context

## Usage Notes

- **Best used with:** Claude Opus or Sonnet with full codebase context loaded (overlay.rs, common.rs, event_kinds.rs, lel.rs, tests/mod.rs)
- **Adjust for:** If the test count changes due to existing codebase evolution, update the expected counts in Steps 3/6 and End Goal
