# Session Handoff: Cross-Framework Convergence Standardization

> Generated: 2026-02-23 | Handoff #25 | Previous: handoff_024_2026-02-23_break-glass-outage-resilience.md

---

## Continuation Directive

Session 20 should turn "it works in prototype code" into "it means the same thing across VASP, GROMACS, and OpenMM." Three deliverables: (1) WDK#42 — finalize a canonical convergence-pattern taxonomy with confidence semantics and cross-framework mapping, (2) WDK#44 — decide placement of convergence-summary computation (adapter-inline vs Stage 1→2 post-pass) with a documented A/B comparison, (3) validate convergence derivation on real OpenMM reporter output (not just mock input). Exit criterion: all three frameworks produce convergence summaries using the same label set, computed at the same pipeline stage, with tests proving equivalence.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The trace-semantics research track is building a Hybrid LEL+DGR intermediate representation for structured simulation traces.

**Goal:** Standardize convergence-pattern semantics across VASP, GROMACS, and OpenMM so downstream causal reasoning and reward logic compare equivalent signals, not framework-specific strings.

**Success criteria:**
1. WDK#42 narrowed/resolved with explicit taxonomy mapping table (VASP native → canonical labels, GROMACS/OpenMM derived → same labels)
2. WDK#44 narrowed/resolved with documented placement decision and rationale
3. Real OpenMM trace validation added and passing
4. FINDINGS.md updated with evidence chain and tests
5. `athena-fom` remains only flagged (not closed) — deferred to a cleanup session

**Tech stack:** Rust prototype crate at `research/trace-semantics/prototypes/lel-ir-prototype/` (100/100 tests, strict clippy clean).

## Key Decisions & Rationale

1. **Per-adapter convergence derivation (not shared module).** Session 19 implemented GROMACS and OpenMM convergence-summary derivation inline in each adapter, matching the VASP pattern. Rationale: VASP emits ConvergencePoints during OSZICAR parsing; keeping GROMACS/OpenMM consistent avoids an architectural departure. A shared module would only be warranted if derivation logic were demonstrably identical across adapters — it wasn't.

2. **4-point minimum energy window for derivation.** Session 19 set `window >= 4` as the minimum data condition before emitting a ConvergencePoint. Rationale: fewer than 4 points cannot distinguish convergence from noise. This threshold is explicit in the code and tested (insufficient-data test case).

3. **WDK#41 resolved in adversarial-reward, not trace-semantics.** The profile-aggregation reward scalar was formalized through 7 adversarial-reward sessions (locked AggregateScore recommendation, ADR 002, CI gate, governance enforcement). Trace-semantics carried a stale WDK#41 reference that Session 19 closed with cross-track citation.

