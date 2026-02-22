# Session Handoff: WDK#41 S4.1 — S5 Ceiling Analysis Plan

> Generated: 2026-02-22 | Handoff #16 | Previous: handoff_015_2026-02-22_plan-session4-fixture-robustness.md

---

## Continuation Directive

Plan `athena-rfp`: S5 ceiling analysis — operating regime bounds and architectural options. The 0.991 upper bound on normalized component scores is the tightest constraint in the hybrid system (margin +0.000009 at BF=110). Determine whether this is a real operating constraint or an academic edge case, decompose whether S6 failures share the same root cause, and evaluate architectural options (raise ceiling / clamp inputs / accept constraint). Produce an implementation plan for the next coding session.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial reward aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` scalar.

**Goal:** Resolve the S5 ceiling question so the hybrid architecture can be recommended (or modified) with known operating bounds. This is the P1 bead on the critical path to the final AggregateScore type recommendation (athena-6ax).

**Constraints:** Stdlib-only Python prototypes, frozen dataclasses. All work in `research/adversarial-reward/prototypes/aggregation-candidates/`. See CLAUDE.md for full rules.

## Key Decisions & Rationale

1. **Hybrid architecture: HTG gating + Fisher product (n_terms=1)** — HTG confidence gating filters noise via `log1p(n/SE²)` precision → sigmoid → confidence. Fisher product compounds concordant evidence via `1 - Π(p_i)`. Using `n_terms=1` avoids df=2N penalty.
   - **Alternatives rejected:** n_terms=N (too much evidence required for weak signals); SE-dampening in normalization (conflates magnitude with units).

2. **S5 ceiling is architectural, not parametric** — Critical analysis of Session 4 results showed the 0.991 failure is a deterministic consequence of `normalized_score = BF / (BF + 1)` hitting a hard-coded bound. It's not fixable by parameter tuning. This reframing drove the bead reorganization.
   - **Alternatives rejected:** "Widen safety margins" framing (treats a design boundary as a tuning problem).

3. **S6 failures may share S5 root cause** — S6 `bf_strong=1000` pushes BF component scores to ~0.999, well past the 0.991 ceiling. Not yet confirmed whether S6 failures are driven by dominant_share or by ceiling overshoot. Decomposition needed.

4. **S2 sigmoid guardrail: x0 >= 0** — All S2 failures confined to x0=-0.2 with k>=2.0. Mechanism understood (negative midpoint compresses output range). One-line spec addition captured as separate bead (athena-zvg, P3).

5. **Regime validity is parallel work** — Whether BF>110 or SE mult>5x occurs in real DSL outputs is a separate question (athena-17c) that can run alongside ceiling analysis.

## Current State

### Completed
- **Session 1:** 3 candidates, 3×7 stress test → IVW 5/7, HTG 5/7, Fisher 3/7
- **Session 2:** 4 structural fixes, two-stage sweep (723 configs), calibration sim, correlation robustness. No 7/7 found.
- **Session 3:** Hybrid candidate implemented. 7/7 on all scenarios. S2 ratio=1.609 (+7.28% margin), S4 delta=0.072, S6 error=1.1e-16.
- **Session 4:** Perturbation robustness sweep (70 runs across 7 axes). Key findings:
  - S2 custom sigmoid: 20/24 pass. Tipping at x0=-0.2, k 1.5→2.0.
  - S5 BayesFactor: 4/9 pass. Tipping at BF 110→120 (margin collapses from +0.000009 to -0.000736).
  - S6 joint compress: 11/16 pass. Failures in high d_mid + high bf_strong corners.
  - S1 SE mult: 3/5 pass. Surprise failures at 5x and 10x (flagged for regime validity check).
  - Fully robust: S2 non-custom SE (5/5), S4 missing count (4/4), S7 boundary SE (7/7).
- **Critical analysis:** Identified three distinct failure modes — S5 hard ceiling (architectural), S2 sigmoid sensitivity (parametric, bounded), S6 possibly derivative of S5.
- **Bead reorganization:** Created athena-rfp (P1, ceiling), athena-17c (P2, regime validity), athena-zvg (P3, guardrail). Rewired dependencies.

### In Progress
- Nothing. Ready to plan athena-rfp.

### Blocked / Open Questions
- **Is BF > 110 realistic?** If DSL environments commonly produce BF values this high, the ceiling must change. If not, document as accepted constraint. (athena-17c runs parallel)
- **Are S6 failures independent of S5?** Need to decompose S6 failure into its two sub-criteria (dominant_share vs reconstruction error) to confirm root cause.
- **S1 at extreme SE multipliers:** Is 5x/10x SE realistic? Deferred to regime validity (athena-17c).

## Key Code Context

**S5 evaluation criterion** (`evaluate.py` line ~183):
```python
in_range = all(0.3 <= s <= 0.991 for s in component_scores)
```
The 0.991 ceiling is hard-coded. BF normalization: `score = BF / (BF + 1)`. At BF=110: score=0.990991 (margin +0.000009). At BF=120: score=0.991736 (FAIL).

**S6 evaluation criterion** (`evaluate.py` line ~210) — two conjuncts:
```python
passed = abs(recon - calibration.aggregate_score) <= 1e-8 and dominant_share >= 0.35
```
S6 failures could be driven by either sub-criterion. Not yet decomposed.

**Hybrid pipeline summary** (`candidates.py:383-474`):
normalize → gate_precision (log1p(n/SE²)) → sigmoid → confidence → gated_score → p-value → log_evidence → chi_square_cdf (n_terms=1) → aggregate

## Files Map

| Path | Role | Status |
|------|------|--------|
| `.../aggregation-candidates/perturbation_test.py` | Session 4 perturbation sweep runner | Created (S4) |
| `.../aggregation-candidates/perturbation_results.json` | Full structured perturbation output | Generated (S4) |
| `.../aggregation-candidates/perturbation_summary.md` | Pass rates, margin grids, tipping points | Generated (S4) |
| `.../aggregation-candidates/candidates.py` | 4 candidates + HybridConfig | Modified (S3) |
| `.../aggregation-candidates/evaluate.py` | 4×7 evaluation harness — **contains the 0.991 ceiling** | Modified (S3) |
| `.../aggregation-candidates/normalization.py` | Normalization + gating helpers | Modified (S2) |
| `.../aggregation-candidates/scenarios.py` | 7 fixed scenario fixtures + builders | Unchanged |
| `.../aggregation-candidates/models.py` | Dataclass contracts | Unchanged |
| `research/adversarial-reward/FINDINGS.md` | Master research log (4 session entries) | Modified (S4) |

## Loop State

- **Iteration:** 4 complete (S1→S2→S3→S4), planning S4.1 (ceiling analysis)
- **Last prompt to Codex:** Session 4 RISEN prompt (`.claude/prompts/prompt_013_2026-02-22_hybrid-robustness-perturbation.md`)
- **Codex result:** 70-run perturbation sweep completed cleanly. Baselines verified. Tipping points identified. FINDINGS.md updated. Bead closed.
- **Review findings (this session):** Critical analysis reframed S5 as architectural ceiling (not parameter sensitivity), identified S6 may share root cause, reorganized beads to put ceiling analysis on critical path.

## Beads (WDK#41 Dependency Graph)

```
athena-rfp (P1)  ──blocks──▶  athena-e2a (P2)  ──blocks──▶  athena-6ax (P2)
  S5 ceiling                    S5 stretch                    Recommendation
  [OPEN, READY]                 [OPEN, BLOCKED]               [OPEN, BLOCKED]

