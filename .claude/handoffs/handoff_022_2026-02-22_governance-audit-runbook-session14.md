# Session Handoff: Governance Audit Runbook and Break-Glass Procedure

> Generated: 2026-02-22 | Handoff #22 | Previous: handoff_021_2026-02-22_ci-gate-contract-enforcement.md

---

## Continuation Directive

Turn the one-time enforcement proof (Sessions 11-13) into a repeatable governance routine. Session 14 should write the "drill card" — a codified baseline, a 5-minute audit runbook, and a break-glass procedure — so any team member can verify branch governance is intact without guessing. Also fix or document the `bd list` panic workaround observed during Session 13.

Done criterion: a new person can run one checklist and confirm branch governance is still intact.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial-reward research track has a locked AggregateScore contract enforced by CI and branch protection.

**Goal:** Codify the governance chain established in Sessions 11-13 into durable, repeatable artifacts: a baseline definition, audit runbook, and emergency procedure.

**Success criteria:**
1. "Must-stay-true" baseline defined (contract-verification required, enforce_admins=true)
2. 5-minute audit runbook with exact `gh api` commands, expected outputs, and evidence logging instructions
3. Break-glass procedure documenting how admin override works in emergencies, who approves, and how settings are restored
4. `bd list` panic issue fixed or workaround documented
5. Session 14 logged in FINDINGS.md

## Key Decisions & Rationale

1. **Log-scaled BF normalization is the locked default** — `log1p(bf) / (log1p(bf) + c)` with `c=0.083647`
   - Rationale: All contracts verify against this. Locked since Session 5.

2. **Run-both-then-gate CI pattern** — Both acceptance_test.py and monitoring_hooks.py run even if one fails; final gate step checks both.
   - Rationale: CI logs contain both "what broke" and "what changed" diagnostics.
   - Alternative rejected: fail-fast (loses monitoring output when acceptance fails).

3. **enforce_admins=true on master** — Admins cannot bypass required checks.
   - Rationale: Without this, contract drift can merge via direct push even though PR enforcement works (the "staff side door").
   - Established in Session 13.

4. **No pull request review requirements** — Solo research project.
   - Rationale: Review gates would block the sole contributor without adding value.

5. **strict: false on required status checks** — Branches don't need to be up-to-date with master before merging.
   - Rationale: Avoids unnecessary rebase churn for a research repo.

6. **Final commits go via PR** — Since direct push to master is blocked by enforce_admins, all FINDINGS.md updates must be pushed through a PR branch.
   - Established in Session 13 (prompt #23, Step 8).

## Current State

### Completed
- **Session 11** (commit `833c331`): Created `.github/workflows/contract-gate.yml`. CI runs acceptance_test.py (15/15) and monitoring_hooks.py (20/20) on every push/PR to master. Bead `athena-aqh` closed.
- **Session 12**: Branch protection enabled on master requiring `contract-verification` check. Passing PR and failing PR smoke tests proved enforcement works. PRs created, evidence recorded, PRs closed.
- **Session 13**: `enforce_admins` set to `true`. Direct push to master rejected (error recorded). PR path confirmed still functional. PRs #3/#4 from Session 12 cleaned up.
- **Prompts created this session**: #21 (CI gate), #22 (branch protection proof), #23 (admin bypass closure).
- **CLAUDE.md updated**: Added `.github/workflows/contract-gate.yml` to directory structure.

### In Progress
- Nothing in progress. Session 14 is the next piece of work.

### Blocked / Open Questions
- **`bd list` panic**: Encountered during Session 13 execution. Needs investigation — either fix the underlying issue or document the workaround. Details should be in the Session 13 FINDINGS.md entry.
- **Audit runbook format**: Where should it live? Options: `research/adversarial-reward/governance/`, a new `governance/` top-level directory, or within FINDINGS.md itself. Session 14 should decide.
- **Break-glass approval process**: Who approves emergency admin override? For a solo project this is self-approval with documentation, but the procedure should be written to scale.

## Key Code Context

**`.github/workflows/contract-gate.yml`** — The CI gate (created Session 11)
- Triggers on push + pull_request to master
- Job `contract-verification` on ubuntu-latest, Python 3.12
- `defaults.run.working-directory: research/adversarial-reward/prototypes/aggregation-candidates`
- Runs both scripts via `tee` to capture artifacts
- Final gate step fails if either script returned non-zero

**Branch protection state** (after Session 13):
```
required_status_checks.contexts: ["contract-verification"]
required_status_checks.strict: false
enforce_admins.enabled: true
required_pull_request_reviews: null
restrictions: null
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `.github/workflows/contract-gate.yml` | CI gate for contract enforcement | Created S11, verified S12-13 |
| `research/adversarial-reward/FINDINGS.md` | Investigation log (Sessions 1-13) | Updated through S13 |
| `.../aggregation-candidates/acceptance_test.py` | Contract correctness proof (15 checks) | Locked, do not modify |
| `.../aggregation-candidates/monitoring_hooks.py` | Drift detection (20 checks, T1-T5) | Locked, do not modify |
| `.../aggregation-candidates/aggregate_score_recommendation.json` | Locked contract source of truth | Read only |
| `CLAUDE.md` | Project governance | Updated this session (added .github/workflows/) |

## Loop State

N/A — This session was prompt authoring (prompts #21-23) and handoff preparation. Sessions 12-13 were executed in separate conversation windows using prompts #22 and #23.

## Next Steps

1. **Read FINDINGS.md** Sessions 11-13 entries to understand the full governance chain evidence.
2. **Define the "must-stay-true" baseline** — Codify the exact branch protection state that must hold: `contract-verification` required, `enforce_admins: true`, `strict: false`, no PR reviews required. Decide where this baseline document lives.
3. **Create 5-minute audit runbook** — Exact `gh api` commands, expected JSON outputs for each field, pass/fail criteria, and instructions for where to log audit evidence. Should be runnable by anyone with repo read access.
4. **Document break-glass procedure** — How to temporarily disable `enforce_admins` for emergencies, who approves (even if self-approval for now), mandatory restoration steps, and where the override is logged.
5. **Fix or document `bd list` panic** — Investigate the panic seen in Session 13, either fix the root cause or document the workaround and its limitations.
6. **Log Session 14 in FINDINGS.md** — Investigation Log entry + Accumulated Findings update.
7. **Manage beads** — Create/claim bead, close after commit, `bd sync`.

## Session Artifacts

- **Prompt #21**: `.claude/prompts/prompt_021_2026-02-22_ci-gate-contract-enforcement.md` (Session 11 CI gate)
- **Prompt #22**: `.claude/prompts/prompt_022_2026-02-22_branch-protection-enforcement-proof.md` (Session 12 enforcement proof)
- **Prompt #23**: `.claude/prompts/prompt_023_2026-02-22_admin-bypass-closure-direct-push.md` (Session 13 admin bypass)
- **Handoff #21**: `.claude/handoffs/handoff_021_2026-02-22_ci-gate-contract-enforcement.md` (previous handoff)
- **Session 11 commit**: `833c331` — CI gate workflow + FINDINGS.md Session 11 entry

## Documentation Updated

| Document | Change Summary | Status |
|----------|---------------|--------|
| CLAUDE.md | Added `.github/workflows/contract-gate.yml` to directory structure | Approved and applied |
