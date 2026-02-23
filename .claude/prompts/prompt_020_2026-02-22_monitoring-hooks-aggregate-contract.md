# Prompt: Monitoring Hooks for Locked AggregateScore Contract

> Generated: 2026-02-22 | Framework: RISEN

---

## Session Goal

Implement continuous monitoring hooks (triggers T1-T5, contract metadata emission, deterministic drift report) for the locked AggregateScore contract, converting Session 9's one-time acceptance proof into an ongoing regression alarm system.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation with clear sequential steps (instrument 5 triggers + metadata validation + report runner), defined role (instrumentation engineer), explicit success criteria (deterministic pass/fail, exit 0/1), and important boundaries (monitoring only, no algorithm changes).
- **Alternatives considered:** TIDD-EC — strong dos/don'ts but the task is fundamentally process-driven with ordered implementation steps.

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Each trigger check is unambiguous after review fixes |
| Specificity | 9/10 | JSON paths, code constants, tolerances all specified |
| Context | 9/10 | Full trigger spec, JSON schema, codebase references |
| Completeness | 9/10 | All 5 triggers + metadata + runner + post-steps |
| Structure | 9/10 | Clean RISEN with numbered steps and sub-steps |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are an instrumentation engineer adding continuous monitoring to an existing research prototype. You have access to the `aggregation-candidates/` codebase — specifically `normalization.py`, `candidates.py`, `models.py`, `scenarios.py`, and `acceptance_test.py`. You understand the locked AggregateScore contract from `aggregate_score_recommendation.json` and the trigger specifications in `monitoring_triggers.md`. You write stdlib-only Python consistent with the existing prototype style.

