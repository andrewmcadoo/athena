# Session Handoff: WDK#41 S5 — Plan Hybrid Stretch Analyses

> Generated: 2026-02-22 | Handoff #17 | Previous: handoff_016_2026-02-22_plan-s41-ceiling-analysis.md

---

## Continuation Directive

Plan `athena-e2a`: Session 5 hybrid stretch analyses (calibration + correlation). The S5 ceiling is now resolved — log-scaled BF normalization extends operating range from BF=110 to BF=9999 while maintaining 7/7 baseline. Design the implementation plan for stretch tests that must run against the POST-ceiling-decision hybrid config. Two analyses: (1) 50-cycle calibration simulation (patterns A/B/C), (2) correlation robustness at rho {0.0, 0.3, 0.5, 0.7, 0.9}. Re-verify S5 pass with the new normalization before running stretch tests.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial reward aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` scalar.

**Goal:** Run stretch analyses (calibration simulation + correlation robustness) against the hybrid candidate with the updated BF normalization, to determine whether it handles calibration instability and inter-metric correlation better than single-family candidates from Session 2.

**Constraints:** Stdlib-only Python prototypes, frozen dataclasses. All work in `research/adversarial-reward/prototypes/aggregation-candidates/`. See CLAUDE.md for full rules.

## Key Decisions & Rationale

1. **S5 ceiling resolved via log-scaled BF normalization (bf_max=10000)**
   - **Rationale:** `bf/(bf+1)` hits 0.991 at BF=110. `log(1+bf)/(log(1+bf)+c)` with calibrated `c=0.083647` extends ceiling to BF=9999. Maintains 7/7 baseline, passes S5 sweep through BF=1000.
   - **Alternatives evaluated:** Power-law (also 7/7, slightly lower ceiling granularity), exp-decay (2/15 failed pre-filter at high bf_max targets), keeping current norm (unacceptable operating range for DSL environments).

2. **S6 failures are independent of S5 — driven by dominant_share < 0.35**
   - **Rationale:** All 5 failing S6 cells have `recon_error < 1e-8` and `dominant_share < 0.35`. It's evidence dilution, not reconstruction error. The log-scaled normalization recovers all 5 S6 failures as a side benefit.
   - **Implication:** S6 does not require separate fixing — the ceiling fix resolves both.

3. **Stretch tests must use POST-ceiling hybrid config**
   - **Rationale:** Per athena-e2a bead description: "Run stretch tests against the POST-ceiling-decision hybrid config, not the current one." The normalization layer change shifts baselines.

4. **Hybrid architecture: HTG gating + Fisher product (n_terms=1)** — unchanged from Sessions 1-4. HTG confidence gating via `log1p(n/SE²)` → sigmoid → confidence. Fisher product with `n_terms=1` compounds concordant evidence.

## Current State

### Completed
- **Session 1-3:** Candidate development through hybrid 7/7 pass
- **Session 4:** Perturbation robustness sweep (70 runs, 7 axes). Found S5 ceiling at BF=110, S6 failures at high d_mid+bf_strong, S2 sensitivity at negative x0.
- **Session 4.1 (this session):** Ceiling analysis resolved. Key results:
  - Sanity gate: max diff = 0.0 across all 7 scenarios (patched hybrid matches baseline exactly)
  - S6 decomposition: all 5 failures confirmed dominant_share-driven
  - Best candidate: `log_scaled_bfmax_10000` (bf_ceiling=9999, 7/7 baseline, S5 sweep pass through BF=1000)
  - S6 bonus: highest-ceiling candidates recover all 5 previously-failing S6 cells
  - Recommendation: GO for athena-e2a adoption

### In Progress
- Nothing. Ready to plan athena-e2a.

### Blocked / Open Questions
- **How to integrate the new normalization into the hybrid?** Options: (a) modify `normalize_component` to accept a BF norm function, (b) build a new config field, (c) only use the patched hybrid in stretch tests and defer integration to athena-6ax.
- **Session 2 stretch test baseline data** — need to read the Session 2 FINDINGS.md entry and any existing calibration/correlation code to understand what was previously tested and what pass criteria were used.
- **athena-17c (regime validity)** and **athena-zvg (S2 guardrail)** are parallel open beads but do NOT block athena-e2a.

## Key Code Context

**Ceiling analysis result — the new BF normalization** (`ceiling_analysis.py`):
```python
def bf_norm_log_scaled(bf: float, c: float) -> float:
    log_term = math.log1p(bf)
    return log_term / (log_term + c)
# Calibrated c = 0.083647 for bf_max_target=10000
```

**Patching approach** (`ceiling_analysis.py`) — how alt BF normalization plugs in:
```python
def aggregate_hybrid_patched(components, config, bf_norm_fn):
    # Copy of aggregate_hybrid with normalize_component_with_alt_bf
    # Only BF components use bf_norm_fn; all others delegate to normalize_component
