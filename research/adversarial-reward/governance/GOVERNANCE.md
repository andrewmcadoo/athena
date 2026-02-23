# AggregateScore Governance Runbook

Last updated: 2026-02-23 (Session 16 — audit automation + escalation thresholds)
Scope: `andrewmcadoo/athena` branch protection + contract-gate CI governance for `master`.

## Section 1 — Must-Stay-True Baseline

Verified live source of truth:
- Repo identity command: `gh repo view --json nameWithOwner --jq .nameWithOwner`
- Protection command: `gh api repos/andrewmcadoo/athena/branches/master/protection`

Expected branch protection baseline (normalized from live API output):

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "contract-verification"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": false,
  "lock_branch": false,
  "allow_fork_syncing": false
}
```

Critical field requirements:

| Field | Required value | Rationale |
| :--- | :--- | :--- |
| `required_status_checks.contexts` | `["contract-verification"]` | Ensures the locked AggregateScore contract gate is required before merge. |
| `required_status_checks.strict` | `true` | Requires PR branches to be up to date with `master` before merge; prevents stale-green merges. |
| `enforce_admins.enabled` | `true` | Removes administrator bypass; direct pushes cannot skip required checks. |
| `allow_force_pushes.enabled` | `false` | Prevents force-push rewrite of protected branch history. |

CI workflow contract:

| Contract element | Required state | Evidence anchor |
| :--- | :--- | :--- |
| Workflow file | `.github/workflows/contract-gate.yml` exists and is active | Session 11, Session 14 live audit |
| Triggers | `push` and `pull_request` on `master` | Session 11 workflow definition |
| Script execution | Runs `acceptance_test.py` and `monitoring_hooks.py` | Session 11 + Session 12 |
| Gate logic | Final job fails unless both script steps succeed | Session 11 |
| Exit-code integrity | `set -o pipefail` required in piped script steps | Session 12 fix |

Governance evidence chain:

| Session | Governance milestone | Proof |
| :--- | :--- | :--- |
| Session 11 | CI gate implemented | Workflow + passing local script evidence logged in FINDINGS |
| Session 12 | Branch-protection required-check enforcement proven | Passing PR (`#3`), failing PR (`#4`), merge-block evidence |
| Session 13 | Admin bypass closed | `enforce_admins=true`, direct-push rejection (`GH006`) |
| Session 14 | Runbook codified for repeatable audits + incident restoration | This document + live re-verification |

## Section 2 — Five-Minute Audit Runbook

Run these steps in order. Total checks: 7.

1. Resolve repo identity.
   Command:
   ```bash
   gh repo view --json nameWithOwner --jq '.nameWithOwner'
   ```
   Expected output:
   ```text
   andrewmcadoo/athena
   ```

2. Check branch protection (4 checks, run each command independently).
   Prepare variables:
   ```bash
   REPO="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
   OWNER="${REPO%/*}"
   NAME="${REPO#*/}"
   ```
   Check 2.1 contexts:
   ```bash
   gh api "repos/$OWNER/$NAME/branches/master/protection" \
     --jq '.required_status_checks.contexts'
   ```
   Expected output:
   ```text
   ["contract-verification"]
   ```
   Check 2.2 strict mode:
   ```bash
   gh api "repos/$OWNER/$NAME/branches/master/protection" \
     --jq '.required_status_checks.strict'
   ```
   Expected output:
   ```text
   true
   ```
   Check 2.3 admin enforcement:
   ```bash
   gh api "repos/$OWNER/$NAME/branches/master/protection" \
     --jq '.enforce_admins.enabled'
   ```
   Expected output:
   ```text
   true
   ```
   Check 2.4 force pushes:
   ```bash
   gh api "repos/$OWNER/$NAME/branches/master/protection" \
     --jq '.allow_force_pushes.enabled'
   ```
   Expected output:
   ```text
   false
   ```

3. Verify CI workflow is active.
   Command:
   ```bash
   gh api "repos/$OWNER/$NAME/actions/workflows" \
     --jq '.workflows[] | select(.path==".github/workflows/contract-gate.yml") | .state'
   ```
   Expected output:
   ```text
   active
   ```

4. Verify most recent contract-gate run conclusion.
   Command:
   ```bash
   gh api "repos/$OWNER/$NAME/actions/workflows/contract-gate.yml/runs?per_page=1" \
     --jq '.workflow_runs[0] | {status, conclusion}'
   ```
   Expected output:
   ```text
   {"status":"completed","conclusion":"success"}
   ```

5. Record result.
   - If all 7 checks pass: update `Last audited` date in this document and optionally add a short note to FINDINGS if this was a formal governance review.
   - If any check fails: stop, create/open a bead immediately, and log the failure + remediation path in `research/adversarial-reward/FINDINGS.md`.

Pass/fail summary (7 checks):

| Check ID | jq path / query | Pass value | Fail indicator |
| :--- | :--- | :--- | :--- |
| C1 | `.nameWithOwner` | `andrewmcadoo/athena` | Any other repository identity |
| C2 | `.required_status_checks.contexts` | `["contract-verification"]` | Missing/extra contexts or different check name |
| C3 | `.required_status_checks.strict` | `true` | `false` or missing |
| C4 | `.enforce_admins.enabled` | `true` | `false` or missing |
| C5 | `.allow_force_pushes.enabled` | `false` | `true` or missing |
| C6 | Workflow query for `.github/workflows/contract-gate.yml` state | `active` | Workflow missing, disabled, or archived |
| C7 | `.workflow_runs[0].status` + `.workflow_runs[0].conclusion` | `completed` + `success` | Any non-success conclusion or non-completed status |

Audit evidence template (copy/paste):

