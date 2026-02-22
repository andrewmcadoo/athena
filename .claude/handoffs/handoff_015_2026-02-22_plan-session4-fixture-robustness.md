# Session Handoff: WDK#41 Session 4 — Fixture Perturbation Robustness

> Generated: 2026-02-21 | Handoff #15 | Previous: handoff_014_2026-02-21_plan-session3-hybrid-aggregation.md

---

## Continuation Directive

Plan and implement Session 4 for the adversarial reward aggregation research (WDK#41). The hybrid candidate achieved 7/7 on fixed fixtures in Session 3 — now stress-test whether that result is structurally robust or fixture-tuned. Perturb fixture parameters (noise levels, correlation structure, missingness patterns) and determine which scenarios are fragile. Beads issue: `athena-bop`.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. Aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` reward scalar for the adversarial experiment designer.

**Goal:** Determine whether the hybrid's 7/7 is robust under fixture perturbation, or if it depends on specific synthetic parameter choices. Identify fragile scenarios and quantify margins.

**Constraints:** Stdlib only, frozen dataclasses, throwaway prototype code. All work in `research/adversarial-reward/prototypes/aggregation-candidates/`. See `CLAUDE.md` for full project constraints.

## Key Decisions & Rationale

1. **Hybrid architecture: HTG gating + Fisher product (n_terms=1)** — HTG confidence gating (front end) filters noise via `log1p(n/SE²)` precision → sigmoid → confidence. Fisher product combination (back end) compounds concordant evidence via `1 - Π(p_i)` where `p = max(eps, 1 - gated_score)`. Using `n_terms=1` avoids the `df=2N` penalty that killed S2 in pure Fisher.
   - **Alternatives rejected:** n_terms=N (df=16 requires ~26 total evidence for significance — far too much for weak signals); Fisher reliability scaling (would double-count sample size already in HTG gating); SE-dampening in normalization (conflates magnitude with units).

2. **c_missing=0.7 separate from c_floor=0.1** — Missing-precision components (no uncertainty data) get higher default confidence than measured-but-very-low precision components. HTG-Max uses `c_floor` for both cases; the hybrid intentionally splits them.
   - **Rationale:** S4 (missing data) needs missing components to contribute meaningfully (c_missing=0.7), while S7 (boundary) needs low-precision components to be suppressed (c_floor=0.1).

3. **Default hybrid params: alpha=1.5, tau=5.0** — Lower tau than HTG-Max (7.8) because gated scores feed into product combination, not max selection. The lower threshold lets more components contribute to the product.

4. **S2 passes with 7.28% margin, driven by a single metric** — `s2.custom.1` (score 0.582) sets the max_single threshold. Other 7 metrics are all < 0.38. This concentration is a robustness concern — if the Custom metric's sigmoid parameters change, S2 could fail.

## Current State

### Completed
- **Session 1:** 3 candidates, 3×7 stress test → IVW 5/7, HTG 5/7, Fisher 3/7
- **Session 2:** 4 structural fixes, two-stage sweep (723 configs), calibration sim, correlation robustness. No 7/7 found.
- **Session 3:** Hybrid candidate implemented and verified. 7/7 on all scenarios. S2 ratio=1.609, S4 delta=0.072, S6 error=1.1e-16. All bounded/finite. FINDINGS.md updated with Session 3 log entry.
- **Prompt engineering:** RISEN prompt for Session 3 saved as `prompt_012`.
- **Beads:** `athena-btk` (Session 3) closed. Three new issues created for Sessions 4-6.

### In Progress
- Nothing. Session 3 is complete.

### Blocked / Open Questions
- **S2 margin fragility:** 7.28% margin is positive but tight. How sensitive is it to fixture parameter variation?
- **Correlation regime:** Session 2 correlation test had floor-saturated Fisher aggregates (~1e-12). Does the hybrid avoid floor saturation in correlated weak-signal regimes?
- **Fixture representativeness:** All 7 scenarios are hand-crafted. Do they cover the real problem space, or are there blind spots?

## Key Code Context

**Hybrid pipeline** (`candidates.py:383-474`) — the complete per-component flow:
```python
precision = gate_precision(component, cfg.eps)  # log1p(n/SE²) or None
confidence = cfg.c_missing if precision is None else max(cfg.c_floor, sigmoid(precision, cfg.alpha, cfg.tau))
gated_score = score * confidence
p_value = max(cfg.p_eps, 1.0 - gated_score)
log_evidence = -2.0 * math.log(p_value)
# After all components:
aggregate = chi_square_cdf_even_df(total_log_evidence, n_terms=1)
# Decomposition: weight_i = log_ev_i * (aggregate / sum(log_ev_j * score_j))
```

**Scenario fixtures** (`scenarios.py`) — these are what need perturbation. Each scenario uses `build_scenario_fixtures()` which constructs `MetricComponent` objects with specific `value`, `standard_error`, `n_observations`, `custom_sigmoid` params.

**Hybrid config** (`candidates.py:53-60`):
```python
class HybridConfig:
    alpha: float = 1.5; tau: float = 5.0; c_floor: float = 0.1
    c_missing: float = 0.7; p_eps: float = 1e-12; eps: float = 1e-12
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/adversarial-reward/prototypes/aggregation-candidates/candidates.py` | 4 candidates (IVW, HTG, Fisher, Hybrid) + configs + registry | Modified (S3) |
| `research/adversarial-reward/prototypes/aggregation-candidates/evaluate.py` | 4×7 evaluation harness | Modified (S3) |
| `research/adversarial-reward/prototypes/aggregation-candidates/normalization.py` | Normalization + SE-dampening + gating helpers | Modified (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/scenarios.py` | 7 fixed scenario fixtures | Unchanged |
| `research/adversarial-reward/prototypes/aggregation-candidates/models.py` | Dataclass contracts | Unchanged |
| `research/adversarial-reward/prototypes/aggregation-candidates/results.json` | Latest 4×7 evaluation output | Regenerated (S3) |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep.py` | Session 2 parameter sweep | Created (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/correlation_test.py` | Fisher correlation robustness | Created (S2) |
| `research/adversarial-reward/FINDINGS.md` | Master research log (3 session entries) | Modified (S3) |

## Loop State

- **Iteration:** 3 complete (S1→S2→S3), planning S4
- **Last prompt to Codex:** Session 3 RISEN prompt (`.claude/prompts/prompt_012_2026-02-21_hybrid-aggregation-candidate.md`)
- **Codex result:** All steps completed cleanly. Hybrid implemented, 7/7 verified, FINDINGS.md updated.
- **Review findings:** All verification criteria passed. No issues found in implementation. S2 margin (+7.28%) noted as tight but passing.

## Next Steps

1. **Read FINDINGS.md Session 3 log** (lines 33-91) — understand the hybrid's exact results and open threads
2. **Read `scenarios.py`** — understand fixture construction to design perturbations
3. **Design perturbation strategy** — identify which fixture parameters to vary per scenario:
   - S1: noise multiplier (currently 2×SE), try {1.5×, 3×, 5×}
   - S2: number of weak metrics, signal strength, Custom sigmoid params
   - S4: which/how many components have missing uncertainty
   - S7: boundary SE values, precision thresholds
4. **Implement perturbation runner** — new prototype script (e.g., `perturbation_test.py`) that generates fixture variants and runs the hybrid against each
5. **Run and analyze** — identify which scenarios break, at what perturbation level, and why
6. **Write FINDINGS.md Session 4 log entry** — include robustness map showing pass/fail regions per scenario

## Session Artifacts

- Prompt: `.claude/prompts/prompt_012_2026-02-21_hybrid-aggregation-candidate.md`
- Handoff: `.claude/handoffs/handoff_014_2026-02-21_plan-session3-hybrid-aggregation.md`
- Beads issues: `athena-bop` (S4, ready), `athena-e2a` (S5, blocked by bop), `athena-6ax` (S6, blocked by both)
- Results: `results.json`, `results.md` (regenerated S3 with 4×7 matrix)

## Documentation Updated

No documentation updates — all project docs were current.
