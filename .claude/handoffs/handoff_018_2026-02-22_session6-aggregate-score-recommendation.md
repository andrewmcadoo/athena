# Session Handoff: WDK#41 Session 6 — AggregateScore Recommendation

> Generated: 2026-02-22 | Handoff #18 | Previous: handoff_017_2026-02-22_plan-athena-e2a-stretch.md

---

## Continuation Directive

/prompt : 
Write the architecture-ready AggregateScore recommendation (athena-6ax). This is NOT another experiment — it is the decision write-up that turns all Session 1-5 + 4.2/4.3 evidence into one stable architecture contract. The crash tests are done; now write the flight manual.

Four deliverables: (1) lock the recommended AggregateScore spec with fixed parameters, (2) tie each design choice to evidence artifacts, (3) state operating boundaries (accepted limitations vs resolved risks), (4) define revisit triggers for when the recommendation must be reopened.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial reward aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` scalar (`AggregateScore`).

**Goal:** Produce the canonical AggregateScore specification document that downstream ATHENA components can build against, backed by 7 sessions of prototype evidence.

**Constraints:** Stdlib-only Python prototypes, frozen dataclasses. All prototype work in `research/adversarial-reward/prototypes/aggregation-candidates/`. See CLAUDE.md for governance. This session produces a specification document, not code.

## Key Decisions & Rationale

1. **Hybrid architecture: HTG gating + Fisher product (n_terms=1)**
   - **Rationale:** Only candidate to achieve 7/7 baseline pass (Session 3). IVW-CDF, HTG-Max, and Fisher-UP each failed at least one scenario.
   - **Evidence:** `perturbation_summary.md` (robustness sweep), FINDINGS.md Sessions 1-3.

2. **Log-scaled BF normalization (c=0.083647, bf_max_target=10000)**
   - **Rationale:** Original `bf/(bf+1)` hit ceiling at BF=110 causing S5 failures. Log-scaled extends to BF=9999 while maintaining 7/7 baseline. Recovers all 5 failing S6 cells as side benefit.
   - **Alternatives rejected:** Power-law (slightly lower ceiling granularity), exp-decay (failed pre-filter), keeping original norm (unacceptable operating range).
   - **Evidence:** `ceiling_analysis.md`, `ceiling_analysis.json`.

3. **Guardrail: x0 >= 0 for custom sigmoid parameters**
   - **Rationale:** S2 fails at x0=-0.2 with k>=2.0. Regime validity analysis classified x0=-0.2 as out-of-range for realistic DSL workflows. x0>=0 provides clean margin above the failure locus.
   - **Evidence:** `guardrail_spec.md` (GR-S2-CUSTOM-SIGMOID-X0-NONNEG), `regime_validity.md`.

4. **Pattern B under-response classified as accepted limitation**
   - **Rationale:** 50x isolated single-metric jump is not representative of physically valid behavior in OpenMM/GROMACS/CESM/VASP. Step_ratio=1.029 (threshold >3.0) is a stress-test extreme, not a realistic operating scenario.
   - **Evidence:** `regime_validity.md` (Domain Check table), `stretch_summary.md`.

5. **S1 SE 5x/10x fragility classified as accepted limitation**
   - **Rationale:** Realistic SE multiplier band is [0.5, 3.0]. Failures at 5x/10x map to low-quality/invalid runs, not normal DSL operation.
   - **Evidence:** `regime_validity.md` (parameter range table).

## Current State

### Completed
- **Sessions 1-3:** Candidate development → hybrid 7/7 pass
- **Session 4:** Perturbation robustness (70 runs, 7 axes). Found S5 ceiling at BF=110, S2 sensitivity at negative x0.
- **Session 4.1:** Ceiling analysis. Log-scaled BF normalization adopted (bf_max=10000). S5 resolved, S6 recovered.
- **Session 5:** Stretch tests (calibration + correlation). Pattern A/C pass, Pattern B fail (non-responsive but smooth). Correlation robustness pass (floor-saturation resolved, inflation=1.0035 at rho=0.5).
- **Session 4.2/4.3:** Regime validity + guardrail. All remaining failures classified as out-of-range or resolved. x0>=0 guardrail codified. athena-17c and athena-zvg closed.
- **All prerequisites for athena-6ax are now closed.**

### In Progress
- Nothing. Ready to write the recommendation.

### Blocked / Open Questions
- **athena-6ax** is open and unblocked — this is the next session's deliverable.
- **Open thread from Session 5:** Whether targeted hybrid adjustments could lift Pattern B step_ratio above 3.0 — explicitly NOT in scope for this session. Document as future work if needed.

## Key Code Context

**The hybrid function** (canonical form from `stretch_test.py`):
```python
hybrid_fn = lambda comps: aggregate_hybrid_patched(
    comps, BASELINE_HYBRID_CONFIG,
    partial(bf_norm_log_scaled, c=0.083647)
)
```

**BF normalization** (`ceiling_analysis.py`):
```python
def bf_norm_log_scaled(bf: float, c: float) -> float:
    log_term = math.log1p(bf)
    return log_term / (log_term + c)
