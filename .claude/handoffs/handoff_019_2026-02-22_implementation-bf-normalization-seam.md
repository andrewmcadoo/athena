# Session Handoff: Implementation Start — BF Normalization Seam (athena-4xm)

> Generated: 2026-02-23 | Handoff #19 | Previous: handoff_018_2026-02-22_session6-aggregate-score-recommendation.md

---

## Continuation Directive

Build athena-4xm (BF normalization seam) in the prototype code. This is the first implementation session — not more analysis.

Session 6 picked the exact engine. Session 7 wrote the spec sheet. This session installs the engine mount so the locked engine can be plugged in safely.

Concretely:
1. Add a first-class BF normalization hook (explicit extension point) to `normalization.py`, with locked default `bf_norm_log_scaled(bf, c)` and `c=0.083647`.
2. Keep all non-BF normalization behavior unchanged.
3. Prove default behavior matches the locked contract outputs.

After athena-4xm, do the other contract-enforcement beads in order:
1. athena-8b9 (x0 >= 0 config-time guardrail)
2. athena-fgo (decomposition invariant <= 1e-8)
3. athena-3lu (full acceptance suite — depends on all three)

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial reward aggregation function collapses `Vec<MetricComponent>` into a bounded `[0,1]` scalar.

**Goal:** Add a configurable BF normalization seam to `normalization.py` so the locked log-scaled function is the default but future normalization families can be substituted without breaking the aggregation contract.

**Why this matters:**
- Without this seam, BF logic stays hard-coded at line 136-138 of `normalization.py` and future changes risk breaking the locked AggregateScore contract.
- athena-3lu acceptance tests depend on this seam being in place.

**Constraints:** Stdlib-only Python, frozen dataclasses. All code in `research/adversarial-reward/prototypes/aggregation-candidates/`. See CLAUDE.md for governance.

## Key Decisions & Rationale

1. **Log-scaled BF normalization is the locked default** — `bf_norm_log_scaled(bf, c) = log1p(bf) / (log1p(bf) + c)` with `c=0.083647` (calibrated for `bf_max_target=10000`). This replaced the original `bf/(bf+1)` which hit a ceiling at BF=110.
   - Evidence: `ceiling_analysis.md`, `aggregate_score_recommendation.md` Section 1.2.

2. **The seam must be a first-class configurable hook** — Not a hard-coded branch. The recommendation explicitly calls for this (Section 6 note 1): "Production implementation should expose BF normalization as a first-class configurable hook."
   - This means `NormalizationConfig` needs a BF normalization callable/function field.

3. **Non-BF branches must remain unchanged** — ZScore, KLDivergence, AbsoluteDifference, EffectSize, Custom normalization is untouched.

4. **n_terms=1 is intentional** — The Fisher combination uses df=2 (single-term chi-square), not df=2N. Do not change this.

## Current State

### Completed (Sessions 1-7)
- Sessions 1-5 + 4.1 + 4.2/4.3: Research — candidate evaluation, robustness sweeps, ceiling analysis, stretch tests, regime validity, guardrail spec.
- Session 6: Locked AggregateScore recommendation (`aggregate_score_recommendation.md` + `.json`). Bead athena-6ax CLOSED.
- Session 7: Architecture integration — ARCHITECTURE.md updated (Section 4.4.1), ADR 002 created, acceptance test spec + monitoring triggers spec created, 5 implementation beads created.

### In Progress
- **athena-4xm** (BF normalization seam) — OPEN, unblocked, ready to implement.

### Blocked
- **athena-3lu** (acceptance test suite) — blocked by athena-4xm, athena-8b9, athena-fgo.

## Key Code Context

**Current hard-coded BF branch** (`normalization.py:136-138`):
```python
elif kind is DivergenceKind.BayesFactor:
    bf = max(transformed_value, 0.0)
    raw_score = 1.0 - 1.0 / (1.0 + bf)
```

**Locked replacement function** (`ceiling_analysis.py:95-97`):
```python
def bf_norm_log_scaled(bf: float, c: float) -> float:
    log_term = math.log1p(bf)
    return log_term / (log_term + c)
# c = 0.083647 calibrated for bf_max_target=10000
```

**NormalizationConfig** (`normalization.py:17-26`) — needs a new field for BF normalization hook:
```python
@dataclass(frozen=True)
class NormalizationConfig:
    absolute_difference_sigmoid: SigmoidParams = field(...)
    custom_sigmoids: dict[str, SigmoidParams] = field(default_factory=dict)
    clip_eps: float = 1e-12
    se_dampen_enabled: bool = False
    se_dampen_k: float = 5.0
    se_dampen_x0: float = 2.0
```