```markdown
### Governance Audit Record
- Date (UTC): YYYY-MM-DDTHH:MM:SSZ
- Auditor: <name>
- Repo: andrewmcadoo/athena
- Commit SHA observed: <git rev-parse HEAD>
- C1 Repo identity: PASS/FAIL
- C2 Required contexts: PASS/FAIL
- C3 Strict mode: PASS/FAIL
- C4 Admin enforcement: PASS/FAIL
- C5 Force pushes disabled: PASS/FAIL
- C6 Workflow active: PASS/FAIL
- C7 Latest run success: PASS/FAIL
- Notes: <freeform>
```

### Audit Cadence

- **Weekly spot-check**: Run the 7-check runbook once per week. Log a Governance Audit Record in FINDINGS.md only if any check fails; otherwise update `Last audited` date in this document.
- **Mandatory pre-merge audit**: Run the full runbook before merging any PR that touches files in `research/adversarial-reward/prototypes/aggregation-candidates/` or `.github/workflows/`.
- **Post-incident audit**: After any break-glass procedure, run the full runbook as part of mandatory restoration (already required by Section 3).

### Escalation Thresholds

| Check | Severity | Failure meaning | Owner action | Response time |
| :--- | :--- | :--- | :--- | :--- |
| C1 | CRITICAL | Wrong repository — audit running against incorrect target | Stop immediately. Verify `gh repo view` context and re-run. | Immediate |
| C2 | CRITICAL | Required status check missing or altered — merges can bypass contract gate | Open bead. Restore `contract-verification` context via branch protection API. Re-run audit. | < 1 hour |
| C3 | HIGH | Strict mode disabled — stale-green merges possible | Open bead. Re-enable strict mode via branch protection API. Re-run audit. | < 1 hour |
| C4 | CRITICAL | Admin enforcement disabled — admins can bypass all checks | Treat as break-glass-level incident. Follow Section 3 restore sequence. Re-run audit. | < 30 min |
| C5 | CRITICAL | Force pushes enabled — branch history can be rewritten | Open bead. Disable force pushes via branch protection API. Re-run audit. | < 30 min |
| C6 | HIGH | Workflow disabled or missing — CI gate not running | Open bead. Re-enable workflow in GitHub Actions settings. Verify trigger config. Re-run audit. | < 1 hour |
| C7 | MEDIUM | Latest run failed — most recent CI execution did not succeed | Investigate run logs. If contract drift: fix and re-run. If transient: re-trigger and monitor. | < 4 hours |

Severity definitions:
- **CRITICAL**: Governance bypass is possible right now. Immediate remediation required.
- **HIGH**: Governance is degraded but not fully bypassed. Remediation within 1 hour.
- **MEDIUM**: Governance is intact but a recent execution anomaly needs investigation. Normal fix flow.

## Section 3 — Break-Glass Procedure

Use only for urgent repository-recovery changes where policy itself prevents emergency remediation.

Prerequisites:
- Admin access to the repository.
- Document reason before override (bead + timestamped note in FINDINGS planned entry).
- Confirm this is truly break-glass (see non-examples below).

Override steps:
1. Capture backup protection JSON:
   ```bash
   REPO="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
   OWNER="${REPO%/*}"
   NAME="${REPO#*/}"
   TS="$(date -u +%Y%m%dT%H%M%SZ)"
   gh api "repos/$OWNER/$NAME/branches/master/protection" > "/tmp/master-protection-backup-$TS.json"
   ```
2. Disable admin enforcement only (preserve all other fields from backup):
   ```bash
   jq '{
     required_status_checks: {
       strict: .required_status_checks.strict,
       contexts: .required_status_checks.contexts
     },
     enforce_admins: false,
     required_pull_request_reviews: .required_pull_request_reviews,
     restrictions: .restrictions,
     required_linear_history: .required_linear_history.enabled,
     allow_force_pushes: .allow_force_pushes.enabled,
     allow_deletions: .allow_deletions.enabled,
     block_creations: .block_creations.enabled,
     required_conversation_resolution: .required_conversation_resolution.enabled,
     lock_branch: .lock_branch.enabled,
     allow_fork_syncing: .allow_fork_syncing.enabled
   }' "/tmp/master-protection-backup-$TS.json" > "/tmp/master-protection-breakglass-$TS.json"

   gh api "repos/$OWNER/$NAME/branches/master/protection" \
     --method PUT \
     --input "/tmp/master-protection-breakglass-$TS.json"
   ```
3. Make emergency change.
4. Document exactly what changed and why.

Guardrails:
- Approval model (current solo project): self-approval is allowed, but documentation is mandatory before override.
- Approval model (future team): upgrade to two-admin approval for any override.
- Maximum override duration: 1 hour. Restoration is mandatory within this window.
- Mandatory restore sequence:
  1. Re-enable `enforce_admins=true`.
  2. Run full 7-check audit runbook.
  3. Close remediation bead.
- Restore failure fallback:
  - If API restore fails, manually restore in GitHub web UI: `Settings` -> `Branches` -> `master` protection rule.
  - Document manual restore path and verification evidence in FINDINGS.
- Required post-incident FINDINGS entry must include:
  - Trigger/reason
  - Override start/end timestamps and total duration
  - Exact changes made during break-glass
  - Restore verification evidence (7 checks)

What is NOT a break-glass scenario:
- CI failures caused by real contract drift (these require investigation and correction).
- `bd` panics or local tooling instability.
- Unrelated PR failures that do not block emergency repository recovery.

## Section 4 — Known Issues

- `bd list` panic: non-reproducing in Session 14; workaround remains `bd list --status=open`; no fix needed at this time.
- This list is intentionally minimal and living; add entries only when they are operationally relevant.