athena-17c (P2)  ──blocks──────────────────────────────────▶  athena-6ax
  Regime validity
  [OPEN, READY]

athena-zvg (P3)  ──blocks──────────────────────────────────▶  athena-6ax
  S2 guardrail
  [OPEN, READY]
```

## Next Steps

1. **Read `athena-rfp` bead description** — `bd show athena-rfp` (may SIGSEGV; bead content is in this handoff's Task Definition if so)
2. **Read `perturbation_summary.md`** — S5 BayesFactor sweep table and S6 joint compression tipping points
3. **Read `evaluate.py`** — understand the 0.991 ceiling implementation and S6 two-criterion logic
4. **Decompose S6 failures** — determine which sub-criterion (dominant_share vs reconstruction) is binding in each failing cell
5. **Research BF regime** — what BayesFactor magnitudes do OpenMM/GROMACS/CESM/VASP produce? (may need ARCHITECTURE.md DSL section)
6. **Evaluate architectural options:** (a) raise ceiling with impact analysis on all 7 scenarios, (b) clamp BF before normalization, (c) accept constraint with documented bounds
7. **Produce implementation plan** for the coding session that resolves athena-rfp

## Session Artifacts

- Prompt: `.claude/prompts/prompt_013_2026-02-22_hybrid-robustness-perturbation.md`
- Previous handoff: `.claude/handoffs/handoff_015_2026-02-22_plan-session4-fixture-robustness.md`
- Perturbation results: `research/adversarial-reward/prototypes/aggregation-candidates/perturbation_results.json`
- Perturbation summary: `research/adversarial-reward/prototypes/aggregation-candidates/perturbation_summary.md`
- Beads: athena-rfp (S4.1, P1, ready), athena-17c (S4.2, P2, ready), athena-zvg (S4.3, P3, ready), athena-e2a (S5, blocked by rfp), athena-6ax (S6, blocked by e2a+17c+zvg)

## Documentation Updated

No documentation updates — all project docs were current.