4. **Append-only FINDINGS.md rule strictly enforced.** Session 19 confirmed: only Accumulated Findings sections (What We Know/Suspect/Don't Know) may be updated. Investigation log entries are immutable historical records — even if they contain stale open threads.

5. **FINDINGS.md authoritative over handoff documents.** For any discrepancy, FINDINGS.md is source of truth. Established Session 14.

6. **Non-mutating governance sessions by default.** All audit/drill work is read-only unless explicitly approved. Governance track (Sessions 11–18) is complete; Session 18 live drill validated all policy windows with strong headroom.

7. **`athena-fom` flagged stale, not closed.** Session 19 flagged this bead as potentially stale in open threads. Evidence suggests it was completed in Sessions 12–13, but closure is deferred to a dedicated cleanup session per scope guard.

## Current State

### Completed
- **Session 19** (commit `5cfb30b`, PRs #12 + #13, merged): WDK#41 closed (bead `athena-apb` closed, trace-semantics FINDINGS resolved with cross-track citation). WDK#43 resolved (prototype scope) — GROMACS and OpenMM convergence-summary derivation implemented per-adapter with 4-point window, 8 new tests (100/100 total). WDK#42 and WDK#44 narrowed. Bead `athena-96u` closed.
- **Session 18** (commit `16ff7f1`, PR #11, merged): Sandboxed live drill of Mode B (298s override, strong headroom) and Mode C (85s containment, strong headroom). All policy windows validated. Sandboxed-drill open thread closed.
- **Session 17** (commit `8a5e781`, PR #10, merged): GOVERNANCE.md extended with Mode A/B/C outage fallbacks, operator decision tree, per-mode evidence requirements.
- **Sessions 11–16**: Governance chain: CI gate → branch protection → admin bypass closure → runbook → audit automation → escalation thresholds.
- **Sessions 1–10 (adversarial-reward)**: AggregateScore research through locked recommendation and architecture integration.
- **Trace-semantics Steps 1–14**: Complete. IR surveys, requirements (R1-R29), candidate evaluation (Hybrid LEL+DGR, 94/100), prototype crate with 3 adapters (OpenMM/GROMACS/VASP), CausalOverlay + R14/R17/R18 queries, convergence trajectory design (Option D hybrid), UncertaintySummary schema.

### In Progress
- Nothing. Session 20 is the next piece of work.

### Blocked / Open Questions
- **WDK#42** (trace-semantics FINDINGS.md line ~1857): "What minimal cross-framework `ConvergencePattern` taxonomy should be adopted, and how pattern confidence should be calibrated." Status: narrowed by Session 13 (hybrid raw+summary direction) and Session 19 (derivation rules exist). Remaining: canonical label set and cross-framework mapping.
- **WDK#44** (trace-semantics FINDINGS.md line ~1861): "Where convergence summary computation should occur and how it should attach graph/query anchors." Status: narrowed by Session 13 (recommends Stage 1→2 boundary) and Session 19 (implemented per-adapter inline). Remaining: explicit A/B comparison and final decision.
- **`athena-fom`**: Flagged stale. Do not close — deferred to cleanup session.

## Key Code Context

**`src/event_kinds.rs`** — ConvergencePoint type (the struct all three adapters must emit):
```rust
pub struct ConvergencePoint {
    pub iteration: u64,
    pub metric_name: String,
    pub metric_value: f64,
    pub converged: bool,
}
```

**`src/vasp_adapter.rs`** — Reference implementation (native ConvergencePoint emission during OSZICAR parsing). VASP emits convergence data directly from parsed SCF iteration deltas. Do not modify.

**`src/gromacs_adapter.rs`** — Session 19 added derived ConvergencePoint emission from EnergyRecord streams. Uses 4-point sliding window on potential energy.

**`src/adapter.rs`** — Session 19 added derived ConvergencePoint emission for OpenMM from EnergyRecord streams. Same 4-point window pattern. Currently uses mock input — WDK#42/Session 20 should validate on real OpenMM reporter output.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Investigation log + accumulated findings | Needs Session 20 entry |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/event_kinds.rs` | ConvergencePoint type definition | Stable — study, don't change struct |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/vasp_adapter.rs` | VASP adapter (reference convergence impl) | Stable — study, don't change |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/gromacs_adapter.rs` | GROMACS adapter + derived convergence | Modified S19, may need taxonomy update |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs` | OpenMM adapter + derived convergence | Modified S19, needs real-trace validation |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/tests/mod.rs` | Test suite (100/100) | Needs WDK#42 equivalence tests |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/lel.rs` | TraceEvent, EventLog, EventIndexes | Stable |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/overlay.rs` | CausalOverlay + R14/R17/R18 queries | Stable |
| `research/adversarial-reward/FINDINGS.md` | Adversarial-reward log (reference only) | No changes expected |
| `CLAUDE.md` | Project governance | Updated this handoff (ADR 002, lel-ir-prototype) |

## Loop State

N/A — single-session work, not a Claude→Codex loop.

## Next Steps

1. **Create bead and branch** (`session-20/wdk42-wdk44-convergence-taxonomy`).
2. **Read current artifacts**: trace-semantics FINDINGS.md (accumulated findings + Session 19 log), all three adapter source files, `event_kinds.rs`.
3. **WDK#42: Define canonical convergence taxonomy.**
   - Survey current `metric_name` strings across all three adapters.
   - Define one canonical label set (e.g., `converged`, `stalled`, `oscillating`, `divergent`, `insufficient_data`).
   - Create explicit mapping table: VASP native signals → canonical labels, GROMACS derived → same, OpenMM derived → same.
   - Define confidence semantics: what does each label mean quantitatively, and how is confidence calibrated.
   - Add cross-framework equivalence tests on synthetic cases proving the same physical scenario produces the same canonical label from all three adapters.
4. **WDK#44: Decide placement.**
   - Document the current state: VASP emits inline during parsing, GROMACS/OpenMM emit inline per Session 19.
   - Run focused A/B analysis: adapter-inline (current) vs Stage 1→2 post-pass. Compare on determinism, provenance clarity, query/index impact.
   - Make one decision. Remove ambiguity. Document rationale in FINDINGS.
5. **OpenMM production-path validation.**
   - Validate derivation rules on real OpenMM reporter output (not mock input).
   - Measure instrumentation overhead for the chosen signal source.
   - Confirm minimum-data behavior and no synthetic certainty in real traces.
6. **Log Session 20** in trace-semantics FINDINGS.md.
7. **Update Accumulated Findings** — resolve or narrow WDK#42 and WDK#44 with citations.
8. **Commit, PR, verify, merge, close bead, `bd sync`.**

## Session Artifacts

- **Prompt #026:** `.claude/prompts/prompt_026_2026-02-23_session-17-break-glass-outage-resilience.md` (Session 17 RISEN prompt)
- **Prompt #027:** `.claude/prompts/prompt_027_2026-02-23_session-18-sandboxed-live-drill.md` (Session 18 RISEN prompt)
- **Prompt #028:** `.claude/prompts/prompt_028_2026-02-23_session-19-wdk41-closure-wdk43.md` (Session 19 RISEN prompt)
- **Handoff #24:** `.claude/handoffs/handoff_024_2026-02-23_break-glass-outage-resilience.md` (Session 17 handoff)
- **PRs merged this conversation:** #10 (S17), #11 (S18), #12 + #13 (S19)
- **Current master:** `5cfb30bfcd4b51038198d3a6dc576273a0cf61c0`

## Documentation Updated

| Document | Change Summary | Status |
|----------|---------------|--------|
| `CLAUDE.md` | Added ADR 002 to Key Artifacts; added `lel-ir-prototype/` to Directory Structure | Approved and applied |
