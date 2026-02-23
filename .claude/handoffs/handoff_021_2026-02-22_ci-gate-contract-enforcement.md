# Session Handoff: CI Gate for AggregateScore Contract Enforcement

> Generated: 2026-02-22 | Handoff #21 | Previous: handoff_020_2026-02-22_acceptance-test-suite-athena-3lu.md

---

## Continuation Directive

Wire `acceptance_test.py` and `monitoring_hooks.py` into a CI/pre-merge gate so the locked AggregateScore contract is continuously enforced on every change. Right now, both scripts exist and pass (15/15 and 20/20 respectively), but they only run when someone manually invokes them. The next session adds the automation — a pipeline that runs both, fails on non-zero exit, and saves output as a build artifact.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial-reward research track has a locked AggregateScore recommendation (`aggregate_score_recommendation.json`) with three enforced contracts and five revisit triggers.

**Goal:** Automate the two verification scripts as a CI gate so contract drift cannot merge silently. This is the "wiring the alarm to power" step — the alarm (monitoring_hooks.py) and the inspection report (acceptance_test.py) exist; they need to run automatically.

**Success criteria:**
1. CI pipeline runs `python acceptance_test.py` and `python monitoring_hooks.py` on every push/PR
2. Pipeline fails on any non-zero exit code
3. Monitoring output saved as a build artifact/log for trigger failure visibility
4. Session documented in `research/adversarial-reward/FINDINGS.md`

**Constraints:** ATHENA has no existing CI infrastructure. This will be the first pipeline. The prototype is stdlib-only Python — no special dependencies needed beyond Python 3.10+.

## Key Decisions & Rationale

1. **Log-scaled BF normalization is the locked default** — `log1p(bf) / (log1p(bf) + c)` with `c=0.083647`. All contracts verify against this.
   - Rationale: Replaced ceiling-hitting `bf/(bf+1)` in Session 4.1.

2. **Acceptance tests are one-time correctness proofs; monitoring hooks detect drift** — Two complementary scripts with distinct purposes. Both must pass for the gate to be green.
   - Rationale: Session 9 built acceptance_test.py (proves contracts hold now), Session 10 built monitoring_hooks.py (detects when conditions change).

3. **Monitoring report is deterministic** — Uses `recommendation.date` from JSON, not wall-clock time. Same codebase + same JSON = same output.
   - Rationale: Reproducibility requirement for CI — flaky timestamps would cause false diffs.

4. **T1-T5 trigger alerts include one-line action summaries, not full prose** — If a trigger trips, the output says what to do (e.g., "open bead tagged revisit-T2") without dumping the full monitoring_triggers.md action path.
   - Rationale: CI logs need to be scannable.

## Current State

### Completed
- **Sessions 1-8:** Research, locked recommendation, architecture integration, BF seam + guardrail + decomposition invariant implementation.
- **Session 9 (athena-3lu, CLOSED):** `acceptance_test.py` — 15/15 PASS (7 margin-parity, 1 guardrail, 7 decomposition). Commit `4bad359`.
- **Session 10 (athena-i4s, CLOSED):** `monitoring_hooks.py` — 20/20 PASS (4 T1 envelope, 6 T2 coverage, 1 T3 Pattern B, 2 T4 scenarios, 2 T5 correlation, 5 metadata). Commit `b956e91`.
- Both prompts saved: `prompt_019` (acceptance test) and `prompt_020` (monitoring hooks).

### In Progress
- Nothing in progress. The CI gate is the next piece of work.

### Blocked / Open Questions
- **No existing CI infrastructure.** Need to decide: GitHub Actions? Pre-commit hook? Both? The repo is on GitHub (`andrewmcadoo/athena`).
- **Python version.** Scripts use `dict[str, X]` and `X | Y` syntax — requires Python 3.10+. CI runner needs this.
- **Working directory.** Both scripts must run from `research/adversarial-reward/prototypes/aggregation-candidates/`. The CI step needs to `cd` there first.