```

**S5 evaluation criterion** (`evaluate.py` ~line 183):
```python
in_range = all(0.3 <= s <= 0.991 for s in component_scores)
```

**S6 evaluation criterion** (`evaluate.py` ~line 210):
```python
passed = abs(recon - calibration.aggregate_score) <= 1e-8 and dominant_share >= 0.35
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `.../aggregation-candidates/ceiling_analysis.py` | 4-phase ceiling analysis (793 lines) | Created (S4.1) |
| `.../aggregation-candidates/ceiling_analysis.json` | Machine-readable phase outputs | Generated (S4.1) |
| `.../aggregation-candidates/ceiling_analysis.md` | Decision summary with tables + recommendation | Generated (S4.1) |
| `.../aggregation-candidates/perturbation_test.py` | Session 4 perturbation sweep runner | Created (S4) |
| `.../aggregation-candidates/perturbation_results.json` | Full perturbation output | Generated (S4) |
| `.../aggregation-candidates/perturbation_summary.md` | Pass rates, margins, tipping points | Generated (S4) |
| `.../aggregation-candidates/candidates.py` | 4 candidates incl. `aggregate_hybrid` + `HybridConfig` | Modified (S3) |
| `.../aggregation-candidates/evaluate.py` | 4×7 harness — S5 0.991 ceiling, S6 dual criterion | Modified (S3) |
| `.../aggregation-candidates/normalization.py` | Normalization + gating: `normalize_component`, `gate_precision` | Modified (S2) |
| `.../aggregation-candidates/scenarios.py` | 7 scenario fixtures + builders | Unchanged |
| `.../aggregation-candidates/models.py` | Dataclass contracts (`MetricComponent`, `AggregateResult`, etc.) | Unchanged |
| `research/adversarial-reward/FINDINGS.md` | Master research log (5 entries: S1-S4 + S4.1) | Modified (S4.1) |

## Loop State

- **Iteration:** 5 complete (S1→S2→S3→S4→S4.1), planning S5
- **Last prompt to Codex:** Session 4.1 RISEN prompt (`.claude/prompts/prompt_014_2026-02-22_s5-ceiling-analysis.md`)
- **Codex result:** Ceiling analysis ran cleanly. Sanity gate passed (max diff 0.0). 13/15 candidates achieved 7/7. Log-scaled with bf_max=10000 selected as best. All outputs verified.
- **Review findings (this session):** Verified script runs, JSON valid, markdown complete with 5 sections, FINDINGS.md entry follows append-only protocol. No issues found.

## Next Steps

1. **Read athena-e2a bead** — `bd show athena-e2a` for full description
2. **Read Session 2 FINDINGS.md entry** — understand what calibration patterns A/B/C and correlation robustness tests were run against single-family candidates, and what pass criteria were used
3. **Read `ceiling_analysis.md`** — specifically the S6 side-benefit table and recommendation section
4. **Design the hybrid config for stretch tests** — decide whether to (a) modify `normalize_component` to use log-scaled BF norm, (b) use the `aggregate_hybrid_patched` approach from ceiling analysis, or (c) create a new hybrid variant
5. **Plan calibration simulation** — 50-cycle deterministic simulation against hybrid with patterns A/B/C. Define what "handles pattern B instability better" means quantitatively
6. **Plan correlation robustness** — rho sweep at {0.0, 0.3, 0.5, 0.7, 0.9}. Compare against Session 2 Fisher floor-saturation findings
7. **Verify S5 pass with new normalization** — must confirm before running stretch tests (per bead description)
8. **Produce RISEN prompt** for Codex implementation

## Beads (WDK#41 Dependency Graph)

```
✓ athena-rfp (P1)  ──(CLOSED)──▶  ○ athena-e2a (P2)  ──blocks──▶  ○ athena-6ax (P2)
  S5 ceiling (DONE)                  S5 stretch (READY)               Recommendation

○ athena-17c (P2)  ──blocks─────────────────────────────────────────▶  athena-6ax
  Regime validity (READY)

○ athena-zvg (P3)  ──blocks─────────────────────────────────────────▶  athena-6ax
  S2 guardrail (READY)
```

## Session Artifacts

- Prompt: `.claude/prompts/prompt_014_2026-02-22_s5-ceiling-analysis.md`
- Previous handoff: `.claude/handoffs/handoff_016_2026-02-22_plan-s41-ceiling-analysis.md`
- Ceiling outputs: `ceiling_analysis.py`, `ceiling_analysis.json`, `ceiling_analysis.md`
- Beads: athena-rfp (CLOSED), athena-e2a (READY), athena-17c (READY), athena-zvg (READY), athena-6ax (BLOCKED by e2a+17c+zvg)

## Documentation Updated

No documentation updates — all project docs were current.
