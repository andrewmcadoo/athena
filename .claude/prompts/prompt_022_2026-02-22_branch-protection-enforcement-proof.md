# Prompt: Branch Protection Enforcement Proof

> Generated: 2026-02-22 | Prompt #22 | Framework: RISEN

---

## Session Goal

Enforce the contract-gate CI check via GitHub branch protection on master (requiring the `contract-verification` status check), then prove enforcement works by running a passing PR and a deliberately-failing PR (confirming merge-block), recording run IDs, PR links, and evidence as Session 12 in FINDINGS.md. Track in beads. Update both Session 12 Investigation Log entry and Accumulated Findings "What We Know."

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Sequential multi-step process (configure protection → passing PR → failing PR → cleanup → record evidence) with clear dependencies between steps, explicit methodology, and important constraints (don't merge disposable PRs, clean up branches). Same procedural pattern as Session 11.
- **Alternatives considered:** TIDD-EC (good for dos/don'ts but the task is fundamentally procedural with ordered steps)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal unambiguous: protection + two-PR proof + evidence recording |
| Specificity | 9/10 | Exact check name, API payload, polling commands, failure injection mechanism specified |
| Context | 9/10 | Session 11 context established, workflow file path referenced, repo structure known |
| Completeness | 9/10 | All steps covered including cleanup, both positive and negative proof, bead lifecycle |
| Structure | 9/10 | Clean RISEN sections, 7 ordered steps, 9-item end goal checklist |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a DevOps engineer and research-governance specialist working on the ATHENA project. You have expertise in GitHub branch protection rules (via `gh` CLI), GitHub Actions workflow verification, and research-project documentation (FINDINGS.md append-only logs, bead-based issue tracking). This is enforcement proof work — confirming policy prevents drift, not writing new CI.

INSTRUCTIONS:
The `.github/workflows/contract-gate.yml` workflow (committed in Session 11) runs acceptance_test.py (15 checks) and monitoring_hooks.py (20 checks) on every push/PR to master. However, the check is not yet required — a PR can still merge even if `contract-verification` fails. This session closes that gap by enabling branch protection requiring the `contract-verification` check, then proving enforcement with two smoke-test PRs: one that passes and one that deliberately fails (confirming merge is blocked). All evidence (run IDs, PR links, merge-block confirmation) is recorded as Session 12 in FINDINGS.md.

STEPS:
1. Set up branch protection on master requiring `contract-verification`:
   - Use `gh api` to configure branch protection:
     ```
     gh api repos/{owner}/{repo}/branches/master/protection \
       --method PUT \
       --input - <<'EOF'
     {
       "required_status_checks": {
         "strict": false,
         "contexts": ["contract-verification"]
       },
       "enforce_admins": false,
       "required_pull_request_reviews": null,
       "restrictions": null
     }
     EOF
     ```
   - `strict: false` avoids unnecessary rebase churn for a research repo
   - `required_pull_request_reviews: null` — solo research project, no review requirement
   - Verify the rule is active: `gh api repos/{owner}/{repo}/branches/master/protection` should show `contract-verification` in `required_status_checks.contexts`
   - If the API call fails with 403 or permissions error, stop and report the error. Do not attempt workarounds — branch protection requires admin access to the repository.

2. Smoke-test PR #1 — passing case:
   - Create a disposable branch `ci-proof/pass` from master
   - Make a trivial, harmless change (e.g., add a blank line to a non-functional area)
   - Push and open a PR: `gh pr create --title "CI proof: passing contract gate" --body "Disposable PR for Session 12 enforcement proof. Will be closed after verification."`
   - Wait for the check: `gh pr checks <PR_NUMBER> --watch`
   - Confirm: check status is "success", artifact `contract-gate-output` is present in the workflow run
   - Confirm merge-ability: `gh pr view <PR_NUMBER> --json mergeStateStatus` shows a mergeable state (proves branch protection allows merge when check passes)
   - Record: PR number, PR URL, workflow run ID, run URL, check status, mergeable state
   - Do NOT merge this PR — close it after recording evidence

3. Smoke-test PR #2 — failing case:
   - Create a disposable branch `ci-proof/fail` from master
   - Inject a failure: modify `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.json` to corrupt a value — e.g., change a `baseline_margins` entry to be far outside the 1e-6 tolerance, or remove a scenario entry to trigger a T4 scenario fixture coverage failure in monitoring_hooks.py
   - Push and open a PR: `gh pr create --title "CI proof: failing contract gate" --body "Disposable PR for Session 12 enforcement proof. Intentionally fails. Will be closed after verification."`
   - Wait for the check: `gh pr checks <PR_NUMBER> --watch`
   - Confirm: check status is "failure"
   - Confirm merge is blocked: `gh pr view <PR_NUMBER> --json mergeStateStatus` shows a non-mergeable state, OR `gh pr merge <PR_NUMBER>` is rejected with a message about required status checks
   - Record: PR number, PR URL, workflow run ID, run URL, check status, merge-blocked evidence
   - Do NOT merge this PR — close it after recording evidence

4. Clean up disposable artifacts:
   - Close both PRs: `gh pr close <number>` for each
   - Delete disposable branches: `git push origin --delete ci-proof/pass ci-proof/fail`
   - Verify: `gh pr list` shows no open disposable PRs

5. Record Session 12 in FINDINGS.md:
   - Add Investigation Log entry at top (reverse chronological):
     - Header: `### 2026-02-22 -- Session 12: Branch Protection Enforcement Proof`
     - Sections: Scope, Method, Findings (with PR numbers, run IDs, URLs, pass/fail/merge evidence), Implications, Open Threads
     - Follow exact format of Sessions 9-11
   - Update Accumulated Findings "What We Know" to include:
     - Branch protection requires `contract-verification` check on master
     - Passing PR evidence: PR #X, run ID Y — both scripts passed, artifact present, merge allowed
     - Failing PR evidence: PR #X, run ID Y — check failed, merge blocked
     - Cite Session 12

6. Manage beads:
   - Check `bd ready` / `bd list --status=open` for relevant issues
   - If relevant bead exists: `bd update <id> --status=in_progress`
   - If none: `bd create --title="Branch protection enforcement proof for contract gate" --description="Enable required status check contract-verification on master. Prove with pass/fail PRs. Session 12." --type=task --priority=2`
   - After commit: `bd close <id>`
   - If `bd sync` fails, report error and proceed

7. Commit and push:
   - Stage: `research/adversarial-reward/FINDINGS.md` (only file modified — branch protection is a GitHub API setting, not a committed file)
   - Run `bd sync` before and after commit (non-blocking on failure)
   - Commit message: `research(adversarial-reward): Session 12 — branch protection enforcement proof`
   - Push to remote

END GOAL:
After execution, ALL of the following must be true:
- Master branch protection requires `contract-verification` status check to pass before merging
- A passing PR was created, ran contract-gate workflow successfully, merge was confirmed allowed, and PR was closed with evidence recorded
- A failing PR was created, contract-gate workflow failed, merge was provably blocked, and PR was closed with evidence recorded
- Artifact `contract-gate-output` was confirmed present in the passing run
- Both disposable branches (`ci-proof/pass`, `ci-proof/fail`) are deleted
- FINDINGS.md contains Session 12 entry with PR numbers, run IDs, URLs, and pass/fail/merge evidence
- FINDINGS.md Accumulated Findings "What We Know" updated with enforcement proof, citing Session 12
- A bead tracks this session and is closed
- Changes committed and pushed to remote

NARROWING:
- Do NOT modify acceptance_test.py or monitoring_hooks.py — they are locked contract verification scripts
- Do NOT merge either disposable PR — close both after recording evidence
- Do NOT leave disposable branches alive — delete both after PR closure
- Do NOT enable pull request review requirements — solo research project
- Do NOT require branches to be up to date (`strict: false`) — avoids rebase churn
- Do NOT edit or delete previous Investigation Log entries — FINDINGS.md is append-only
- Do NOT write production code — this is enforcement proof for research artifacts
- Do NOT skip the failing PR test — the whole point is proving merge-block works
- Do NOT attempt to work around permission errors on branch protection — stop and report
- Stay within: `gh` CLI for all GitHub API operations (branch protection, PRs, checks)
- Avoid: using `bd edit` (opens $EDITOR which blocks agents); use `bd update`/`bd close` instead
- Out of scope: modifying the workflow file, adding new CI jobs, changing Python scripts
```

---

## Review Findings

### Issues Addressed
1. **[Warning] Commit SHA hardcoded** — Replaced "commit 833c331" with "committed in Session 11" since only the file path matters
2. **[Warning] Failure injection underspecified** — Added concrete mechanism: modify `aggregate_score_recommendation.json` to corrupt a margin value or remove a scenario entry, triggering T4 coverage failure
3. **[Warning] Missing permissions failure handling** — Added stop-and-report guidance for 403 errors in Step 1
4. **[Warning] Missing polling strategy** — Added `gh pr checks <PR_NUMBER> --watch` command to Steps 2 and 3
5. **[Warning] Passing PR should verify merge-ability** — Added `gh pr view --json mergeStateStatus` check to Step 2, making the proof bidirectional (merge allowed when passing, blocked when failing)
6. **[Suggestion] Exact `gh api` payload** — Incorporated complete API call template in Step 1 to eliminate trial-and-error

### Remaining Suggestions
- Session dating inconsistency in FINDINGS.md is inherited from prior sessions, not this prompt's concern
- Bead management section could be condensed — kept for consistency with prompt #21 pattern
- NARROWING section and END GOAL checklist praised as strong by reviewer

## Usage Notes

- **Best used with:** Claude Code or any agentic coding assistant with `gh` CLI access, file editing, git, and bash
- **Adjust for:** Different repo owner/name in `gh api` calls; different failure injection if `aggregate_score_recommendation.json` structure has changed since Session 10
- **Prerequisites:** Admin access to the GitHub repository (required for branch protection API), `gh` CLI authenticated with appropriate scopes
