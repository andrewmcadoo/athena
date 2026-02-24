# Session Handoff: VASP Parser Reality-Check Validation

> Generated: 2026-02-23 | Handoff #27 | Previous: handoff_026_2026-02-23_gromacs-parser-reality-check.md

---

## Continuation Directive

VASP reality-check validation — the third and final leg of the cross-framework parser validation sequence. OpenMM (Session 21) and GROMACS (Session 22) are now load-tested against format variants. VASP has 8 synthetic inline fixtures and ~30 existing tests but has never been tested against realistic INCAR/OSZICAR/OUTCAR format variations. Apply the same Session 21/22 methodology: source tier hierarchy (real > source-derived), dual-tier tests (parser extraction + convergence classification), minimal-fix scope, tier citations per variant, FINDINGS.md Session 23 entry. Closing this makes "cross-framework convergence classification" evidence-complete.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. Trace-semantics research track building a Hybrid LEL+DGR IR for structured simulation traces.

**Goal:** Validate `vasp_adapter.rs` parsing (INCAR, OSZICAR, OUTCAR) against realistic VASP output variants so derived convergence labels are evidence-backed across all three frameworks.

**Success criteria:**
1. File-backed VASP fixtures in `testdata/vasp/` (INCAR, OSZICAR, OUTCAR combinations)
2. Dual-tier variant tests: parser extraction + `classify_all_convergence` canonical pattern assertions
3. Key variants covered: converged relaxation, non-converged SCF/ionic, truncated output, execution error (EDDDAV/VERY BAD NEWS)
4. Source tier cited per variant (real vs source-derived)
5. FINDINGS.md Session 23 entry following append-only protocol
6. `cargo test` (138+ existing + N new) and `cargo clippy -- -D warnings` pass
7. Parser changes to `vasp_adapter.rs` ≤10 lines (if any needed)

**Tech stack:** Rust crate at `research/trace-semantics/prototypes/lel-ir-prototype/` (138/138 tests, strict clippy clean on master).

## Key Decisions & Rationale

1. **Reality-check methodology established across Sessions 21-22.**
   - **Rationale:** Synthetic-only fixtures don't prove parsers handle real output. Tier 1 (real logs) > Tier 2 (source-derived from docs/source inspection). Each variant cites its provenance tier.
   - **Evidence:** Session 21 found 2 real OpenMM parser gaps (BOM prefix, unquoted headers). Session 22 found 0 GROMACS gaps (parser handled all variants without modification).

2. **Dual-tier testing pattern is mandatory per variant.**
   - Tier 1: Parser function extraction (e.g., `parse_oszicar()` returns correct `(step, energy)` pairs)
   - Tier 2: Full adapter pipeline (`VaspAdapter.parse_trace()` → `classify_all_convergence()` → assert canonical pattern)
   - **Rationale:** Tier 1 catches parser regressions; Tier 2 catches classification pipeline regressions.

3. **Minimal-fix discipline: ≤10 lines per session.**
   - Fix only what new tests expose. No refactoring. If a fix exceeds scope, document as deferred open thread.

4. **VASP adapter has NO convergence derivation call.**
   - Unlike GROMACS (`derive_energy_convergence_summary` at parse_trace time) and OpenMM, the VASP adapter (`vasp_adapter.rs:488-618`) does NOT call `convergence::derive_energy_convergence_summary`. This means `classify_all_convergence(&log, "vasp")` may return empty results for VASP traces. Session 23 must account for this — it may be an intentional gap or an oversight. Investigate before testing.

5. **Do NOT change GROMACS EM semantics (Potential vs Total Energy).**
   - Deferred from Session 22. Requires a separate design decision session.

## Current State

