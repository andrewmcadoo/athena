# Session Handoff: GROMACS Parser Reality Check

> Generated: 2026-02-23 | Handoff #26 | Previous: handoff_025_2026-02-23_cross-framework-convergence-standardizat.md

---

## Continuation Directive

Real-world GROMACS parser validation. We just crash-tested the OpenMM parser on messy real files (Session 21); now do the same for GROMACS so convergence labels are trustworthy across all three frameworks. Collect real `md.log` / energy-block samples across GROMACS versions and run modes, add fixture-driven tests, apply minimal parser fixes for anything that breaks, and update FINDINGS.md with per-variant results.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. Trace-semantics research track building a Hybrid LEL+DGR IR for structured simulation traces.

**Goal:** Validate `gromacs_adapter.rs` parsing against real GROMACS output variants so derived convergence labels are evidence-backed, not synthetic-fixture-only. Same scope discipline as Session 21 (OpenMM reality check).

**Success criteria:**
1. Real GROMACS `md.log` / energy-block fixtures added to test corpus
2. Parser extraction + `classify_all_convergence` canonical pattern assertions pass
3. Any parser gaps found are fixed minimally (~10 lines max per fix, no signature changes)
4. FINDINGS.md Session 22 log entry documents per-variant pass/fail
5. GROMACS reality-check thread explicitly closed or narrowed with evidence
6. `cargo test` (128+ tests) and `cargo clippy -- -D warnings` pass after every change

**Tech stack:** Rust crate at `research/trace-semantics/prototypes/lel-ir-prototype/` (128/128 tests, strict clippy clean on master).

## Key Decisions & Rationale

