# Prompt: GROMACS md.log Parser Reality-Check

> Generated: 2026-02-23 | Prompt #031 | Framework: RISEN

---

## Session Goal

Validate the GROMACS `md.log` parser against realistic format variations by sourcing real GROMACS log samples where available locally (Tier 1), falling back to docs/source-derived fixtures for variants that can't be sourced from real output (Tier 2), citing the source tier per variant. Add ~10 dual-tier tests (parser extraction + end-to-end classification), apply minimal parser fixes for any gaps found (e.g., tab normalization), and update FINDINGS.md — following the same testing methodology used in Session 21 for OpenMM (source tiers, dual-tier assertions, minimal fixes).

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** The task is a complex multi-step engineering process with clear sequential dependencies, gating between steps, and explicit scoping constraints. RISEN's Steps + End Goal + Narrowing sections map directly to this structure. Session 21 used TIDD-EC, which emphasizes dos/don'ts over sequential flow; RISEN is a better fit here because the step ordering and gating are central to the methodology.
- **Alternatives considered:** TIDD-EC (used in Session 21 — strong for boundary constraints but weaker at capturing sequential methodology with inter-step gates), Chain of Thought (overkill — reasoning steps already well-defined in the plan)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal and steps are unambiguous; each step has concrete deliverables |
| Specificity | 9/10 | Test names, fixture names, file paths all specified |
| Context | 8/10 | Dedicated CONTEXT section covers prior session state, crate layout, CI gate; relies on implementer reading existing code for full adapter internals |
| Completeness | 9/10 | Covers what/why/how, output format (test names, fixture paths), verification criteria |
| Structure | 9/10 | RISEN sections map cleanly to the task; steps are sequentially ordered with gates |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a Rust systems developer with molecular dynamics domain expertise, specifically GROMACS md.log output format internals. You are working in the LEL IR prototype crate at:
  research/trace-semantics/prototypes/lel-ir-prototype/

You have deep familiarity with:
- GROMACS 2023.x md.log energy block format (header/value row pairs under "Energies (kJ/mol)")
- The existing `gromacs_adapter.rs` parser (parse_log, tokenize_energy_headers, parse_energy_row)
- The Session 21 OpenMM reality-check testing methodology (source tier hierarchy, dual-tier assertions, minimal-fix scope)
- The project's convergence classification pipeline (parse_trace -> classify_all_convergence -> ConvergencePattern)

CONTEXT:
This is Session 22, continuing from Session 21 (the most recent entry in the Investigation Log in research/trace-semantics/FINDINGS.md). Session 21 closed the OpenMM CSV parser reality-check thread; GROMACS is next in the framework validation sequence. The VASP adapter will follow in a later session.

Key codebase facts:
- Test file: `src/tests/mod.rs` (~127KB, all tests in one module, currently 128 passing tests)
- Parser: `src/gromacs_adapter.rs` — `parse_log()` dispatches to `tokenize_energy_headers()` and `parse_energy_row()` for energy block extraction
- CI gate: `.github/workflows/contract-gate.yml` runs acceptance + monitoring on every PR
- Existing GROMACS fixtures: 11 inline constants (GROMACS_LOG_SAMPLE, GROMACS_LOG_STABLE_SERIES, etc.) — all synthetic, never tested against real GROMACS output variations
- File-based fixture pattern: `testdata/openmm_state_datareporter/` contains 5 files loaded via `include_str!` — follow the same pattern for GROMACS

INSTRUCTIONS:
Follow the Session 21 reality-check testing methodology:

1. **Source tier hierarchy.** Prefer real GROMACS md.log output (Tier 1: "real") over documentation/source-derived fixtures (Tier 2: "source-derived"). If real logs exist locally or can be generated from an installed GROMACS, use them. For format variants that cannot be sourced from real output (e.g., tab-separated headers, double-precision builds), construct fixtures from GROMACS source code inspection and cite the tier. If a real log is found but has different components or step numbering than the pre-designed fixtures, use the real log as-is and adjust the test expectations to match — real data takes priority over designed fixtures.

