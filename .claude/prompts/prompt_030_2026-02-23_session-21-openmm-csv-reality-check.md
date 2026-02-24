# Prompt: Session 21 — OpenMM CSV Parser Reality Check

> Generated: 2026-02-23 | Framework: TIDD-EC

---

## Session Goal

Validate the Session 20 OpenMM CSV parser against real StateDataReporter output variants (column orders, optional columns, unit labels) to confirm or fix canonical convergence labels before downstream causal reasoning depends on them — scoped as a reality-check session, not a redesign.

## Framework Selection

- **Chosen:** TIDD-EC (Task, Instructions, Dos, Don'ts, Examples, Context)
- **Rationale:** High-precision validation session with explicit scope boundaries (reality-check, not redesign). The critical content is what to do and what NOT to do, mapping directly to TIDD-EC's Do/Don't structure. Clear pass/fail criteria per fixture.
- **Alternatives considered:** RISEN (good for multi-step but overkill — steps are simple and sequential here; the constraints are what matter), RTF (too lightweight for the narrowing constraints)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Binary pass/fail per fixture, explicit scope boundary |
| Specificity | 9/10 | Named files, open thread reference, CSV variant dimensions, fix-size boundary |
| Context | 9/10 | Session 20 state, open thread, parser function names, test file location |
| Completeness | 9/10 | All 4 user steps covered, examples illustrate expected/failing/anti-pattern |
| Structure | 9/10 | TIDD-EC maps naturally to the user's constraints |
| **Overall** | **9/10** | |

---

## Structured Prompt

TASK TYPE:
Parser validation against real-world data — reality-check session (not redesign).

INSTRUCTIONS:

1. Land the current branch/PR (`athena-3s3-session20`) through required CI checks. The CI gate config is at `.github/workflows/contract-gate.yml`. If the `contract-verification` check fails, diagnose and fix the minimum needed to pass — do not refactor Session 20 code. If the PR doesn't exist yet, create it targeting `master`.

2. Collect real OpenMM `StateDataReporter` CSV samples representing the variant dimensions that matter:
   - **Column order**: "Step" and "Potential Energy" in non-default positions (e.g., columns 3 and 5 instead of 1 and 3)
   - **Optional columns**: Reporters configured with extra columns (`Volume`, `Density`, `Speed`) or minimal columns (only `Step` + `Potential Energy`)
   - **Unit variants**: `kJ/mol` vs `kcal/mol` in the "Potential Energy" header
   - **Quoting variants**: With/without `#"..."` quoting on header fields
   - **Edge cases**: Empty trailing columns, Windows-style `\r\n` line endings, BOM prefix

   Sourcing priority (use highest available tier per variant):
   1. **Preferred**: Extract from real simulation output (cite file path + OpenMM version)
   2. **Acceptable**: Construct from official OpenMM `StateDataReporter` docstring (cite doc URL + parameter combination that produces this format)
   3. **Last resort**: Synthesize from OpenMM source code inspection (cite source file + behavior verification)

3. Add a small corpus of those fixtures as test constants in `src/tests/mod.rs` (path: `research/trace-semantics/prototypes/lel-ir-prototype/src/tests/mod.rs`). For each fixture:
   - Parse through `parse_openmm_energy_series` and assert correct `(step, energy)` extraction
   - Run through the full adapter → `classify_all_convergence` pipeline and assert canonical label matches expected `ConvergencePattern` enum variant
   - Name tests descriptively: `test_openmm_csv_variant_{variant_name}` (e.g., `test_openmm_csv_variant_reordered_columns`, `test_openmm_csv_variant_kcal_units`)

4. Record results in `FINDINGS.md` (`research/trace-semantics/FINDINGS.md`):
   - Append investigation log entry (Session 21, top of log, reverse chronological)
   - For each variant: state whether parsing succeeded or failed, what broke if anything, and what minimal fix was applied
   - Determine whether Session 20 open thread #1 (FINDINGS.md line 134: "Validate OpenMM CSV behavior against additional real-world reporter variants") is closed or narrowed with remaining gaps
   - Update Accumulated Findings accordingly

DO:
- Run `cargo test` and `cargo clippy -- -D warnings` after every code change (gates are non-negotiable)
- Keep all 119 existing tests passing — new tests are additive only
- Preserve the `parse_openmm_energy_series` dual-mode architecture (CSV detection + whitespace fallback)
- Use real or faithfully-representative CSV fixtures, not invented formats
- If a variant breaks parsing, fix the parser minimally — smallest diff that makes the test pass
- Cite OpenMM StateDataReporter documentation for each variant you claim exists
- Mark Session 20 open thread #1 status explicitly (closed/narrowed) with evidence

DON'T:
- Don't redesign `convergence.rs`, `classify_convergence`, or the taxonomy types — Session 20 architecture is settled
- Don't modify existing test assertions or rename existing tests
- Don't add new taxonomy patterns, confidence tiers, or enum variants
- Don't change the `ConvergencePoint` struct or any existing event types
- Don't refactor code that isn't broken by a real variant — fix only what fails
- Don't add unit-conversion logic unless a real variant proves it's needed (document the finding either way)
- Don't skip the FINDINGS.md update — this is a research session, not just a coding session

Parser fix scope boundary: If a variant requires a fix, the fix must be:
- Max ~10 lines of new/changed code per variant
- No change to `parse_openmm_energy_series` function signature or return type
- No conditional branches deeper than the existing CSV/whitespace fork
- If a variant needs more than this: document as a narrowed open thread in FINDINGS.md ("Variant X validated as real but deferred — requires parser refactor beyond reality-check scope"), do NOT fix in this session

EXAMPLES:

Example 1 — Fixture that SHOULD pass (reordered columns):
```
#"Time (ps)","Potential Energy (kJ/mol)","Step","Temperature (K)"
1.0,-5000.0,100,300.0
2.0,-5001.0,200,300.1
3.0,-5001.5,300,300.0
4.0,-5001.8,400,299.9
5.0,-5001.9,500,300.0
```
Expected: `parse_openmm_energy_series` finds "Step" at col 2, "Potential Energy" at col 1, extracts `[(100, -5000.0), (200, -5001.0), ...]`.

Example 2 — Fixture that MAY break (no quoting on header):
```
Step,Time (ps),Potential Energy (kJ/mol),Temperature (K)
100,1.0,-5000.0,300.0
200,2.0,-5001.0,300.1
```
Expected: If current parser requires `#"` prefix, this will fail CSV detection and fall through to whitespace mode (which also fails). This is a real finding — document and fix if confirmed.

Example 3 — What NOT to do:
If `kcal/mol` units are found in real OpenMM output, do NOT add unit-conversion math to the parser. Instead: document the finding, add a test that asserts parsing extracts the raw numeric value (unit-agnostic), and note in FINDINGS.md that unit-awareness is a deferred concern.

CONTEXT:
- **Prior session**: Session 20 (branch `athena-3s3-session20`, PR pending) built `convergence.rs` with canonical taxonomy, extended `parse_openmm_energy_series` for CSV, and added 19 new tests (119/119 total).
- **Open thread**: FINDINGS.md line 134 — "Validate OpenMM CSV behavior against additional real-world reporter variants (column order/optional fields) beyond current synthetic fixtures."
- **Parser location**: `parse_openmm_energy_series` in `src/adapter.rs` (line ~41), CSV helper `parse_openmm_csv_energy_series` (line ~81).
- **Test file**: `src/tests/mod.rs` (127KB, all tests in one module).
- **CI gate**: `.github/workflows/contract-gate.yml` — `contract-verification` required check blocks direct push to master.
- **Why OpenMM first**: VASP convergence is native (no parser risk). GROMACS derivation works but validation against real logs is a separate future session. OpenMM has the identified open thread and is the last framework without real-data CSV validation.
- **Why this matters**: If real CSV variants break parsing, canonical labels become wrong, and downstream causal reasoning gets bad evidence.

---

## Review Findings

### Issues Addressed
1. **CRITICAL-1 (CI diagnosis path):** Added `.github/workflows/contract-gate.yml` pointer in Step 1.
2. **CRITICAL-2 (real vs synthetic boundary):** Added three-tier sourcing priority in Step 2 (real output > docs-constructed > source-code-inspected).
3. **CRITICAL-3 (fix scope boundary):** Added explicit parser fix size rule (~10 lines max, no signature changes, no deeper nesting; else defer and document).

### Remaining Suggestions
- Could add more fixture examples for optional columns, unit variants, and edge cases (two examples set the pattern; implementer can extrapolate)
- Could add explicit test-naming table mapping variants to test names (convention is demonstrated in Step 3)
- Could add OpenMM documentation URL for StateDataReporter reference (implementer can locate via standard search)
- Could add open-thread closure criteria table (closed = all 5 dimensions validated with <=3 fixes; narrowed = 3-4 validated; open = needs redesign)

## Usage Notes

- **Best used with:** Claude Code in a session with the `athena-3s3-session20` branch checked out (or after it's merged to master)
- **Adjust for:** If real CSV variants are hard to source, tier-2 (docs-constructed) fixtures are acceptable — the point is variant coverage, not provenance perfection
