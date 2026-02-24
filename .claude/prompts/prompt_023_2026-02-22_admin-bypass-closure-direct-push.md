# Prompt: Admin Bypass Closure and Direct-Push Proof

> Generated: 2026-02-22 | Prompt #23 | Framework: RISEN

---

## Session Goal

Close the admin direct-push bypass on master by enforcing `contract-verification` checks on admins too (`enforce_admins: true`), proving direct pushes are rejected while the PR path still works, cleaning up proof PRs #3 and #4 from Session 12, and recording all evidence as Session 13 in FINDINGS.md.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Sequential procedural pattern matching Sessions 11-12 (configure policy → prove it works → clean up → record evidence). Clear step dependencies, explicit methodology, constraints on what not to break.
- **Alternatives considered:** TIDD-EC (dos/don'ts are important but RISEN's Narrowing handles them within the procedural frame)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal unambiguous: close bypass, prove rejection, clean up, record |
| Specificity | 9/10 | Exact API fields, branch names, PR numbers, verification commands specified |
| Context | 9/10 | Sessions 11-12 chain established, current protection state referenced |
| Completeness | 9/10 | All steps including final commit-via-PR (since direct push is now blocked), admin identity check |
| Structure | 9/10 | Clean RISEN sections, 8 ordered steps, 9-item end goal |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a repository governance specialist working on the ATHENA project. You have expertise in GitHub branch protection rules (including admin enforcement), `gh` CLI operations, and research-project documentation (FINDINGS.md append-only logs, bead-based issue tracking). This session is about closing a policy bypass — making the existing contract-verification gate airtight, not creating new CI.

INSTRUCTIONS:
Sessions 11-12 established a CI gate (`.github/workflows/contract-gate.yml`) and proved it blocks PR merges when `contract-verification` fails. However, the current branch protection has `enforce_admins: false`, meaning administrators can bypass the required check by pushing directly to master. This session closes that bypass by setting `enforce_admins: true`, proving that direct pushes are rejected, confirming the PR path still works, cleaning up leftover proof PRs #3 and #4 from Session 12, and recording all evidence as Session 13 in FINDINGS.md.

Important: Resolve `{owner}/{repo}` in all `gh api` URLs by running `gh repo view --json owner,name` first and substituting the actual values throughout.

STEPS:
1. Verify current state and preconditions:
   - Resolve repo identity: `gh repo view --json owner,name` — use these values for all `gh api` calls
   - Confirm the current authenticated user has admin access: `gh api repos/{owner}/{repo}/collaborators/{username}/permission --jq .permission` should return `admin`
   - Capture the full current branch protection: `gh api repos/{owner}/{repo}/branches/master/protection` — save this response
   - Confirm `enforce_admins.enabled` is currently `false`
   - Record the full current protection state as Session 13 "before" evidence

2. Enable admin enforcement:
   - Replay the full current protection state from Step 1 in the PUT payload, changing ONLY `enforce_admins` to `true`. Do not assume `null` for any field you have not inspected — if `required_pull_request_reviews` or `restrictions` have values, preserve them.
   - Example (adjust based on actual Step 1 output):
     ```
     gh api repos/{owner}/{repo}/branches/master/protection \
       --method PUT \
       --input - <<'EOF'
     {
       "required_status_checks": {
         "strict": false,
         "contexts": ["contract-verification"]
       },
       "enforce_admins": true,
       "required_pull_request_reviews": null,
       "restrictions": null
     }
     EOF
     ```
   - Verify: `gh api repos/{owner}/{repo}/branches/master/protection` now shows `enforce_admins.enabled: true` and all other fields unchanged
   - If the API call fails with 403 or permissions error, stop and report. Do not attempt workarounds.

3. Prove direct push is rejected:
   - Create a disposable local branch `ci-proof/direct-push` from master
   - Make a trivial change (e.g., add a blank line to a non-functional file)
   - Commit locally
   - Attempt to push directly to master: `git push origin ci-proof/direct-push:master`
   - Expected result: push is REJECTED by GitHub's branch protection
   - Record: the exact error message from the rejected push (this is the key evidence)
   - Clean up: delete the local branch `git branch -D ci-proof/direct-push`

4. Prove PR path still works:
   - Create a disposable branch `ci-proof/pr-path` from master
   - Make a trivial, harmless change
   - Push the branch and open a PR: `gh pr create --title "CI proof: PR path still works" --body "Disposable PR for Session 13. Verifies PR workflow functions with enforce_admins=true. Will be closed after verification."`
   - Wait for check: `gh pr checks <PR_NUMBER> --watch`
   - Confirm: `contract-verification` check passes, PR shows as mergeable (`gh pr view <PR_NUMBER> --json mergeStateStatus`)
   - Record: PR number, PR URL, workflow run ID, check status, mergeable state
   - Do NOT merge — close the PR after recording evidence
   - Delete the remote branch: `git push origin --delete ci-proof/pr-path`

5. Clean up proof PRs #3 and #4 from Session 12:
   - Verify PRs #3 and #4 are closed: `gh pr view 3 --json state` and `gh pr view 4 --json state`
   - If either is still open, close it: `gh pr close <number>`
   - Check for associated branches (from Session 12, likely named `ci-proof/pass` and `ci-proof/fail`); delete any that remain: `git push origin --delete <branch-name>`
   - Record the final state of PRs #3 and #4 (closed, branches deleted)

