# Prompt: BF Normalization Seam + Contract Enforcement

> Generated: 2026-02-22 | Prompt #1 | Framework: RISEN

---

## Session Goal

Implement a three-part change to the adversarial-reward aggregation prototype: (1) add a configurable BF normalization seam defaulting to `bf_norm_log_scaled(bf, c=0.083647)` with non-BF branches unchanged, (2) enforce x0 >= 0 at config construction with explicit guardrail errors (no clamping), (3) enforce decomposition invariant `abs(sum(contributions) - aggregate) <= 1e-8` in `candidates.py`. Verify against the 7-scenario suite with baseline margin parity within 1e-6, not just pass/fail. No algorithm retuning.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation with clear methodology (three sequential beads), explicit constraints (no retuning, no clamping), and a quantitative end goal (margin parity within 1e-6). RISEN's Role/Instructions/Steps/End Goal/Narrowing structure maps directly onto the task.
- **Alternatives considered:** TIDD-EC (good for dos/don'ts but lacks sequential step structure), Chain of Thought (overkill — reasoning steps already specified in plan)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Unambiguous targets with exact function signatures, line references, and guardrail IDs |
| Specificity | 10/10 | Quantitative thresholds (1e-6, 1e-8, c=0.083647), exact file paths, named error types |
| Context | 8/10 | Assumes familiarity with codebase structure; Role section provides framing |
| Completeness | 9/10 | All 5 user targets covered; verification steps explicit |
| Structure | 9/10 | RISEN components map cleanly; steps numbered with substeps; narrowing specific |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are an implementation engineer working on the ATHENA adversarial-reward
aggregation prototype. You have full context on the locked AggregateScore
recommendation (bf_norm_log_scaled with c=0.083647) and the existing codebase
in research/adversarial-reward/prototypes/aggregation-candidates/. You
understand frozen dataclasses, callable config fields, and the normalize →
aggregate → evaluate pipeline.

INSTRUCTIONS:
Make surgical edits to three files (normalization.py, ceiling_analysis.py,
candidates.py). Each change is a discrete bead (athena-4xm, athena-8b9,
athena-fgo). Preserve all non-BF normalization branches exactly as-is.
Default behavior must match the locked recommendation — no tuning, no
parameter changes, no algorithm modifications. Every change must be
verifiable against the existing 7-scenario evaluation suite with quantitative
margin parity (within 1e-6 of baseline_margins from
aggregate_score_recommendation.json), not just pass/fail labels.

STEPS:
1. BF Normalization Seam (athena-4xm) — normalization.py:
   a. Define bf_norm_log_scaled(bf, c=0.083647) using
      math.log1p(bf) / (math.log1p(bf) + c). Place it before the sigmoid
      function. Export constant BF_NORM_LOG_SCALED_C = 0.083647
      (source: aggregate_score_recommendation.json::parameters.bf_normalization.c).
   b. Add bf_norm_fn: Callable[[float], float] field to NormalizationConfig
      (frozen dataclass), defaulting via default_factory=lambda: bf_norm_log_scaled.
      Callers invoke config.bf_norm_fn(bf) — c=0.083647 is used via the
      function's parameter default, no partial application needed.
   c. In normalize_component(), replace ONLY the BayesFactor branch
      (lines ~136-138) from hard-coded "1 - 1/(1+bf)" to
      "config.bf_norm_fn(bf)". Leave all other branches unchanged.
   d. Add Callable to the typing imports (update existing import line).
   e. In ceiling_analysis.py: import bf_norm_log_scaled from normalization
      instead of defining it locally; remove the local duplicate definition
      (lines ~95-97). Keep bf_norm_current, bf_norm_power_law,
      bf_norm_exp_decay — those are ceiling-analysis-specific research candidates.

2. x0 >= 0 Config-Time Guardrail (athena-8b9) — normalization.py:
   a. Add __post_init__ to NormalizationConfig that iterates custom_sigmoids
      and raises ValueError if any SigmoidParams.x0 < 0.
   b. Error message must include guardrail ID "GR-S2-CUSTOM-SIGMOID-X0-NONNEG",
      the offending key name, and the actual x0 value.
   c. No silent clamping — reject at construction time.

3. Decomposition Invariant (athena-fgo) — candidates.py:
   a. At the end of aggregate_hybrid(), after contributions list is built
      and before return, check:
      abs(sum(c.contribution for c in contributions) - aggregate) <= 1e-8.
   b. If violated, raise RuntimeError with the magnitude of the discrepancy.

4. Verify all changes:
   a. Run python evaluate.py from the aggregation-candidates/ directory.
   b. Confirm all 7 scenarios PASS for Hybrid.
   c. Confirm per-scenario margin values match
      aggregate_score_recommendation.json::baseline_margins within 1e-6
      absolute tolerance.
   d. Confirm NormalizationConfig(custom_sigmoids={"test": SigmoidParams(k=1.0, x0=-0.2)})
      raises ValueError containing "GR-S2-CUSTOM-SIGMOID-X0-NONNEG".
   e. Confirm no RuntimeError from the decomposition invariant during
      normal evaluation.

END GOAL:
After implementation, all of the following must hold simultaneously:
- bf_norm_log_scaled is the default BF normalization, callable and overridable
  via NormalizationConfig.bf_norm_fn.
- Non-BF normalization branches are identical to their pre-change state.
- Negative x0 in custom_sigmoids is rejected at NormalizationConfig construction
  with a descriptive ValueError (guardrail GR-S2-CUSTOM-SIGMOID-X0-NONNEG).
- aggregate_hybrid() raises RuntimeError if decomposition drift exceeds 1e-8.
- evaluate.py reports 7/7 PASS with per-scenario margin values matching
  baseline_margins within 1e-6 absolute tolerance.
- No algorithm retuning or parameter changes beyond the specified seam insertion.

NARROWING:
- Do NOT modify candidates.py's aggregate logic, only add the invariant assertion.
- Do NOT change models.py, scenarios.py, or evaluate.py.
- Do NOT retune any weights, thresholds, or sigmoid parameters.
- Do NOT silently clamp invalid config values — always raise.
- Do NOT change non-BF branches in normalize_component.
- Do NOT remove ceiling_analysis.py's other BF norm research candidates
  (bf_norm_current, bf_norm_power_law, bf_norm_exp_decay) — only remove the
  duplicate bf_norm_log_scaled definition.
- Stay within the aggregation-candidates/ directory. No changes to parent
  research files.
```

---

## Review Findings

### Issues Addressed
- Clarified default_factory semantics: lambda returns function directly, c used via parameter default (reviewer C1/C2/W6)
- Added source reference for BF_NORM_LOG_SCALED_C constant (reviewer W1)
- Specified "ONLY the BayesFactor branch" in Step 1.c for precision (reviewer W3)
- Clarified "per-scenario" and "absolute tolerance" in verification steps (reviewer C5/W5)

### Remaining Suggestions
- Could add explicit evidence citations to aggregate_score_recommendation.json section numbers (S1)
- Test infrastructure is evaluate.py; no separate test runner needed for verification (S2)
- n_terms locking is covered by the "no algorithm retuning" narrowing constraint (S4)

## Usage Notes

- **Best used with:** Claude Opus 4.6 or Sonnet 4.6 with full codebase access
- **Adjust for:** If baseline_margins values in the JSON change, update the 1e-6 comparison targets accordingly
