# Prompt: Hybrid Aggregation Candidate Implementation

> Generated: 2026-02-21 | Framework: RISEN

---

## Session Goal

Implement a hybrid aggregation candidate (HTG-gated product combination) in the adversarial-reward research prototype that bolts HTG's confidence gating onto Fisher's product combination method, targeting 7/7 stress-test scenario passes where no single-family candidate exceeded 5/7.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation with clear sequential dependencies, specific methodology (HTG+Fisher hybrid), explicit constraints (ATHENA prototype rules), and measurable success criteria (7/7 scenario passes, reconstruction tolerance, ratio thresholds). RISEN's Steps + Narrowing map directly.
- **Alternatives considered:** TIDD-EC — good for precision requirements but its Do/Don't structure less natural than RISEN's sequential Steps for an implementation with dependencies.

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal unambiguous: implement hybrid candidate, verify 7/7 |
| Specificity | 9/10 | Exact formulas, parameter values, file locations, verification thresholds |
| Context | 9/10 | Prior session results, failure mode analysis, architectural rationale included |
| Completeness | 9/10 | Covers what/why/how, verification criteria, fallback sweep, documentation |
| Structure | 9/10 | RISEN components map naturally to sequential implementation |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a research prototype implementer with expertise in meta-analysis
aggregation methods (Fisher's method, inverse-variance weighting), statistical
signal combination (p-value products, chi-square CDFs), and Python scientific
computing. You are working within the ATHENA project's adversarial-reward
research track, extending an existing aggregation-candidates prototype that
evaluates candidate reward-aggregation functions against a 7-scenario stress
test.

INSTRUCTIONS:
Implement a hybrid aggregation candidate that combines HTG-Max's per-component
confidence gating (front end) with Fisher's product combination method using
n_terms=1 (back end). The hybrid exploits structurally complementary failure
modes: HTG gates well but can't compound weak concordant signals; Fisher
compounds well but has no noise filter. The hybrid should reuse existing helper
functions (normalize_component, gate_precision, sigmoid, chi_square_cdf_even_df)
— no new math primitives needed.

Key design decisions already made:

1. SE-dampening OFF in normalization — Noise filtering belongs in per-component
   gating via n/SE² (unit-independent), not global normalization which conflates
   divergence magnitude with measurement units (AbsDiff in eV vs ZScore
   dimensionless).

2. No Fisher reliability scaling — HTG confidence already accounts for
   measurement quality via log1p(n/SE²). Adding p^reliability would double-count
   sample size.

3. n_terms=1 in chi_square_cdf — This gives aggregate = 1 - exp(-total/2)
   = 1 - Π(p_i), the product combination method. This avoids the growing df=2N
   penalty that kills weak signal compounding (with df=16, ~26 total evidence
   needed vs ~2 with df=2). This is the critical design choice enabling S2.

4. c_missing vs c_floor — The hybrid introduces c_missing=0.7 as a SEPARATE
   parameter from c_floor=0.1. This is intentionally different from HTG-Max,
   which uses c_floor for both low-precision AND missing-precision cases.
   Rationale: a missing-precision component (no uncertainty data at all) should
   contribute more than a component with measured-but-very-low precision.
   c_floor=0.1 applies when precision exists but sigmoid output is near zero;
   c_missing=0.7 applies when no uncertainty data is available (e.g., 3 of 4
   S4 missing-data components).

STEPS:
1. Read the existing codebase: candidates.py (configs, aggregate functions,
   registry), evaluate.py (evaluation harness), normalization.py (shared
   helpers), models.py (data types), scenarios.py (7 stress-test fixtures).
   Understand the pattern used by IVW-CDF, HTG-Max, and Fisher-UP before
   writing any code.

2. Add HybridConfig dataclass to candidates.py after FisherUPConfig (~line 49):
   - Fields: alpha=1.5, tau=5.0, c_floor=0.1, c_missing=0.7, p_eps=1e-12,
     eps=1e-12, normalization=NormalizationConfig()
   - Note: tau=5.0 (not HTG's 7.8) — hybrid needs a lower precision threshold
     since gated scores feed into product combination, not max selection

3. Add aggregate_hybrid() function to candidates.py after aggregate_fisher_up().
   Per-component pipeline:
   a. Normalize: score = normalize_component(component, cfg.normalization)
      with SE-dampening OFF
   b. Precision: precision = gate_precision(component, cfg.eps)
      (returns float or None; note: requires eps argument — see normalization.py
      line 108 signature)
   c. Confidence: if precision is not None →
      max(cfg.c_floor, sigmoid(precision, cfg.alpha, cfg.tau));
      if precision is None → cfg.c_missing
   d. Gated score: gated = score * confidence
   e. P-value: p = max(cfg.p_eps, 1.0 - gated)
   f. Log evidence: log_ev = -2.0 * math.log(p)
   g. Sum all log_ev → total_log_ev
   h. Aggregate: chi_square_cdf_even_df(total_log_ev, n_terms=1)
   i. Weight decomposition: denom = sum(log_ev_i * score_i). If denom > eps:
      weight_i = log_ev_i * (aggregate / denom),
      contribution_i = weight_i * score_i.
      Algebraic proof: sum(weight_i * score_i) = aggregate.
      Note: this decomposition is algebraically exact for S6 reconstruction.
      The weights are scaling factors, not directly interpretable as evidence
      shares, because log_ev already incorporates confidence gating.
      eps serves dual purpose here (precision denominator and decomposition
      guard) — this is acceptable for a prototype.
   j. Return AggregateResult(aggregate_score, contributions, warnings, skipped)

4. Register hybrid in get_candidate_registry(): add
   hybrid_cfg: HybridConfig | None = None parameter, default to HybridConfig()
   if None, add "Hybrid": lambda components: aggregate_hybrid(components,
   hybrid_cfg) entry.

5. Update evaluate.py: add HybridConfig to the import from candidates. Update
   the get_candidate_registry() call in main() to pass
   hybrid_cfg=HybridConfig(normalization=normalization). No need to import
   aggregate_hybrid separately — the registry wraps it.

6. Run python evaluate.py from
   research/adversarial-reward/prototypes/aggregation-candidates/. Verify:
   - Existing candidates produce same results as Session 2
     (IVW 5/7, HTG 5/7, Fisher 3/7)
   - Hybrid achieves 7/7 (or 6/7 — S2 has 6.3% predicted margin)
   - S6 reconstruction: |sum(weight_i * score_i) - aggregate| <= 1e-8
   - S2 ratio: aggregate / max_single >= 1.5
   - S4 delta: relative_delta <= 0.20
   - All Hybrid scores in [0, 1], finite

7. If S2 fails, run a targeted parameter sweep: alpha in {1.0, 1.5, 2.0},
   tau in {4.0, 5.0, 6.0}, c_missing in {0.5, 0.7, 0.9} — 27 configs total.
   Focus on S2 margin vs S4 delta tradeoff. If S2 margin stays < 5%, document
   a recommendation in FINDINGS.md about whether relaxing the 1.5x compounding
   threshold to 1.3x is justified — do not change the evaluation harness
   threshold without explicit approval.

8. Document S2 sensitivity analysis: exact aggregate value, max_single value,
   and margin defined as: margin = (aggregate / (1.5 * max_single)) - 1.0,
   expressed as a percentage. Note the Custom metric's outsized role in setting
   the threshold (it has k=2.2, x0=0.0 giving the highest individual score
   ~0.582, while the other 7 metrics are all < 0.38).

9. Append a Session 3 log entry to research/adversarial-reward/FINDINGS.md at
   the top of the Investigation Log section. Use header format:
   ### 2026-02-21 -- WDK#41 Session 3: [descriptive title]
   Follow the append-only protocol: Scope, Method, Findings, Implications,
   Open Threads. Update the Accumulated Findings section (What We Know /
   What We Suspect / What We Don't Know) with new evidence.

END GOAL:
A working hybrid aggregation candidate integrated into the evaluation harness
that:
- Achieves 7/7 scenario passes (or 6/7 with documented S2 sensitivity analysis
  showing margin < threshold)
- Preserves backward compatibility (existing 3 candidates produce identical
  results)
- Satisfies S6 exact decomposition within floating-point tolerance (≤ 1e-8)
- Has all scores bounded in [0, 1] and finite
- Is documented in FINDINGS.md with a complete Session 3 investigation log
  entry citing evidence for all claims
- Uses only existing helper functions — no new math primitives

NARROWING:
- Do NOT write production code. This is a research prototype — throwaway
  artifact.
- Do NOT add SE-dampening to normalization. Noise filtering is handled by
  per-component confidence gating.
- Do NOT add Fisher reliability scaling (p^reliability). HTG confidence already
  accounts for measurement quality.
- Do NOT use n_terms=N in chi_square_cdf (where N = number of components).
  Use n_terms=1 to avoid the df=2N penalty.
- Do NOT modify existing aggregate functions (IVW-CDF, HTG-Max, Fisher-UP) or
  their configs.
- Do NOT edit or delete previous Investigation Log entries in FINDINGS.md —
  append only.
- Do NOT create new files. Add HybridConfig and aggregate_hybrid to existing
  candidates.py; update existing evaluate.py.
- Do NOT use c_floor for the missing-precision case. Use c_missing (separate
  parameter) — see design decision #4 in INSTRUCTIONS.
- Avoid premature abstraction — the hybrid is one specific combination, not a
  generic framework.
- Stay within existing codebase patterns: same import style, same
  AggregateResult return type, same registry pattern.
- Out of scope: production hardening, CLI flags, visualization, any work
  outside the aggregation-candidates prototype directory.
```

---

## Review Findings

### Issues Addressed
1. **[Critical] gate_precision signature** — Fixed Step 3b to pass `cfg.eps` as required second argument
2. **[Critical] c_missing vs c_floor semantics** — Added design decision #4 in INSTRUCTIONS explaining the intentional split with rationale; added NARROWING constraint against conflating them
3. **[Warning] Step 5 ambiguity** — Rewrote to reference `get_candidate_registry()` call explicitly, noted no separate `aggregate_hybrid` import needed
4. **[Warning] S2 threshold relaxation scope** — Clarified as documentation recommendation, not code change
5. **[Warning] Weight decomposition semantics** — Added note that weights are scaling factors for algebraic reconstruction, not interpretable evidence shares
6. **[Warning] Margin definition** — Defined explicitly as `(aggregate / (1.5 * max_single)) - 1.0` percentage
7. **[Warning] eps dual use** — Acknowledged in Step 3i as acceptable for prototype

### Remaining Suggestions
- Could add concrete example of c_floor vs c_missing triggering conditions (partially addressed in design decision #4)
- Step 6 backward compatibility check is redundant with NARROWING but harmless — kept for verification completeness
- Parameter sweep runtime note omitted (trivial computation, not worth the prompt space)
- n_terms=1 rationale elevated to design decision #3 in INSTRUCTIONS for scannability
- Log entry date format specified in Step 9

## Usage Notes

- **Best used with:** Claude Opus or Sonnet in a fresh context window with access to the athena repository
- **Adjust for:** If Session 2 results have changed (verify IVW/HTG/Fisher pass counts), update backward compatibility expectations in Step 6