1. **Adapter-inline derivation with shared utility (WDK#44, resolved).**
   - **Rationale:** Preserves natural provenance at parse time, avoids Stage 1→2 post-pass architecture cost. Shared `convergence::derive_energy_convergence_summary` eliminates duplication.
   - **Alternatives rejected:** Stage 1→2 post-pass (requires new component, VASP doesn't fit, provenance becomes indirect).

2. **Canonical taxonomy is a read-only projection (WDK#42, resolved).**
   - **Rationale:** `CanonicalConvergence` classifies existing `TraceEvent` data without modifying `ConvergencePoint` struct. 5 patterns: Converged, Oscillating, Stalled, Divergent, InsufficientData. 3 confidence tiers: Direct, Derived, Absent.
   - **Alternatives rejected:** Modifying ConvergencePoint struct (breaks existing adapters, conflates storage with classification).

3. **Reality-check sessions use minimal-fix discipline.**
   - **Rationale:** Established in Session 21 — max ~10 lines per variant fix, no signature changes, no deeper nesting than existing CSV/whitespace fork. If a fix exceeds this, document as deferred open thread.
   - **Evidence:** Session 21 found 2 real OpenMM parser gaps (BOM prefix, unquoted headers), fixed both within scope.

4. **Provenance invariants are non-negotiable.**
   - `causal_refs` linking to source energy events + ExecutionStatus/NumericalStatus
   - `Completeness::Derived { from_elements }` for GROMACS/OpenMM (never `FullyObserved`)
   - Minimum-data guard: window >= 4 energy events, `None` below threshold

5. **FINDINGS.md is append-only, authoritative over handoffs.** Investigation log entries are immutable. Only Accumulated Findings sections are updated as evidence changes.

## Current State

### Completed
- **Session 20** (commits `262dd93`, `297f4dc`, PR #15 merged): Extracted shared derivation into `convergence.rs`, added canonical taxonomy (5 patterns, 3 confidence tiers), extended OpenMM CSV parser, added 19 new tests (100→119), cross-framework equivalence tests A-F, WDK#42 and WDK#44 resolved.
- **Session 21** (commit `a2dca80`, PR #15 merged): Validated OpenMM CSV parser against real OpenMM 8.4 StateDataReporter variants. Found and fixed 2 parser gaps (BOM prefix, unquoted headers). Added 9 new tests (119→128). 5 real CSV fixtures in `testdata/openmm_state_datareporter/`. Session 20 open thread #1 closed.
- **Sessions 1-19**: IR surveys, 3 adapters (VASP/GROMACS/OpenMM), CausalOverlay, R14/R17/R18 queries, convergence derivation, governance chain complete.

### In Progress
- Nothing. Session 22 (GROMACS reality check) is the next piece of work.

### Blocked / Open Questions
- **GROMACS reality-check thread (new):** No existing open thread in FINDINGS.md — Session 22 should create one or document inline. The OpenMM equivalent was FINDINGS.md:134 (now closed).
- **Session 20 open thread #2 (FINDINGS.md:135):** "If production indexing needs differ, revisit post-pass architecture" — deferred, not Session 22 scope.
- **`athena-fom`:** Flagged stale. Do not close — deferred to cleanup session.

## Key Code Context

**`src/gromacs_adapter.rs`** — The parser under test. Key areas:
- `parse_gromacs_log_section` (line ~195): Parses energy blocks from GROMACS `md.log` format. Extracts step number, energy components.
- `gromacs_energy_total` (line ~530): Extracts total energy from `EnergyRecord` events.
- Convergence derivation now calls `convergence::derive_energy_convergence_summary(events, "simulation.log")` (line ~597).

**`src/convergence.rs`** — Shared derivation + taxonomy. Do NOT modify unless a GROMACS variant exposes a real gap in the shared logic.
- `derive_energy_convergence_summary`: 4-point window, rel_delta threshold 1e-4, sign-change oscillation detection.
- `classify_convergence`: Divergent override → metric-name mapping → confidence from Completeness.

**`src/adapter.rs`** — OpenMM adapter. Reference for how Session 21 reality-check was structured (fixtures, tests, parser fixes). Use as a pattern.

**`testdata/openmm_state_datareporter/`** — Session 21 fixture corpus. Create analogous `testdata/gromacs_md_log/` directory.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `src/gromacs_adapter.rs` | GROMACS parser + convergence derivation | Target for reality-check validation |
| `src/convergence.rs` | Shared derivation + taxonomy types + mapping | Stable — don't modify unless broken |
| `src/adapter.rs` | OpenMM adapter (Session 21 reference pattern) | Stable |
| `src/tests/mod.rs` | All tests (128/128) | Add GROMACS variant tests here |
| `src/lib.rs` | Module declarations | Stable |
| `testdata/openmm_state_datareporter/` | Session 21 OpenMM fixtures | Reference for fixture organization |
| `FINDINGS.md` (trace-semantics) | Investigation log + accumulated findings | Needs Session 22 entry |
| `CLAUDE.md` | Project governance | Updated this handoff (removed stale test count) |

## Loop State

N/A — single-session work, not a Claude→Codex loop.

## Next Steps

1. **Read** `gromacs_adapter.rs` fully — understand the MDP + LOG parsing pipeline, energy block format, and how events flow to `derive_energy_convergence_summary`.
2. **Collect real GROMACS fixtures.** Variant dimensions to cover:
   - **Version differences:** GROMACS 2021.x vs 2023.x vs 2024.x `md.log` energy block format
   - **Run modes:** Normal MD, energy minimization, NVT/NPT equilibration
   - **Truncated output:** Simulation killed mid-step (incomplete energy block)
   - **NaN/instability:** Blown-up simulation with non-finite energy values
   - **MDP variations:** Different integrator/thermostat combinations affecting log format
   Source priority: real output > docs-constructed > source-inspected (same as Session 21).
3. **Create `testdata/gromacs_md_log/` directory** for fixture files (parallel to OpenMM fixtures).
4. **Add fixture-driven tests** in `src/tests/mod.rs`:
   - Parser extraction: `test_gromacs_log_variant_{name}` asserting correct event extraction
   - Classification: full adapter → `classify_all_convergence` → assert canonical pattern
5. **If a variant breaks parsing**, fix minimally (~10 lines max, no signature changes). If fix exceeds scope, document as deferred.
6. **Update FINDINGS.md**: Session 22 log entry with per-variant results, close or narrow GROMACS reality-check thread.
7. **Gates after every change:** `cargo test`, `cargo clippy -- -D warnings`.
8. **Commit, PR, verify CI, merge, close bead, `bd sync`.**

## Session Artifacts

- **Prompt #029:** `.claude/prompts/prompt_029_2026-02-23_session-20-convergence-taxonomy.md` (Session 20 RISEN prompt)
- **Prompt #030:** `.claude/prompts/prompt_030_2026-02-23_session-21-openmm-csv-reality-check.md` (Session 21 TIDD-EC prompt)
- **Handoff #25:** `.claude/handoffs/handoff_025_2026-02-23_cross-framework-convergence-standardizat.md` (Session 20 handoff)
- **PRs merged this session:** #15 (Sessions 20 + 21 combined)
- **Current master:** `9778bf8eb4826c04688b0f2c9951b16fcc83bfe2`

## Documentation Updated

| Document | Change Summary | Status |
|----------|---------------|--------|
| `CLAUDE.md` | Removed stale test count `(100/100 tests)` from directory structure | Approved and applied |