2. **Dual-tier testing.** Every variant test asserts TWO levels:
   - Tier 1 (parser extraction): `parse_log()` returns the expected `(step, total_energy)` pairs (1e-6 tolerance for floats)
   - Tier 2 (end-to-end classification): `GromacsAdapter.parse_trace()` -> `classify_all_convergence()` -> expected `ConvergencePattern`

3. **Minimal-fix scope.** Parser changes in `gromacs_adapter.rs` must be <=10 lines total. Fix only what the new tests expose. Do not refactor working code.

4. **Cite source tier per variant.** Each test function's doc-comment or the FINDINGS.md entry must state whether the fixture is Tier 1 (real) or Tier 2 (source-derived) and why.

5. **Gate after every change.** Run `cargo test && cargo clippy -- -D warnings` after each modification. Do not proceed if either fails.

STEPS:
1. **Verify baseline.** Run `cargo test` (expect 128 passing tests) and `cargo clippy -- -D warnings` (expect zero warnings) on the current master state. Record the exact test count.

2. **Search for real GROMACS logs.** Check the local filesystem for any existing `.log` files from GROMACS runs. Check `testdata/`, `research/`, `/tmp/`, and common locations. If found, use them as Tier 1 fixtures. If none exist, note this and proceed with Tier 2 only.

3. **Create `testdata/gromacs_md_log/` directory** with three file-based fixtures. Load each file in tests via `include_str!` following the `testdata/openmm_state_datareporter/` pattern:
   - `gromacs2023_nvt_md.log` — Standard NVT MD run. Full component set (Bond, Angle, Proper Dih., LJ-14, Coulomb-14, LJ (SR), Coulomb (SR), Coul. recip., Potential, Kinetic En., Total Energy, Pressure (bar)). Multiple energy blocks with converging Total Energy. Includes GROMACS banner line, `Step Time` headers, `Finished mdrun` completion marker.
   - `gromacs2023_npt_equilibration.log` — NPT equilibration. Adds NPT-specific components (Volume, Density, Pres-XX, Pres-YY, Pres-ZZ, Box-XX, Box-YY, Box-ZZ). Multiple energy blocks. Tests wider multi-row blocks with more components per block. **Note:** Verify actual NPT header names from GROMACS 2023.x source or documentation before constructing this fixture. If any NPT-specific headers contain spaces (e.g., "Pres. DC", "Vir-XX"), add them to the `known` normalization table in `tokenize_energy_headers()`.
   - `gromacs2023_energy_minimization.log` — Energy minimization (integrator = steep). Uses Potential without Total Energy. No Kinetic En. or Temperature. Multiple EM iteration blocks. Tests the known EM gap where parser emits NumericalStatus warning when Total Energy is absent.

4. **Add inline fixture constants** to `src/tests/mod.rs`:
   - `GROMACS_LOG_COMPACT_BLOCK` — 2-component blocks (Potential + Total Energy), 4 steps. Tests minimal block parsing.
   - `GROMACS_LOG_WIDE_BLOCK` — 6-component header rows (12 components total across 2 rows), 1 step. Tests column alignment with many components.
   - `GROMACS_LOG_SCIENTIFIC_NOTATION` — Values like `-1.23456e+05`, 4 steps. Tests f64::parse() on scientific format.
   - `GROMACS_LOG_TRUNCATED_MID_BLOCK` — File ends after header line, before value line. Tests incomplete block recovery.
   - `GROMACS_LOG_TAB_WHITESPACE` — Tab characters between column headers instead of spaces. **Highest risk** — in the current `split_columns()` inner function, `char::is_whitespace()` returns true for `\t`, incrementing `whitespace_run` to 1 (intra-word space), not 2 (column delimiter). A single tab between headers will be treated as part of the same token.
   - `GROMACS_LOG_DOUBLE_PRECISION` — 9+ decimal places from `-DGMX_DOUBLE=on` builds. Tests extended precision numeric parsing.
   - `GROMACS_LOG_EM_NO_TOTAL_ENERGY` — Energy blocks with Potential but no Total Energy, matching EM fixture. Documents EM limitation explicitly.