### Completed
- **Session 21** (PR #15 merged): OpenMM CSV parser validated against 5 real + 4 source-derived fixtures. 2 parser gaps found and fixed. 128 tests.
- **Session 22** (PR #16, commit `304e712` merged): GROMACS md.log parser validated against 3 file-based + 7 inline fixtures (all Tier 2 — no real GROMACS logs available locally). 0 parser gaps. 138 tests. Full NVT component assertions, NPT-specific component assertions, EM limitation documented.

### In Progress
- Nothing. Session 23 (VASP reality-check) is the next piece of work.

### Blocked / Open Questions
- **VASP missing convergence derivation call** — `vasp_adapter.rs:parse_trace()` does not call `derive_energy_convergence_summary`. Must determine if this is intentional before writing classification tests. If intentional, document why. If oversight, adding the call is within minimal-fix scope.
- **Session 22 open thread: GROMACS EM semantics** — Treating `Potential` as convergence quantity for energy minimization runs requires a design decision. Not Session 23 scope.
- **Session 20 open thread #2** — "If production indexing needs differ, revisit post-pass architecture." Deferred.

## Key Code Context

**`src/vasp_adapter.rs`** — Three sub-parsers:
- `parse_incar()` — Key=value pairs with `!`/`#` comment stripping. Produces `ParameterRecord` events.
- `parse_oszicar()` — SCF iteration lines (`DAV:`/`RMM:`) produce `ConvergencePoint` events. Ionic step summary lines (`F=`) produce `EnergyRecord` events with E0 and dE components.
- `parse_outcar()` — Version/cores detection (`ResourceStatus`), `free energy TOTEN` lines (`EnergyRecord`), force blocks (`StateSnapshot`), completion marker `General timing and accounting` (`ExecutionStatus::Success`), error markers `EDDDAV`/`VERY BAD NEWS` (`ExecutionStatus::CrashDivergent`).

**Key difference from GROMACS/OpenMM:** VASP uses three separate input files (INCAR/OSZICAR/OUTCAR) combined via `--- MARKER ---` delimiters, vs single-file parsing for the other adapters.

**Existing VASP inline fixtures (8):** `VASP_INCAR_SAMPLE`, `VASP_OSZICAR_SAMPLE`, `VASP_OUTCAR_SAMPLE`, `VASP_OUTCAR_TRUNCATED`, `VASP_OUTCAR_ERROR`, `VASP_COMBINED_SAMPLE`, `VASP_OSZICAR_NO_F_SAMPLE`, `VASP_COMBINED_DIVERGENT_SAMPLE`.

**Existing VASP tests (~30):** Classification tests, parser extraction tests, combined adapter tests, overlay/confounder litmus tests. All use synthetic inline fixtures.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `src/vasp_adapter.rs` | VASP parser (INCAR/OSZICAR/OUTCAR) | Target for reality-check validation |
| `src/convergence.rs` | Shared derivation + taxonomy | Stable — check if VASP adapter should call it |
| `src/gromacs_adapter.rs` | GROMACS parser (Session 22 reference) | Stable |
| `src/adapter.rs` | OpenMM adapter (Session 21 reference) | Stable |
| `src/tests/mod.rs` | All tests (138/138) | Add VASP variant tests here |
| `testdata/openmm_state_datareporter/` | OpenMM file fixtures (Session 21) | Pattern reference |
| `testdata/gromacs_md_log/` | GROMACS file fixtures (Session 22) | Pattern reference |
| `FINDINGS.md` (trace-semantics) | Investigation log | Needs Session 23 entry |

All paths relative to `research/trace-semantics/prototypes/lel-ir-prototype/`.

## Loop State

N/A — single-session work, not a Claude→Codex loop.

## Next Steps

1. **Read `vasp_adapter.rs` fully.** Understand the three sub-parsers, the marker-based section splitting, and how events flow through `parse_trace()`. Confirm whether the missing `derive_energy_convergence_summary` call is intentional or an oversight.
2. **Search for real VASP output files locally.** Check `testdata/`, `research/`, `/tmp/` for INCAR/OSZICAR/OUTCAR files. If found, use as Tier 1 fixtures.
3. **Create `testdata/vasp/` directory** with file-based fixtures. Key variants:
   - **Converged ionic relaxation** — Multi-ionic-step OSZICAR with F= lines showing convergence + matching OUTCAR with `General timing and accounting` marker
   - **Non-converged SCF** — OSZICAR with DAV/RMM lines but no F= summary (SCF didn't converge within NELM)
   - **Truncated OUTCAR** — No completion marker, triggers `ExecutionOutcome::Timeout` path
   - **Execution error** — OUTCAR with `VERY BAD NEWS` or `EDDDAV` marker
4. **Add dual-tier variant tests** following Session 22 pattern:
   - Helper functions: `parse_vasp_oszicar_energy_pairs()`, `assert_vasp_variant()`, `assert_vasp_parses_energy_count()`
   - ~8-10 test functions named `test_vasp_variant_*`
5. **Apply minimal parser fixes** if tests expose gaps (≤10 lines).
6. **Update FINDINGS.md** — Session 23 entry with per-variant pass/fail and tier citations.
7. **Gate after every change:** `cargo test && cargo clippy -- -D warnings`.
8. **Commit, PR, verify CI, merge.**

## Session Artifacts

- **Prompt #031:** `.claude/prompts/prompt_031_2026-02-23_session-22-gromacs-parser-reality-check.md` (Session 22 RISEN prompt — use as template for Session 23 prompt)
- **Handoff #26:** `.claude/handoffs/handoff_026_2026-02-23_gromacs-parser-reality-check.md` (Session 22 handoff)
- **PRs merged this session:** #16 (Session 22 — GROMACS variants, commits `3c8fed1` + `ba78539`, merge commit `304e712`)
- **Current master after merge:** `304e712`

## Documentation Updated

No documentation updates — all project docs were current.
