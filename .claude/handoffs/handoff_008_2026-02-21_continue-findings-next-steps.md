# Session Handoff: Hybrid LEL+DGR Phase 2 Complete — Continue FINDINGS.md Next Steps

> Generated: 2026-02-21 | Handoff #8 | Previous: handoff_007_2026-02-21_phase2-hybrid-prototyping.md

---

## Continuation Directive

Continue the trace-semantics research by tackling the next unblocked FINDINGS.md next steps. The prototype now has a working CausalOverlay with R14 confounder detection. The natural continuation is implementing R17 (prediction-observation comparison) and/or R18 (causal implication traversal) queries against the overlay, which would complete the Stage 2-3 query surface and validate the Hybrid architecture across all three critical query patterns. Alternatively, a second mock adapter (GROMACS) would stress-test cross-framework generalization.

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** Validate the Hybrid LEL+DGR IR architecture through progressive prototyping. The IR must enable three-way causal fault classification (theory/methodology/implementation) from DSL trace logs.

**Success criteria:** An IR prototype that demonstrates correct fault classification on planted faults, exercising all three stages (implementation audit, methodology audit, theory evaluation).

**Constraints:** Per CLAUDE.md — research artifacts only (prototypes/), append-only FINDINGS.md, cite evidence for all claims. Per ADR 001 — Rust for perf-critical, Python for orchestration.

## Key Decisions & Rationale

1. **Hybrid LEL+DGR is the IR architecture (94/100)**
   - **Rationale:** LEL streaming for Stage 1, DGR graph traversal for Stages 2-3. Only candidate that passes all 9 anti-patterns.
   - **Alternatives rejected:** LEL standalone (82, weak R14/R18), DGR standalone (82, unnecessary graph overhead for Stage 1), TAL (deferred to query interface layer)

2. **1:1 entity mapping for CausalOverlay (entity index == event index)**
   - **Rationale:** Avoids separate index translation, eliminates off-by-one errors, events without `dag_node_ref` still participate in causal chains
   - **Alternatives rejected:** Selective mapping (~3x less memory but requires two-pass construction and translation layer). Acceptable memory tradeoff for prototype.

3. **Index-only overlay design (no lifetime parameters)**
   - **Rationale:** CausalOverlay stores `usize` positions, log passed to query methods as `&LayeredEventLog`. Avoids Rust lifetime complexity.
   - **Alternatives rejected:** Reference-holding overlay (would require `'a` lifetime threading through all methods)

4. **On-demand BFS for transitive ancestors (no pre-computed closure)**
   - **Rationale:** Pre-computing all transitive closures is O(n^2) worst case. BFS from a handful of seed events visits far fewer nodes in sparse causal graphs.

5. **Vec-first allocation (arena deferred)**
   - **Rationale:** Bench confirms Vec with `with_capacity(n)` achieves adequate performance. Arena benefits only interleaved allocations, not batch O(n) construction.

## Current State

### Completed
- **Steps 1-6 all complete.** Full investigation chain: DSL surveys -> requirements (R1-R29) -> candidate schemas -> LEL prototype -> open thread resolution -> CausalOverlay + R14 query
- **29/29 tests pass**, strict clippy clean, bench uses real `CausalOverlay::from_log`
- **Benchmark:** 10^6 events: 251.82ms overlay construction, 22.62ms at 10^5. O(n) confirmed.
- **FINDINGS.md updated:** Step 6 log entry, WDK #37 closed, #38 narrowed, WK #48-50 added, prototype index updated

### In Progress
- Nothing — clean state for next research step.

### Blocked / Open Questions
- **WDK #35:** `ContractTerm` may need `value: Option<Value>` for VASP Stage 3 — non-blocking
- **WDK #36:** `Value` enum may need `KnownMatrix` variant for VASP spectral data — non-blocking
- **WDK #38:** Arena allocation deferred — benchmark Vec validated, adopt arena only if profiling shows need
- **WDK #28:** R17 prediction-observation comparison method not yet formalized — blocks Stage 3 feasibility
- **No Step 7+ items in FINDINGS.md Next Steps yet** — next session should define and add them

## Key Code Context

**`src/overlay.rs:107-177`** — R14 detect_confounders (the pattern for R17/R18):
```rust
pub fn detect_confounders(
    &self,
    log: &LayeredEventLog,
    observable_var: &str,
    intervention_var: &str,
) -> Vec<ConfounderCandidate> {
    // Pattern: variable lookup via by_variable -> position resolve via by_id
    //   -> BFS transitive ancestors -> set intersection -> filter + group
}
```

**`src/overlay.rs:78-105`** — transitive_ancestors BFS (reusable for R17/R18):
```rust
pub fn transitive_ancestors(&self, start_idx: usize) -> Vec<usize> {
    // BFS over causal_parents, start node excluded, visited tracking via HashSet
}
```

**`src/lel.rs:37-46`** — ExperimentSpec (needed for R17 — PredictionRecord access):
```rust
pub struct ExperimentSpec {
    pub predictions: Vec<PredictionRecord>,      // R17 needs these
    pub interventions: Vec<InterventionRecord>,
    pub controlled_variables: Vec<ControlledVariable>,
    // ...
}
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master investigation log (50 WK, 38 WDK) | Updated this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/lel.rs` | Core LEL types, builders, indexes (with `by_id`) | Modified this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/overlay.rs` | CausalOverlay, R14 query | Created this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/lib.rs` | Module declarations (added `overlay`) | Modified this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/bench.rs` | Overlay benchmark (real `from_log` path) | Modified this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/tests/mod.rs` | 29 tests (11 original + 18 new) | Modified this session |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/common.rs` | Shared types (unchanged) | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/event_kinds.rs` | 12 EventKind variants (unchanged) | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs` | Mock OpenMM adapter (unchanged) | Reference |
| `research/trace-semantics/dsl-evaluation/candidate-ir-schemas.md` | Source of truth for Hybrid design (Section 4) | Reference |

## Loop State

N/A — single-session work. No Codex loop. Implementation done directly via Codex with Claude Code review.

## Next Steps

1. **Read FINDINGS.md** — specifically Step 6 log entry and WDK items #28, #35, #36, #38 for full context on what's unblocked
2. **Add Step 7+ items to FINDINGS.md Next Steps** — the section currently ends at Step 6. Candidate items:
   - **Step 7: R17 prediction-observation comparison query** — Match `ComparisonResult` events against `PredictionRecord` entries in spec, using overlay to trace DAG path. Requires formalizing WDK #28 (divergence metrics).
   - **Step 8: R18 causal implication traversal** — Given a falsified prediction (R17 result), walk causal graph backward to identify implicated DAG nodes. This is the core ATHENA value prop.
   - **Step 9: Second mock adapter (GROMACS)** — Stress-test IR generalization across DSL frameworks (Architectural Constraint #1).
3. **Implement chosen next step** following the same pattern: design -> implement -> test -> clippy -> bench -> FINDINGS.md log entry
4. **Note:** R17 + R18 together complete the Stage 2-3 query surface. Once all three queries (R14, R17, R18) work, the prototype validates the architecture, not just one query path.

## Session Artifacts

- Prompt: `.claude/prompts/prompt_002_2026-02-21_lel-overlay-r14.md` (RISEN prompt for this session's work)
- Plan: `.claude/plans/tranquil-mapping-robin.md` (approved implementation plan)
- Benchmark results: 10^6 events — 251.82ms overlay, 22.62ms at 10^5

## Documentation Updated

No documentation updates — all project docs were current.