5. **Add helper functions** to `src/tests/mod.rs`:
   - `parse_gromacs_log_energy_pairs(log_content: &str) -> Vec<(u64, f64)>` — Extracts (step, total_energy) pairs from `parse_log()` output by filtering for EnergyRecord events.
   - `assert_gromacs_log_variant(log: &str, expected_pairs: &[(u64, f64)], expected_pattern: ConvergencePattern)` — Dual-tier assertion: Tier 1 checks parser extraction matches expected pairs (1e-6 tolerance), Tier 2 checks GromacsAdapter.parse_trace() -> classify_all_convergence() returns expected pattern.
   - `assert_gromacs_log_parses_energy_count(log: &str, count: usize)` — For variants where we test event counts without convergence classification (EM, wide block, truncated).

6. **Add ~10 test functions:**
   - `test_gromacs_log_variant_standard_nvt_md` — File: gromacs2023_nvt_md.log -> parser extracts energy pairs matching fixture step count, Converged pattern
   - `test_gromacs_log_variant_npt_equilibration` — File: gromacs2023_npt_equilibration.log -> parser extracts energy pairs, verifies NPT components parsed correctly
   - `test_gromacs_log_variant_energy_minimization` — File: gromacs2023_energy_minimization.log -> 0 EnergyRecords (no Total Energy), NumericalStatus warnings present, documents EM limitation
   - `test_gromacs_log_variant_compact_block` — Inline: GROMACS_LOG_COMPACT_BLOCK -> energy pairs extracted, Converged pattern
   - `test_gromacs_log_variant_wide_block` — Inline: GROMACS_LOG_WIDE_BLOCK -> energy record extracted with all components present
   - `test_gromacs_log_variant_scientific_notation` — Inline: GROMACS_LOG_SCIENTIFIC_NOTATION -> energy pairs extracted, Converged pattern
   - `test_gromacs_log_variant_truncated_mid_block` — Inline: GROMACS_LOG_TRUNCATED_MID_BLOCK -> first complete block extracted, InsufficientData classification
   - `test_gromacs_log_variant_tab_whitespace` — Inline: GROMACS_LOG_TAB_WHITESPACE -> headers parsed correctly despite tabs
   - `test_gromacs_log_variant_double_precision` — Inline: GROMACS_LOG_DOUBLE_PRECISION -> energy pairs extracted, Converged pattern
   - `test_gromacs_log_variant_em_no_total_energy` — Inline: GROMACS_LOG_EM_NO_TOTAL_ENERGY -> 0 EnergyRecords, NumericalStatus warnings present

7. **Apply minimal parser fixes** only if tests from step 6 fail:
   - **Anticipated: Tab normalization (~1 line).** Add `let normalized = header_line.replace('\t', "  ");` as the first line of `tokenize_energy_headers()`, before the `known` replacement loop, so that tabs are normalized to double-spaces before `split_columns()` is called. This ensures tabs act as column delimiters (2+ whitespace) instead of intra-word space.
   - **Possible: NPT header additions (~2 lines).** If NPT multi-word headers cause column mismatches, add entries to the `known` normalization table in `tokenize_energy_headers()`. (Unlikely — Pres-XX, Box-XX are hyphenated single tokens, but verify from GROMACS source.)
   - **Deferred (NOT session scope): EM Total Energy semantics.** Treating Potential as convergence quantity for EM runs changes semantic interpretation and requires a design decision. Document as open thread only.

