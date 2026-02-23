# Session Handoff: Acceptance Test Suite for Locked AggregateScore Contract

> Generated: 2026-02-22 | Handoff #20 | Previous: handoff_019_2026-02-22_implementation-bf-normalization-seam.md

---

## Continuation Directive

Write the acceptance test suite (bead athena-3lu) for the adversarial-reward aggregation prototype. This is a single Python script (`acceptance_test.py` in `aggregation-candidates/`) encoding three contract checks as automated assertions. Do NOT modify any implementation files -- only create the test.

Three contracts to test:
1. **Margin parity** -- 7-scenario suite margins match `baseline_margins` from locked JSON within 1e-6 absolute tolerance.
2. **Guardrail rejection** -- `NormalizationConfig` with negative x0 raises `ValueError` containing guardrail ID `GR-S2-CUSTOM-SIGMOID-X0-NONNEG`.
3. **Decomposition invariant** -- `aggregate_hybrid()` does not raise `RuntimeError` during normal evaluation (invariant holds).

Scope boundary: Just the test. No implementation changes. No monitoring hooks (athena-i4s).

## Task Definition

**Project:** ATHENA -- falsification-driven AI co-scientist. The adversarial reward aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` scalar.

**Goal:** Automated regression gate for the three contracts implemented in Session 8 (BF normalization seam, x0 guardrail, decomposition invariant). The test makes the locked AggregateScore recommendation enforceable, not just verified-once.

**Constraints:** Stdlib-only Python, all code in `research/adversarial-reward/prototypes/aggregation-candidates/`. See CLAUDE.md for governance.

## Key Decisions & Rationale

1. **Log-scaled BF normalization is the locked default** -- `bf_norm_log_scaled(bf, c) = log1p(bf) / (log1p(bf) + c)` with `c=0.083647`. Replaces the original `bf/(bf+1)` which hit a ceiling at BF=110.
   - Source: `aggregate_score_recommendation.md` Section 1.2, bead athena-6ax.

2. **BF norm seam is a configurable callable field** -- `NormalizationConfig.bf_norm_fn: Callable[[float], float]`, defaults to `bf_norm_log_scaled`. Non-BF branches untouched.
   - Source: Recommendation Section 6 note 1; implemented in Session 8 (athena-4xm).

3. **x0 >= 0 enforced at construction, no clamping** -- `__post_init__` raises `ValueError`, doesn't silently fix.
   - Source: `guardrail_spec.md`, recommendation Section 1.4; implemented in Session 8 (athena-8b9).

4. **Decomposition invariant tolerance is 1e-8** -- `abs(sum(contributions) - aggregate) <= 1e-8` enforced in `aggregate_hybrid()`.
   - Source: Recommendation Section 6 note 3; implemented in Session 8 (athena-fgo).

5. **Margin parity tolerance is 1e-6 absolute, per-scenario** -- Verified in Session 8: max observed delta was 4.414e-07.

## Current State

### Completed (Sessions 1-8)
- Sessions 1-7: Research, locked recommendation, architecture integration, implementation beads created.
- Session 8: Implemented athena-4xm (BF seam), athena-8b9 (x0 guardrail), athena-fgo (decomposition invariant). All verified: 7/7 PASS, max margin delta 4.414e-07. Commit `6e8ccfb`.

### In Progress
- **athena-3lu** (acceptance test suite) -- OPEN, now unblocked. Ready to implement.

### Blocked / Open Questions
- **athena-i4s** (monitoring hooks) -- OPEN, independent. Requires production instrumentation decisions not yet made.
- Open: Should the acceptance test also verify that the decomposition invariant *fires* when synthetically broken? (Optional enhancement, not required by spec.)

## Key Code Context

**Margin computation** (`ceiling_analysis.py:294-321`) -- `margin_from_cell(cell)` returns scenario-specific margin values. Each scenario has a different formula (S1: base-doubled, S2: ratio/1.5-1, etc.). The acceptance test must use this function.

**Baseline margins** (`aggregate_score_recommendation.json:43-51`):
```json
"baseline_margins": {
    "S1_noisy_tv": 0.030647,
    "S2_unanimous_weak_signal": 0.072804,
    "S3_mixed_signal": 0.006164,
    "S4_missing_data": 0.128007,
    "S5_scale_heterogeneity": 0.008802,
    "S6_calibration_decomposability": 0.000000,
    "S7_boundary_seeking": 0.102971
}
```

**Evaluation pipeline** -- `evaluate.py:main()` builds fixtures via `build_scenario_fixtures()`, constructs `NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)`, creates `HybridConfig(normalization=normalization)`, then runs `evaluate_fixture(fixture, name, fn)` per scenario.

**Guardrail error format**: `"GR-S2-CUSTOM-SIGMOID-X0-NONNEG: custom_sigmoids['test'] has x0=-0.2; expected x0 >= 0"`

## Files Map

| Path | Role | Status |
|------|------|--------|
| `.../aggregation-candidates/normalization.py` | BF seam + guardrail (modified Session 8) | READ ONLY for test |
| `.../aggregation-candidates/candidates.py` | Decomposition invariant (modified Session 8) | READ ONLY for test |
| `.../aggregation-candidates/ceiling_analysis.py` | `margin_from_cell()` for margin computation | READ ONLY for test |
| `.../aggregation-candidates/evaluate.py` | `evaluate_fixture()`, `build_scenario_fixtures()`, `DEFAULT_CUSTOM_SIGMOIDS` | READ ONLY for test |
| `.../aggregation-candidates/aggregate_score_recommendation.json` | Locked baselines source of truth | READ ONLY |
| `.../aggregation-candidates/aggregate_score_acceptance_test_spec.md` | Test categories, tolerances, structure | Reference spec |
| `.../aggregation-candidates/acceptance_test.py` | **TO CREATE** -- the acceptance test suite | New file |

## Loop State

**Iteration 2** of Claude-Codex-Claude workflow:
- **Iteration 1**: Codex implemented athena-4xm/8b9/fgo. Claude verified: all correct, 7/7 PASS, max margin delta 4.414e-07. Commit `6e8ccfb` pushed.
- **Iteration 2**: Next session writes athena-3lu (acceptance test suite). Can be Claude-direct or Claude-Codex depending on complexity.

## Next Steps

1. **Read** `aggregate_score_acceptance_test_spec.md` for the formal test contract (categories A-D, tolerances, reporting).
2. **Read** `bd show athena-3lu` for bead acceptance criteria.
3. **Create** `acceptance_test.py` with three test categories:
   - Category A: Margin parity (7 scenarios, 1e-6 tolerance, using `margin_from_cell` + locked JSON baselines).
   - Category B: Guardrail rejection (negative x0 -> ValueError with correct ID/key/value).
   - Category C: Decomposition invariant (no RuntimeError during normal evaluation).
4. **Run** `python acceptance_test.py` and confirm all assertions pass.
5. **Update** `FINDINGS.md` -- Session 9 log entry.
6. **Close** athena-3lu, commit, sync, push.

## Session Artifacts

- Prompt #18: `.claude/prompts/prompt_018_2026-02-22_bf-norm-seam-contract-enforcement.md`
- Previous handoff: `.claude/handoffs/handoff_019_2026-02-22_implementation-bf-normalization-seam.md`
- Implementation commit: `6e8ccfb` (Session 8 -- BF normalization seam + contract enforcement)
- FINDINGS.md updated with Session 8 investigation log entry.

## Documentation Updated

No documentation updates -- all project docs were current. CLAUDE.md and AGENTS.md verified; project remains in "Research (Active Investigation)" phase.