# c = 0.083647 calibrated for bf_max_target=10000
```

**Guardrail constraint** (`guardrail_spec.md`):
```
GR-S2-CUSTOM-SIGMOID-X0-NONNEG
For all entries in NormalizationConfig.custom_sigmoids: x0 >= 0
Enforcement: reject with error at config construction time, no silent clamping.
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `.../aggregation-candidates/candidates.py` | 4 candidates incl. `aggregate_hybrid` + `HybridConfig` | Created (S3) |
| `.../aggregation-candidates/evaluate.py` | 4x7 harness — S5 0.991 ceiling, S6 dual criterion | Created (S3) |
| `.../aggregation-candidates/normalization.py` | Normalization + gating: `normalize_component`, `gate_precision` | Created (S2) |
| `.../aggregation-candidates/scenarios.py` | 7 scenario fixtures + builders | Created (S1) |
| `.../aggregation-candidates/models.py` | Dataclass contracts (`MetricComponent`, `AggregateResult`, etc.) | Created (S1) |
| `.../aggregation-candidates/perturbation_test.py` | Session 4 perturbation sweep (70 runs, 7 axes) | Created (S4) |
| `.../aggregation-candidates/perturbation_summary.md` | Pass rates, margins, tipping points | Generated (S4) |
| `.../aggregation-candidates/ceiling_analysis.py` | BF ceiling analysis (4-phase) | Created (S4.1) |
| `.../aggregation-candidates/ceiling_analysis.md` | Decision summary: log-scaled BF normalization | Generated (S4.1) |
| `.../aggregation-candidates/stretch_test.py` | Calibration + correlation robustness (419 lines) | Created (S5) |
| `.../aggregation-candidates/stretch_summary.md` | Calibration results + correlation pass | Generated (S5) |
| `.../aggregation-candidates/regime_validity.md` | In-range vs out-of-range failure classification | Created (S4.2/4.3) |
| `.../aggregation-candidates/guardrail_spec.md` | x0>=0 guardrail specification | Created (S4.2/4.3) |
| `research/adversarial-reward/FINDINGS.md` | Master research log (7 entries: S1-S5 + S4.1 + S4.2/4.3) | Updated throughout |

## Loop State

- **Iteration:** 7 complete (S1→S2→S3→S4→S4.1→S5→S4.2/4.3), writing Session 6 recommendation
- **Last prompt to Codex:** Prompt #16 (regime validity + guardrail spec)
- **Codex result:** regime_validity.md, regime_validity.json, guardrail_spec.md all delivered. athena-17c and athena-zvg closed. FINDINGS.md updated.
- **Review findings:** Verified all deliverables. Regime validity classifications are well-evidenced with confidence indicators. Guardrail spec has clean structure (constraint, rationale, scope, enforcement).

## Next Steps

1. **Read athena-6ax bead** — `bd show athena-6ax` for full description and acceptance criteria
2. **Read all evidence artifacts** — `perturbation_summary.md`, `ceiling_analysis.md`, `stretch_summary.md`, `regime_validity.md`, `guardrail_spec.md`, and FINDINGS.md accumulated findings
3. **Write AggregateScore recommendation document** — create `aggregate_score_recommendation.md` in the aggregation-candidates directory with:
   - Canonical spec: hybrid form, fixed parameters, normalization family, guardrail
   - Evidence map: each design choice → artifact reference
   - Operating boundaries: accepted limitations (Pattern B, S1 SE) vs resolved risks (S5 BF, S6 compression)
   - Revisit triggers: conditions that reopen this recommendation
4. **Create machine-readable spec** — `aggregate_score_recommendation.json` with the locked parameters
5. **Update FINDINGS.md** — Session 6 investigation log entry + accumulated findings update
6. **Close athena-6ax** — `bd close athena-6ax` + session protocol (sync, commit, push)

## Beads (WDK#41 Dependency Graph)

```
✓ athena-rfp (CLOSED)  ──▶  ✓ athena-e2a (CLOSED)  ──blocks──▶  ○ athena-6ax (OPEN)
  S5 ceiling                   S5 stretch                          Recommendation

✓ athena-17c (CLOSED)  ──blocks──────────────────────────────────▶  athena-6ax
  Regime validity

✓ athena-zvg (CLOSED)  ──blocks──────────────────────────────────▶  athena-6ax
  S2 guardrail

All prerequisites CLOSED. athena-6ax is READY.
```

## Session Artifacts

- Prompts: `.claude/prompts/prompt_015_2026-02-22_session5-hybrid-stretch.md`, `.claude/prompts/prompt_016_2026-02-22_regime-validity-guardrail-spec.md`
- Previous handoff: `.claude/handoffs/handoff_017_2026-02-22_plan-athena-e2a-stretch.md`
- Key outputs verified: `stretch_test.py` (419 lines), `stretch_summary.md`, `regime_validity.md`, `guardrail_spec.md`
- Beads: athena-e2a (CLOSED), athena-17c (CLOSED), athena-zvg (CLOSED), **athena-6ax (OPEN, UNBLOCKED)**

## Documentation Updated

No documentation updates — all project docs were current.
