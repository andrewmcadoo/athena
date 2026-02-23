# Prompt: Acceptance Test Suite for ATHENA Aggregation Prototype

> Generated: 2026-02-22 | Framework: RISEN

---

## Session Goal

Create a structured prompt that instructs an AI to implement an acceptance test suite for the ATHENA adversarial-reward aggregation prototype, covering three contracts (margin parity, guardrail rejection, decomposition invariant) as automated regression gates.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation task with clear sequential dependencies (3 contracts + runner), a defined role (test implementer within existing prototype codebase), explicit success criteria (15 PASS, exit 0), and important boundaries (stdlib-only, single file, no modifications).
- **Alternatives considered:** TIDD-EC — strong dos/don'ts support, but the task is fundamentally process-driven rather than constraint-driven.

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal, role, and steps are unambiguous |
| Specificity | 10/10 | Exact imports, tolerances, key mappings, expected margins all specified |
| Context | 9/10 | Full file references, line numbers, architectural context provided |
| Completeness | 9/10 | Covers all 3 contracts, runner, verification, and post-test steps |
| Structure | 9/10 | Clean RISEN hierarchy with numbered steps and sub-steps |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a Python test engineer implementing automated acceptance tests for a research prototype. You have access to the existing aggregation-candidates codebase in `research/adversarial-reward/prototypes/aggregation-candidates/` and understand its module interfaces. You write stdlib-only Python (no pytest, no unittest — just functions and assertions). You are meticulous about numerical tolerances and contract enforcement.

INSTRUCTIONS:
Create a single file `acceptance_test.py` in the `aggregation-candidates/` directory. This file enforces three contracts from Session 8's manual verification (7/7 PASS, max margin delta 4.414e-07) as automated regression gates. Use only Python standard library. Structure as three test functions plus a `main()` runner. The file must be self-contained and runnable via `python acceptance_test.py`.

STEPS:

1. **Set up imports.** At the top of `acceptance_test.py`, import:
   ```python
   import json, sys
   from pathlib import Path
   from scenarios import DEFAULT_CUSTOM_SIGMOIDS, build_scenario_fixtures
   from normalization import NormalizationConfig
   from models import SigmoidParams
   from candidates import HybridConfig, aggregate_hybrid
   from evaluate import evaluate_fixture, ScenarioCellResult
   from ceiling_analysis import margin_from_cell
   ```
   Note: `math` is optional — `abs()` is a builtin and suffices for tolerance checks.

2. **Define the fixture-index-to-JSON-key mapping** as a module-level constant:
   ```python
   SCENARIO_KEYS = {
       1: "S1_noisy_tv",
       2: "S2_unanimous_weak_signal",
       3: "S3_mixed_signal",
       4: "S4_missing_data",
       5: "S5_scale_heterogeneity",
       6: "S6_calibration_decomposability",
       7: "S7_boundary_seeking",
   }
   ```

3. **Implement `test_margin_parity() -> list[tuple[str, bool, str]]`** (Contract 1 — Category A):
   a. Load `baseline_margins` from `aggregate_score_recommendation.json`. Resolve the path via `Path(__file__).resolve().parent / "aggregate_score_recommendation.json"`.
   b. Build fixtures via `build_scenario_fixtures()`.
   c. Construct locked config: `NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)` → `HybridConfig(normalization=norm_cfg)`.
   d. Build candidate fn: `lambda comps: aggregate_hybrid(comps, config=hybrid_cfg)`.
   e. For each `fixture` in the list returned by `build_scenario_fixtures()`:
      - `cell = evaluate_fixture(fixture, "Hybrid", fn)`
      - Assert `cell.passed is True`
      - `margin, _ = margin_from_cell(cell)`
      - Look up expected margin from JSON via `SCENARIO_KEYS[fixture.idx]`
      - Assert `abs(margin - expected) <= 1e-6`
      - Note: S6 (`S6_calibration_decomposability`) has baseline margin `0.000000` in JSON (rounded); actual margin is ~1e-8. The 1e-6 tolerance accommodates this.
   f. Return list of `(scenario_name, passed_bool, detail_string)` tuples.

4. **Implement `test_guardrail_rejection() -> tuple[str, bool, str]`** (Contract 2 — Category B3):
   a. Attempt `NormalizationConfig(custom_sigmoids={"test": SigmoidParams(k=2.0, x0=-0.2)})`.
      Note: The `k` value is irrelevant to this guardrail — only `x0 < 0` triggers the rejection. Any positive `k` suffices.
   b. Assert this raises `ValueError`.
   c. Assert the error message contains `"GR-S2-CUSTOM-SIGMOID-X0-NONNEG"`.
   d. Return a single `(test_name, passed_bool, detail_string)` tuple.

