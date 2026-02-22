# Session Handoff: WDK#41 Session 3 — Hybrid Aggregation Design

> Generated: 2026-02-21 | Handoff #14 | Previous: handoff_013_2026-02-21_wdk41-reward-aggregation-experiment.md

---

## Continuation Directive

Plan Session 3 for the adversarial reward aggregation research (WDK#41). Design a **hybrid aggregation candidate** that combines HTG-Max's confidence gating (front end) with Fisher/IVW compounding (back end). Session 2 proved no single-family candidate can pass 7/7 adversarial scenarios — the failures are structural, not parametric. The hybrid must unify noise filtering and signal compounding in one architecture while preserving decomposable weights for calibration. Beads issue: `athena-btk`.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. Aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` reward scalar for the adversarial experiment designer.

**Goal:** Prototype a hybrid candidate targeting 7/7 adversarial scenarios, or 6/7 with documented S2 sensitivity analysis.

**Constraints:** Stdlib only, frozen dataclasses, throwaway prototype code. All work in `research/adversarial-reward/prototypes/aggregation-candidates/`. See `CLAUDE.md` for full project constraints.

## Key Decisions & Rationale

1. **Single-family tuning is exhausted** — Session 2 swept ~723 configs across 4 structural fixes (SE-dampening, multiplicity bonus, soft aggregation, SE-aware reliability). Best results: HTG 5/7, Fisher 5/7 (isolation), IVW 2/7. No 7/7 found.
   - **IVW:** Weighted average can never exceed max input → structurally fails S2 (compounding)
   - **HTG:** Per-kind max discards cross-metric concordance → fails S2, decomposition breaks → fails S5/S6
   - **Fisher:** No noise-filtering mechanism → fails S1 (Noisy TV)

2. **SE-dampening in normalization conflicts with per-candidate logic** — Normalization-level dampening (`se_dampen_enabled=True`, winner N061: `k=8.0, x0=1.0`) fixed S1/S7 for Fisher but degraded IVW on S4/S5/S6. Fisher isolation (SE-reliability ON, SE-dampening OFF) reached 5/7 vs 4/7 in main sweep.
   - **Implication:** Noise filtering belongs in per-metric gating, not global normalization.

3. **HTG gating + Fisher compounding = recommended hybrid**
   - HTG gates well (passes S1, S3, S4, S7) but can't compound
   - Fisher compounds well (passes S3, S5, S6) but can't filter noise
   - **Open question:** Does gated-Fisher produce well-defined `sum(w_i * score_i) = aggregate` decomposition for S6?

4. **S2 criterion (1.5x) may need relaxation** — No config achieved 6/7-with-only-S2-fail. The 1.5x multiplier is aggressive. No feasibility frontier could be established.

5. **Calibration behavior is candidate-specific** — HTG tracks gradual convergence + oscillating uncertainty well (passes A, C). Fisher tracks convergence + uncertainty but has smoothness issues on sudden regime change (B: max_delta=0.9996). IVW failed all three patterns.

## Current State

### Completed
- **Session 1:** 3 candidates, 3×7 stress test → 5/7, 5/7, 3/7. Commit `33dcd08`.
- **Session 2:** 4 structural fixes, two-stage sweep (723 configs), calibration sim, Fisher correlation robustness. All 9 plan steps executed. Commit `2e9da3d`.
- **FINDINGS.md:** Two Session log entries, Prototype Index (8 entries), Accumulated Findings updated.

### In Progress
- Nothing. Session 2 is complete.

### Blocked / Open Questions
- **Decomposition math:** HTG uses `weight = winner_gate * confidence`. Fisher uses `weight = log_evidence * scale`. Hybrid needs a unified weight scheme where `sum(w_i * score_i)` reconstructs the aggregate within 1e-8 (S6 criterion).
- **S2 correlation regime:** Session 2 correlation test had floor-saturated aggregates (~1e-12), making inflation ratios uninformative. Needs a mid-range test regime.

## Key Code Context

**HTG gating logic** (`candidates.py:157-164`) — this is the front end for the hybrid:
```python
precision = gate_precision(component, cfg.eps)  # log1p(n/se²)
confidence = sigmoid(precision, cfg.alpha, cfg.tau) if precision else cfg.c_floor
gated_score = score * confidence
```

**Fisher combination logic** (`candidates.py:280-298`) — this is the back end:
```python
p_value = max(p_eps, 1.0 - score)
reliability = min(1.0, n_i / n_ref)
p_adj = p_value ** reliability
log_evidence = -2.0 * log(p_adj)
# chi_square_cdf(sum(log_evidence), n_terms=len(staged))
```

**Key hybrid insight:** In the hybrid, `score` entering Fisher should be `gated_score` (score × confidence), so the p-value reflects both divergence magnitude AND measurement quality.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/adversarial-reward/prototypes/aggregation-candidates/candidates.py` | 3 candidates + configs | Modified (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/normalization.py` | Normalization + SE-dampening | Modified (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/evaluate.py` | 3×7 evaluation harness | Unchanged |
| `research/adversarial-reward/prototypes/aggregation-candidates/scenarios.py` | 7 scenario fixtures | Unchanged |
| `research/adversarial-reward/prototypes/aggregation-candidates/models.py` | Dataclass contracts | Unchanged |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep.py` | Two-stage parameter sweep | Created (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep_summary.md` | Best configs per candidate | Created (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_sim.py` | 50-cycle calibration | Created (S2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/correlation_test.py` | Fisher correlation robustness | Created (S2) |
| `research/adversarial-reward/FINDINGS.md` | Master research log | Modified (S2) |

## Loop State

- **Iteration:** 2 complete (Session 1 → Session 2), planning Session 3
- **Last prompt to Codex:** Session 2 RISEN prompt (`.claude/prompts/prompt_011_2026-02-21_session2-aggregation-fixes-sweep.md`)
- **Codex result:** All 9 steps completed. One mid-run fix needed (chi_square_cdf overflow → recurrence series + 1000-term cap).
- **Review findings:** All verification criteria passed. Key result: no 7/7, hybrid needed.

## Next Steps

1. **Read FINDINGS.md Session 2 log** (lines 33-93) — understand what was tried and the open threads
2. **Read `sweep_summary.md`** — exact best configs and failure scenarios per candidate
3. **Design hybrid architecture** — bolt HTG confidence gating onto Fisher p-value combination
4. **Resolve decomposition question** — can `sum(w_i * score_i)` reconstruct a chi-squared CDF aggregate? If not, design an alternative decomposition scheme
5. **Implement `aggregate_hybrid()` in `candidates.py`** — new `HybridConfig` dataclass, add to `get_candidate_registry`
6. **Run against all 7 scenarios**, then sweep hybrid-specific params if needed
7. **Write FINDINGS.md Session 3 log entry**

## Session Artifacts

- Prompt: `.claude/prompts/prompt_011_2026-02-21_session2-aggregation-fixes-sweep.md`
- Beads issue: `athena-btk` (Session 3 hybrid design, P2, open)
- Sweep outputs: `sweep_results.json`, `sweep_summary.md`
- Calibration outputs: `calibration_results.json`, `calibration_summary.md`
- Correlation output: `correlation_results.json`
- Commit: `2e9da3d` on `master`

## Documentation Updated

No documentation updates — all project docs were current.