**The hybrid uses normalization via** (`candidates.py:392`):
```python
score, local_warnings, score_diag = normalize_component(component, cfg.normalization)
```

**Patched hybrid from ceiling analysis** (`ceiling_analysis.py`) used `aggregate_hybrid_patched()` which injected an alternate BF normalization into the pipeline. The seam should formalize this pattern.

## Files Map

| Path | Role | Action Needed |
|------|------|---------------|
| `.../aggregation-candidates/normalization.py` | Normalization functions + `NormalizationConfig` | **MODIFY**: Add BF norm hook to config, update `normalize_component` BF branch |
| `.../aggregation-candidates/candidates.py` | `aggregate_hybrid()` + `HybridConfig` | Verify unchanged behavior (uses `normalize_component`) |
| `.../aggregation-candidates/evaluate.py` | 7-scenario evaluation harness | Run to verify 7/7 still passes |
| `.../aggregation-candidates/ceiling_analysis.py` | Contains `bf_norm_log_scaled` reference impl | Source for the locked function |
| `.../aggregation-candidates/models.py` | Dataclass contracts | Likely unchanged |
| `.../aggregation-candidates/scenarios.py` | 7 scenario fixtures | Unchanged |
| `.../aggregation-candidates/aggregate_score_recommendation.md` | Locked spec — source of truth | READ ONLY |
| `.../aggregation-candidates/aggregate_score_recommendation.json` | Locked parameters (machine-readable) | READ ONLY |
| `.../aggregation-candidates/aggregate_score_acceptance_test_spec.md` | Test contract from Session 7 | Reference for acceptance criteria |

## Loop State

N/A — this is a direct implementation session, not a Claude-Codex review loop.

## Next Steps

1. **Read athena-4xm bead** — `bd show athena-4xm` for acceptance criteria.
2. **Read locked spec** — `aggregate_score_recommendation.md` Section 1.2 (BF normalization) and Section 6 note 1 (seam requirement).
3. **Implement the seam in `normalization.py`**:
   - Add a BF normalization callable field to `NormalizationConfig` (default: `bf_norm_log_scaled` with `c=0.083647`).
   - Move `bf_norm_log_scaled` from `ceiling_analysis.py` into `normalization.py` as the canonical location.
   - Update the `DivergenceKind.BayesFactor` branch in `normalize_component()` to use the configurable hook instead of hard-coded `1 - 1/(1+bf)`.
4. **Prove behavioral equivalence** — Run `evaluate.py` and verify 7/7 pass with margins matching `aggregate_score_recommendation.json` baseline_margins within `1e-6`.
5. **Update FINDINGS.md** — Session 8 log entry.
6. **Close athena-4xm** — `bd close athena-4xm` + session close protocol.
7. **Then proceed to**: athena-8b9 → athena-fgo → athena-3lu.

## Beads (Implementation Dependency Graph)

```
✓ athena-9h2 (CLOSED)  ──▶  ○ athena-4xm (OPEN)  ──blocks──▶  ○ athena-3lu
  S7 integration               BF seam                           Acceptance suite

✓ athena-9h2 (CLOSED)  ──▶  ○ athena-8b9 (OPEN)  ──blocks──▶  ○ athena-3lu
                               x0 guardrail

✓ athena-9h2 (CLOSED)  ──▶  ○ athena-fgo (OPEN)  ──blocks──▶  ○ athena-3lu
                               Decomposition invariant

○ athena-i4s (OPEN) — monitoring hooks (independent)
```

## Session Artifacts

- Locked spec: `aggregate_score_recommendation.md`, `aggregate_score_recommendation.json`
- Architecture: `ARCHITECTURE.md` (Section 4.4.1), `decisions/002-aggregate-score-contract.md`
- Test contract: `aggregate_score_acceptance_test_spec.md`
- Monitoring contract: `monitoring_triggers.md`
- Prompt #17: `.claude/prompts/prompt_017_2026-02-22_aggregate-score-architecture-integration.md`
- Previous handoff: `.claude/handoffs/handoff_018_2026-02-22_session6-aggregate-score-recommendation.md`

## Documentation Updated

No documentation updates needed — CLAUDE.md and ARCHITECTURE.md are current. The project remains in "Research (Active Investigation)" phase since all code changes are to prototype artifacts in `prototypes/`.