5. **Implement `test_decomposition_invariant() -> list[tuple[str, bool, str]]`** (Contract 3 — Decomposition Gate):
   a. Build fixtures and locked Hybrid config (same construction as Contract 1).
   b. For each of the 7 fixtures (one result per scenario, not per-dataset):
      - For each dataset key in `fixture.datasets`, call `aggregate_hybrid(fixture.datasets[key], config=hybrid_cfg)` directly.
      - Assert no `RuntimeError` is raised. The invariant check at `candidates.py:468-472` raises `RuntimeError` if the sum of contribution terms deviates from the aggregate by >1e-8.
      - This is intentionally redundant with Contract 1's implicit coverage via `evaluate_fixture`. It serves as an independent regression gate that survives if Contract 1's evaluator structure changes.
   c. Collapse all sub-dataset results into a single pass/fail per scenario. Return 7 `(scenario_name, passed_bool, detail_string)` tuples.

6. **Implement `main()`**:
   a. Run all three test functions. Wrap guardrail's single tuple in a list to merge into a flat list of `(test_name, passed, detail)`.
   b. Print per-test `PASS`/`FAIL` lines.
   c. Print summary: `"N/M passed"`.
   d. Print contract metadata: version `1.0`, `bf_norm_c=0.083647`, `n_terms=1`, guardrail enabled.
   e. `sys.exit(0)` if all pass, else `sys.exit(1)`.

7. **Verify.** Run:
   ```bash
   cd research/adversarial-reward/prototypes/aggregation-candidates
   python acceptance_test.py
   ```
   Expected: 15 PASS lines (7 margin-parity + 1 guardrail + 7 decomposition), exit code 0.

8. **Post-test documentation.** Update `research/adversarial-reward/FINDINGS.md` with a Session 9 investigation log entry documenting the acceptance test suite creation: scope, method, findings, implications, open threads.

9. **Post-test issue management.** Close the beads issue (`athena-3lu`) and run `bd sync`.

END GOAL:
A single `acceptance_test.py` file that:
- Passes all 15 assertions (7 margin-parity + 1 guardrail + 7 decomposition) with exit code 0
- Reproduces the exact baseline margins from `aggregate_score_recommendation.json` within tolerance 1e-6
- Catches guardrail violations with the correct error code `GR-S2-CUSTOM-SIGMOID-X0-NONNEG`
- Confirms decomposition invariant holds for all 7 scenarios (sum of contributions == aggregate within 1e-8)
- Uses only Python stdlib (no external test frameworks)
- Is runnable via `python acceptance_test.py` from the `aggregation-candidates/` directory

NARROWING:
- Do NOT modify any existing files in `aggregation-candidates/` — this is a read-only contract over the existing codebase
- Do NOT use pytest, unittest, or any external testing framework — stdlib only
- Do NOT add type: ignore comments or suppress any runtime errors
- Do NOT hardcode margin values in the test file — read them from `aggregate_score_recommendation.json`
- Avoid catch-all exception handlers — catch only the specific exceptions each contract tests for (`ValueError` for guardrail, `RuntimeError` for decomposition)
- Stay within the existing module API — do not monkey-patch or modify imported modules
- Out of scope: new scenarios, new contracts, performance benchmarks
- The tolerance for margin comparison is 1e-6 (not 1e-7 or looser) — this was validated against Session 8's max delta of 4.414e-07
```

---

## Review Findings

### Issues Addressed
1. **Critical (fixed):** Clarified that `k` value in guardrail test is irrelevant — only `x0 < 0` triggers rejection
2. **Critical (fixed):** Clarified Step 5 decomposition test: iterates per-dataset but collapses to per-scenario results (7 total), calls `aggregate_hybrid` directly as independent gate
3. **Warning (fixed):** Made fixture iteration explicitly reference `build_scenario_fixtures()` return
4. **Warning (fixed):** Added S6 near-zero margin note with explanation
5. **Warning (fixed):** Added `Path(__file__).resolve().parent` path resolution
6. **Warning (fixed):** Separated post-test into documentation (Step 8) and issue management (Step 9)
7. **Warning (fixed):** Explicitly stated 7 decomposition results = per-scenario
8. **Warning (applied):** Removed `math` from required imports, noted as optional

### Remaining Suggestions
- `SCENARIO_KEYS` could be derived from JSON keys instead of hardcoded — but current approach is explicit and readable
- Contract metadata in `main()` could load from JSON rather than hardcode — consistent with "no hardcoded margins" principle but not strictly required for metadata constants
- Return type adapter (guardrail single-tuple wrapped in list) could be more explicit — noted in Step 6a

## Usage Notes

- **Best used with:** Claude Code / Claude Opus in a session with access to the `aggregation-candidates/` directory
- **Adjust for:** If scenario count changes, update `SCENARIO_KEYS` and expected PASS count accordingly