INSTRUCTIONS:
Create a single file `monitoring_hooks.py` in the `aggregation-candidates/` directory. This file instruments the five revisit triggers (T1-T5) from `monitoring_triggers.md`, validates contract metadata from `aggregate_score_recommendation.json`, and produces a deterministic pass/fail monitoring report. It complements `acceptance_test.py` (Session 9's one-time proof) by making contract protection continuous. The file must be runnable via `python monitoring_hooks.py` from the `aggregation-candidates/` directory.

STEPS:

1. **Set up imports.** At the top of `monitoring_hooks.py`, import:
   ```python
   import json, sys
   from pathlib import Path
   from models import DivergenceKind, SigmoidParams
   from normalization import NormalizationConfig, BF_NORM_LOG_SCALED_C
   from scenarios import DEFAULT_CUSTOM_SIGMOIDS, build_scenario_fixtures
   ```
   Import only what the check functions actually use. Do not import `evaluate_fixture`, `margin_from_cell`, `HybridConfig`, or `aggregate_hybrid` — those belong to `acceptance_test.py`, not monitoring.

2. **Load contract metadata.** Read `aggregate_score_recommendation.json` via `Path(__file__).resolve().parent / "aggregate_score_recommendation.json"`. Extract and store:
   - `recommendation.version` (expect `"1.0"`)
   - `recommendation.status` (expect `"LOCKED"`)
   - `recommendation.date` (used for deterministic timestamp in report footer)
   - `parameters.bf_normalization.c` (expect `0.083647`)
   - `parameters.hybrid_config.n_terms` (expect `1`)
   - `parameters.normalization_config.absolute_difference_sigmoid` (expect `k=1200.0, x0=0.0007`)
   - `guardrails` (full list — used for metadata validation)
   - `operating_envelope` (full dict — used by T1)
   - `accepted_limitations` (full list — used by T3)
   - `revisit_triggers` (full list — used for trigger ID validation)

3. **Implement `check_t1_operating_envelope() -> list[tuple[str, bool, str]]`** (T1 — Empirical operating range violation):
   a. Check that BF normalization constant in code (`BF_NORM_LOG_SCALED_C` from `normalization.py`) matches the JSON value `parameters.bf_normalization.c` within `1e-8`.
   b. Check that `NormalizationConfig()` default sigmoid parameters (`k`, `x0`) match `parameters.normalization_config.absolute_difference_sigmoid` from JSON within `1e-10`.
   c. Check that each custom sigmoid in `DEFAULT_CUSTOM_SIGMOIDS` has `x0` within the inclusive range `[operating_envelope.custom_sigmoid_x0.validated_min, operating_envelope.custom_sigmoid_x0.validated_max]` (i.e., `validated_min <= x0 <= validated_max`), and `k` within the inclusive range `[operating_envelope.custom_sigmoid_k.validated_min, operating_envelope.custom_sigmoid_k.validated_max]`.
   d. Return per-check `(check_name, passed_bool, detail_string)` tuples.

4. **Implement `check_t2_divergence_coverage() -> list[tuple[str, bool, str]]`** (T2 — New DivergenceKind addition):
   a. Enumerate all members of `DivergenceKind` from `models.py`.
   b. Hardcode the known covered set: `{AbsoluteDifference, ZScore, BayesFactor, KLDivergence, EffectSize, Custom}`. These are the 6 kinds handled by `normalize_component`'s `if kind is DivergenceKind.X` dispatch chain. T2 trips when `DivergenceKind` gains a new member not in this set. (Runtime introspection of normalize_component is not needed — the check detects new enum members.)
   c. Assert that every `DivergenceKind` member is in the covered set.
   d. Return per-kind `(kind_name, covered_bool, detail)` tuples.

5. **Implement `check_t3_pattern_b_metadata() -> tuple[str, bool, str]`** (T3 — Pattern B recovery becomes blocking):
   a. Load `accepted_limitations` from the JSON.
   b. Find the entry with `id == "L1"` (Pattern B under-response).
   c. Read `observed_value` and `threshold` from the JSON entry. Assert `classification == "out_of_range"` and `observed_value < threshold`, confirming Pattern B is still accepted-but-not-blocking. Do not hardcode the threshold or observed value — read both from JSON.
   d. Return a single `(check_name, passed_bool, detail)` tuple.

6. **Implement `check_t4_scenario_coverage() -> list[tuple[str, bool, str]]`** (T4 — Scenario suite expansion):
   a. Call `build_scenario_fixtures()` and collect the set of fixture `.idx` values returned.
   b. Define the expected baseline set: `{1, 2, 3, 4, 5, 6, 7}` (matching the 7 locked scenarios).
   c. Assert no new fixture indices beyond the baseline set exist (new indices would trip T4).
   d. Assert no baseline indices are missing.
   e. Return per-check results.

7. **Implement `check_t5_correlation_envelope() -> list[tuple[str, bool, str]]`** (T5 — Correlation structure change):
   a. Load `operating_envelope.correlation_rho` from the JSON.
   b. Assert `max_inflation` (from JSON) is below the T5 threshold of `1.5`.
   c. If `correlation_results.json` exists in the same directory, load it and check that no recorded `inflation_ratio` exceeds `1.5`.
   d. Return per-check results.

8. **Implement `check_contract_metadata() -> list[tuple[str, bool, str]]`** (Contract metadata validation):
   a. Assert `recommendation.version == "1.0"`.
   b. Assert `recommendation.status == "LOCKED"`.
   c. Assert `abs(parameters.bf_normalization.c - BF_NORM_LOG_SCALED_C) <= 1e-8` (code-JSON parity).
   d. Assert `parameters.hybrid_config.n_terms == 1`.
   e. Assert guardrail `GR-S2-CUSTOM-SIGMOID-X0-NONNEG` is present in the `guardrails` list and has `enforcement == "reject_at_config_construction"`.
   f. Return per-check results.

9. **Implement `main()`**:
   a. Run all check functions (T1 through T5 + contract metadata), collecting results into a flat list of `(check_name, passed, detail)`.
   b. Print section headers for each trigger group (e.g., `"--- T1: Operating Envelope ---"`).
   c. Print per-check `PASS`/`FAIL` lines.
   d. Print summary: `"N/M checks passed"`.
   e. If any checks failed, print a `TRIGGER ALERT` block listing each failed check's trigger ID and a one-line action summary (e.g., `"T2 tripped: open bead tagged revisit-T2; implement normalization for new kind"`). Do not print the full action path prose from `monitoring_triggers.md`.
   f. Print contract metadata footer: version, bf_norm_c, n_terms, guardrail status, and `recommendation.date` from JSON (not current wall-clock time — ensures determinism).
   g. `sys.exit(0)` if all pass, else `sys.exit(1)`.

10. **Verify.** Run:
    ```bash
    cd research/adversarial-reward/prototypes/aggregation-candidates
    python monitoring_hooks.py
    ```
    Expected: all checks pass, exit code 0, no trigger alerts. Approximate check count: T1 (3-5 checks), T2 (6 checks), T3 (1 check), T4 (2 checks), T5 (1-2 checks), metadata (5 checks) = 18-20 total.

11. **Post-implementation documentation.** Update `research/adversarial-reward/FINDINGS.md` with a Session 10 investigation log entry documenting: scope (monitoring hooks for locked contract), method (T1-T5 instrumentation + metadata validation), findings (all checks pass, trigger coverage), implications (continuous protection vs. one-time proof), open threads (CI integration, production telemetry wiring).

12. **Post-implementation issue management.** Close bead `athena-i4s` and run `bd sync`.

END GOAL:
A single `monitoring_hooks.py` file that:
- Instruments all 5 revisit triggers (T1-T5) as executable checks
- Validates contract metadata (version, status, bf_norm_c, n_terms, guardrail) against code constants
- Produces a deterministic pass/fail monitoring report (same codebase + same JSON = same output)
- Prints specific trigger alerts with action summaries on failure
- Complements `acceptance_test.py` — acceptance tests verify correctness once; monitoring hooks detect drift continuously
- Uses only Python stdlib
- Exits 0 when all triggers clear, exits 1 with trigger alerts when any trip

NARROWING:
- Do NOT modify any existing files in `aggregation-candidates/` — monitoring is read-only observation
- Do NOT use pytest, unittest, or any external testing framework — stdlib only
- Do NOT implement new aggregation algorithms or modify the locked contract — this is monitoring, not development
- Do NOT hardcode threshold values that exist in `aggregate_score_recommendation.json` — read them from the JSON
- Avoid catch-all exception handlers — catch only specific exceptions relevant to each trigger check
- Stay within the existing module API — do not monkey-patch or modify imported modules
- Out of scope: production telemetry emission, CI pipeline configuration, new scenarios, algorithm changes
- The monitoring report must be deterministic — same codebase + same JSON = same output every time (use JSON's recommendation.date, not wall-clock time)
```

---

## Review Findings

### Issues Addressed
1. **Critical (fixed):** Removed unused imports (`evaluate_fixture`, `margin_from_cell`, `HybridConfig`, `aggregate_hybrid`) from Step 1 with explicit note to not import them
2. **Critical (fixed):** T2 check now explicitly states hardcoded covered set and explains that T2 detects new enum members, not missing normalization branches at runtime
3. **Critical (fixed):** Replaced fragile line number references with logic-based references (e.g., "the `if kind is DivergenceKind.X` dispatch chain")
4. **Warning (fixed):** T1 boundary comparison clarified as inclusive (`<=`)
5. **Warning (fixed):** T3 removed hardcoded parenthetical values; reads from JSON per NARROWING rule
6. **Warning (fixed):** T4 clarified that baseline set comes from fixture `.idx` values
7. **Warning (fixed):** Action path output specified as trigger ID + one-line summary, not full prose
8. **Warning (fixed):** Step 3b sigmoid param comparison uses `1e-10` tolerance
9. **Warning (fixed):** Timestamp uses `recommendation.date` from JSON for determinism

### Remaining Suggestions
- Check count approximation tightened to 18-20 (from 18-21)
- Type alias for `list[tuple[str, bool, str]]` could reduce repetition but is cosmetic
- END GOAL partially overlaps INSTRUCTIONS — acceptable for a task prompt where redundancy aids clarity

## Usage Notes

- **Best used with:** Claude Code / Claude Opus in a session with access to the `aggregation-candidates/` directory
- **Adjust for:** If trigger thresholds change in `aggregate_score_recommendation.json`, the monitoring hooks will automatically pick up new values (by design)