6. Record Session 13 in FINDINGS.md:
   - Collect all evidence artifacts from Steps 1-5 before writing
   - Add Investigation Log entry at top (reverse chronological):
     - Header: `### 2026-02-22 -- Session 13: Admin Bypass Closure and Direct-Push Rejection Proof`
     - Sections: Scope, Method, Findings (with before/after protection state, direct-push rejection error message, PR-path pass evidence, PRs #3/#4 cleanup confirmation), Implications, Open Threads
     - Follow exact format of Sessions 9-12
   - Update Accumulated Findings "What We Know" to include:
     - `enforce_admins: true` now active on master — admins cannot bypass required checks
     - Direct push to master is rejected (cite error message from Step 3)
     - PR path confirmed functional with admin enforcement enabled
     - Governance chain complete: workflow (S11) → PR enforcement proof (S12) → admin bypass closure (S13)
     - Cite Session 13

7. Manage beads:
   - Check `bd ready` / `bd list --status=open` for relevant issues
   - If relevant bead exists: `bd update <id> --status=in_progress`
   - If none: `bd create --title="Close admin direct-push bypass on master" --description="Set enforce_admins=true, prove direct push rejected, confirm PR path works, clean up S12 PRs. Session 13." --type=task --priority=2`
   - After commit: `bd close <id>`
   - If `bd sync` fails, report error and proceed

8. Commit and push via PR (direct push is now blocked!):
   - Stage: `research/adversarial-reward/FINDINGS.md`
   - Run `bd sync` (non-blocking on failure)
   - Commit locally on branch `session-13/findings`:
     - `git checkout -b session-13/findings`
     - `git add research/adversarial-reward/FINDINGS.md`
     - `git commit -m "research(adversarial-reward): Session 13 — admin bypass closure and direct-push rejection proof"`
   - Push and open PR:
     - `git push -u origin session-13/findings`
     - `gh pr create --title "Session 13: admin bypass closure proof" --body "FINDINGS.md update with enforcement evidence. Auto-merge after contract-verification passes."`
   - Wait for check: `gh pr checks <PR_NUMBER> --watch`
   - If `contract-verification` passes: merge the PR via `gh pr merge <PR_NUMBER> --merge --delete-branch`
   - If `contract-verification` fails: STOP and investigate — FINDINGS.md changes should not affect the contract scripts. Do not force-merge.
   - Run `bd sync` after merge (non-blocking on failure)
   - Switch back to master and pull: `git checkout master && git pull`

END GOAL:
After execution, ALL of the following must be true:
- `enforce_admins.enabled` is `true` on master branch protection
- Direct push to master was attempted and rejected — error message recorded
- PR path confirmed functional: check passed, mergeable, evidence recorded
- PRs #3 and #4 from Session 12 are closed and their branches deleted
- FINDINGS.md contains Session 13 entry with before/after protection state, rejection evidence, PR-path evidence, and cleanup confirmation
- FINDINGS.md Accumulated Findings "What We Know" updated with admin enforcement proof, citing Session 13
- Governance chain documented: workflow (S11) → PR enforcement (S12) → admin bypass closure (S13)
- A bead tracks this session and is closed
- Changes committed and pushed to remote (via PR, since direct push is now blocked)

NARROWING:
- Do NOT modify acceptance_test.py or monitoring_hooks.py — locked contract verification scripts
- Do NOT modify .github/workflows/contract-gate.yml — the workflow is already correct
- Do NOT merge disposable proof PRs (Steps 3-4) — close after recording evidence
- Do NOT leave disposable branches alive — delete all after use
- Do NOT weaken existing branch protection settings — replay full current state, only change enforce_admins
- Do NOT assume null for protection fields you haven't inspected — read before writing
- Do NOT edit or delete previous Investigation Log entries — FINDINGS.md is append-only
- Do NOT attempt to work around permission errors — stop and report
- Do NOT push the final FINDINGS.md commit directly to master — use a PR (the policy you just set blocks it)
- Do NOT force-merge if contract-verification fails on the final PR — investigate first
- Stay within: `gh` CLI for all GitHub API operations
- Avoid: `bd edit` (opens $EDITOR); use `bd update`/`bd close` instead
- Out of scope: new CI jobs, workflow changes, Python script modifications
```

---

## Review Findings

### Issues Addressed
1. **[Warning] Step 8 circular dependency** — Added explicit branch name `session-13/findings`, PR title/body, merge instruction (`gh pr merge --merge --delete-branch`), and failure contingency (stop and investigate if check fails)
2. **[Warning] PUT payload overwrites** — Added instruction to replay full current protection state from Step 1, only changing `enforce_admins`. Explicit: "Do not assume `null` for any field you have not inspected"
3. **[Warning] `{owner}/{repo}` placeholders** — Added resolution step using `gh repo view --json owner,name` at the start of Step 1
4. **[Warning] Admin identity verification** — Added `gh api .../collaborators/{username}/permission` check as a precondition in Step 1

### Remaining Suggestions
- Evidence could be staged to a scratch file before writing FINDINGS.md — added "collect all evidence artifacts" note to Step 6
- PRs #3/#4 branch names from Session 12 (likely `ci-proof/pass` and `ci-proof/fail`) — noted in Step 5
- Bead priority scale not documented — consistent with prompts #21-22, not worth changing
- Date inconsistency between prior sessions is inherited, not this prompt's concern

## Usage Notes

- **Best used with:** Claude Code or any agentic coding assistant with `gh` CLI access (admin scope), file editing, git, and bash
- **Adjust for:** Actual PR numbers if #3/#4 differ; actual branch names from Session 12 if different from `ci-proof/pass`/`ci-proof/fail`
- **Prerequisites:** Admin access to the GitHub repository (required for `enforce_admins`), `gh` CLI authenticated with `repo` and `admin:repo_hook` scopes
- **Key difference from prior sessions:** Final commit MUST go via PR since direct push will be blocked by the policy change made in Step 2