## Key Code Context

**`acceptance_test.py`** — Runs 15 checks across 3 contracts. Entry: `python acceptance_test.py`. Exit 0 = all pass, exit 1 = failure.
```
PASS S1_noisy_tv: margin=0.030646615, expected=0.030647000, delta=3.851e-07
...
15/15 passed
contract_metadata version=1.0 bf_norm_c=0.083647 n_terms=1 guardrail_enabled=true
```

**`monitoring_hooks.py`** — Runs 20 checks across T1-T5 + metadata. Entry: `python monitoring_hooks.py`. Exit 0 = all clear, exit 1 = trigger alert.
```
--- T1: Operating Envelope ---
PASS t1.bf_norm_c_matches_contract: ...
...
20/20 checks passed
contract_metadata version=1.0 bf_norm_c=0.083647 n_terms=1 guardrail_status=ok date=2026-02-22
```

**Trigger alert format** (when checks fail):
```
--- TRIGGER ALERT ---
T2 tripped (t2.kind.NewKind): open bead tagged revisit-T2; implement normalization for new kind...
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `.../aggregation-candidates/acceptance_test.py` | One-time contract correctness proof (15 checks) | Committed, verified |
| `.../aggregation-candidates/monitoring_hooks.py` | Continuous drift detection (20 checks, T1-T5) | Committed, verified |
| `.../aggregation-candidates/aggregate_score_recommendation.json` | Locked contract source of truth | READ ONLY |
| `.../aggregation-candidates/monitoring_triggers.md` | T1-T5 trigger specifications and action paths | Reference |
| `research/adversarial-reward/FINDINGS.md` | Investigation log (Sessions 1-10 entries) | Update with Session 11 |
| `.github/workflows/` or equivalent | **TO CREATE** — CI pipeline | New |

## Loop State

**Iteration 3** of Claude-Codex-Claude workflow:
- **Iteration 1** (Session 8): Codex implemented BF seam, guardrail, decomposition invariant. Claude verified: 7/7 PASS.
- **Iteration 2** (Sessions 9-10): Codex implemented acceptance_test.py (15/15) and monitoring_hooks.py (20/20). Claude verified both.
- **Iteration 3**: Next session creates CI gate. Can be Claude-direct (GitHub Actions YAML is straightforward) or Codex if preferred.

## Next Steps

1. **Check for existing CI config.** Look for `.github/workflows/`, `.gitlab-ci.yml`, `Makefile`, or any CI-adjacent files in the repo.
2. **Create CI pipeline** (likely `.github/workflows/contract-gate.yml`) that:
   - Triggers on push/PR to master
   - Sets up Python 3.10+
   - Runs `cd research/adversarial-reward/prototypes/aggregation-candidates && python acceptance_test.py`
   - Runs `cd research/adversarial-reward/prototypes/aggregation-candidates && python monitoring_hooks.py`
   - Fails on non-zero exit from either
   - Saves stdout from both as build artifacts
3. **Verify** by pushing a commit and checking the Actions tab (or running locally via `act` if available).
4. **Update** `research/adversarial-reward/FINDINGS.md` with Session 11 log entry.
5. **Close** the relevant bead and sync.

## Session Artifacts

- Prompt #19: `.claude/prompts/prompt_019_2026-02-22_acceptance-test-suite-aggregation.md`
- Prompt #20: `.claude/prompts/prompt_020_2026-02-22_monitoring-hooks-aggregate-contract.md`
- Handoff #20: `.claude/handoffs/handoff_020_2026-02-22_acceptance-test-suite-athena-3lu.md`
- Session 9 commit: `4bad359` (acceptance_test.py)
- Session 10 commit: `b956e91` (monitoring_hooks.py)
- FINDINGS.md: Sessions 9 and 10 log entries present.

## Documentation Updated

No documentation updates — all project docs were current. CLAUDE.md verified; project remains in "Research (Active Investigation)" phase.