8. **Update FINDINGS.md** (research/trace-semantics/FINDINGS.md):
   - Append Session 22 investigation log entry (reverse chronological, at top of Investigation Log section)
   - Structure: Date (2026-02-23), Scope, Method, Findings (per-variant pass/fail with test names and fixture paths, source tier cited per variant), Implications, Open Threads
   - Update Accumulated Findings: move "GROMACS parser handles real format variations" to What We Know (if all pass), add EM limitation to What We Suspect / What We Don't Know

9. **Final gate and commit.** Run `cargo test && cargo clippy -- -D warnings`. Verify test count is 128 + N new tests. Single commit message: `Session 22: validate GROMACS md.log parser against format variants`

END GOAL:
When complete, the following must all be true:
- `cargo test` passes with 128 + ~10 new tests, zero failures
- `cargo clippy -- -D warnings` returns zero warnings
- Three file-based fixtures exist in `testdata/gromacs_md_log/` and load via `include_str!`
- Seven inline fixture constants exist in `src/tests/mod.rs`
- Each variant test cites its source tier (real vs source-derived) in either the test doc-comment or FINDINGS.md
- `classify_all_convergence` returns the expected canonical pattern for each variant
- Parser changes to `gromacs_adapter.rs` total <=10 lines
- FINDINGS.md has a Session 22 entry following the append-only log protocol
- No changes to `convergence.rs`, `adapter.rs`, or `lib.rs`

NARROWING:
- Do NOT modify `src/convergence.rs`, `src/adapter.rs`, or `src/lib.rs`
- Do NOT change EM semantics (treating Potential as convergence quantity) — document as open thread only
- Do NOT refactor existing parser code that already works — fixes are additive only
- Do NOT write production code — all artifacts are research prototypes
- Avoid test fixtures that require external GROMACS installation to generate
- Stay within the crate root: research/trace-semantics/prototypes/lel-ir-prototype/
- Parser fixes <=10 lines total
- All existing 128 tests must continue to pass
- Follow FINDINGS.md append-only protocol (new entries at top, do not edit previous entries)
```

---

## Review Findings

### Issues Addressed

**Critical (2 resolved):**
1. **Incorrect line reference** — Removed specific line number for `tokenize_energy_headers()`, now references function name and position relative to the `known` replacement loop
2. **Framework mismatch claim** — Clarified that Session 22 follows the same *testing methodology* (source tiers, dual-tier assertions, minimal fixes) while using RISEN framework instead of Session 21's TIDD-EC, with rationale for the framework change

**Warnings (6 resolved):**
1. **Prompt number collision** — Changed from #029 to #031
2. **Session number context** — Added explicit note: "This is Session 22, continuing from Session 21"
3. **Tab fix placement** — Made explicit: "as the first line of `tokenize_energy_headers()`, before the `known` replacement loop"
4. **NPT header verification** — Added note: "Verify actual NPT header names from GROMACS 2023.x source or documentation before constructing this fixture"
5. **Missing CONTEXT section** — Added dedicated CONTEXT block with prior session state, test file size, CI gate path, fixture pattern reference, and sequencing rationale
6. **Brittle test expectations** — Softened from hardcoded counts ("5 energy pairs, Converged") to conditions ("parser extracts energy pairs matching fixture step count, Converged pattern")

### Remaining Suggestions

1. **Could use a concrete GROMACS energy block example** — A 5-line example showing header/value row pairs with the multi-word column alignment would ground the implementer's understanding of `split_columns()`. Not blocking because the existing `GROMACS_LOG_SAMPLE` constant in the test file serves this purpose.
2. **Duplicate constraint** — "No changes to convergence.rs/adapter.rs/lib.rs" appears in both END GOAL and NARROWING. Kept for emphasis since it's a hard constraint.
3. **`include_str!` loading instruction** — Now explicitly added to Step 3 ("Load each file in tests via `include_str!`").

## Usage Notes

- **Best used with:** Claude Opus or Sonnet in Claude Code with access to the athena repository
- **Adjust for:** If real GROMACS logs are found locally, use them as-is and adjust test expectations to match the real data — real data takes priority over pre-designed fixtures
