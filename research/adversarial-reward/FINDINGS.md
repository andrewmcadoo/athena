# Adversarial Reward Function: Epistemic Information Gain

## Research Question

How should "epistemic information gain within a bounded, deterministic subspace" be formalized as a reward function for the Adversarial Experiment Designer? The formalization must satisfy three competing constraints: it must drive the adversary toward experiments that maximally stress-test hypotheses (information-seeking), it must exclude experiments in stochastic/unlearnable regions that trigger Noisy TV degeneration (noise-avoiding), and it must remain computationally tractable when evaluated via forward simulation against the causal DAG (efficiency). Success criteria: a formal reward function specification with defined behavior across the conservatism-vs-boundary-seeking failure spectrum, validated against at least one synthetic scenario. This investigation blocks adversarial experiment design.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.4 (Adversarial Experiment Designer) | Component definition — maximizes expected epistemic gain within bounded subspaces |
| ARCHITECTURE.md | 5.4 (Adversarial Calibration Feedback) | Calibration loop: predicted vs. actual surprise, persistent miscalibration triggers |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: Medium. Reward contract is specified; implementation fidelity and monitoring remain |
| VISION.md | Section 4.2 (Adversarial Experiment Design) | Bounded active learning, Noisy TV constraint |
| VISION.md | Section 6.2 (Noisy TV Problem) | Pathological adversarial design failure modes |
| VISION.md | Open Question #3 | "Tuning the Adversarial Reward Function" — must penalize unlearnable stochasticity |
| Constraint | Bounded Adversarial Design | Non-negotiable: adversary restricted to deterministic, domain-valid subspaces |

## Status

IN PROGRESS

## Key Definitions

- **Epistemic information gain**: The expected reduction in uncertainty about the causal DAG structure resulting from an experiment's outcome. Formally related to KL divergence between prior and expected posterior beliefs.
- **Bounded subspace**: The deterministic, domain-valid region of experiment parameter space defined by the DSL Environment Interface's constraint specifications. The adversary cannot propose experiments outside this region.
- **Noisy TV degeneration**: A failure mode where a surprise-maximizing agent becomes fixated on stochastic, unlearnable noise because such noise consistently yields high prediction error, producing no epistemic value.
- **Conservatism failure**: A failure mode where the reward function is over-constrained, causing the adversary to select safe, uninformative experiments that do not stress-test hypotheses.
- **Boundary-seeking failure**: A failure mode where the adversary clusters experiments at the edge of the valid subspace without yielding theoretical insight, exploiting constraint boundaries rather than probing causal structure.

## Investigation Log

### 2026-02-23 -- Session 17: Break-Glass Outage Resilience (Tabletop, Read-Only)

**Scope**

- Close the outage-mode documentation gaps identified in Session 16 by extending GOVERNANCE.md Section 3 with explicit fallback paths for API outage and full control-plane outage.
- Reconcile outage taxonomy so "UI down, API up" is treated as Mode A because the break-glass default path is API-only.
- Run a documentation-only tabletop walkthrough for Modes A, B, and C (no live GitHub settings mutations, no live `audit.sh` execution).
- Update accumulated findings and current open threads based on Session 17 outcomes.

**Method**

- Read `research/adversarial-reward/governance/GOVERNANCE.md` in full and re-read the top of this file (Accumulated Findings plus latest Session 16 log entry) before editing.
- Inserted four new Section 3 subsections between "Restore failure fallback" and "Required post-incident FINDINGS entry":
  - `3.1 Outage Mode Definitions`
  - `3.2 Mode B — API Down, Web UI Up`
  - `3.3 Mode C — Both Down (Containment)`
  - `3.4 Operator Decision Tree`
- Added per-mode evidence requirements (JSON backup vs screenshots vs outage log + timestamps), explicit time windows, and post-action obligations.
- Performed tabletop walkthrough by tracing documented operator actions only, with phase-level timing estimates.

**Findings**

- GOVERNANCE.md Section 3 now has explicit outage-mode coverage:
  - Mode A: API/UI available, default API path, max 1 hour override window.
  - Mode B: API unavailable, UI available; screenshot-based backup/restore workflow with `Include administrators` (`enforce_admins`) toggle control and deferred `audit.sh` within 1 hour of API recovery.
  - Mode C: API/UI unavailable; containment-only hold with merge freeze, `/tmp/governance-outage-$TS.txt` outage log, 15-minute probe loop, 2-hour escalation threshold to GitHub Support, and recovery handoff to Mode A/B when either path returns.
- Outage taxonomy reconciliation is now documented in Section 3.1:
  - "UI down, API up" is intentionally collapsed into Mode A because the current break-glass procedure is API-driven end-to-end.
- Operator decision tree is now explicit and time-bounded:
  - API probe first -> Mode A if available.
  - If API unavailable, UI probe -> Mode B if available.
  - If both unavailable -> Mode C containment (max 2 hours, 15-minute checks, escalate after 2 hours).
- Tabletop drill results (documentation-only):
  - Mode A walkthrough remains coherent in the new structure; Session 16 timing anchor still applies at roughly 6 minutes total excluding emergency fix implementation.
  - Mode B walkthrough sequence is complete (`before screenshot -> uncheck Include administrators -> emergency change -> re-check Include administrators -> restore screenshot -> deferred audit`) with estimated operator-active time of ~5-8 minutes excluding emergency fix time and outage wait.
  - Mode C walkthrough sequence is complete (`freeze merges -> record outage start -> 15-minute monitor loop -> recover via A/B`) with initial containment setup ~2-3 minutes and each monitor cycle ~1-2 minutes.
- Realism assessment of containment windows:
  - 1-hour override windows for Modes A/B are consistent with existing tabletop timing and mandatory restore/audit tasks.
  - 2-hour containment and 15-minute polling are policy defaults rather than empirically calibrated values; calibration requires timestamped data from future outages or an explicitly approved sandboxed live drill.

**Implications**

- Session 16 governance open threads for UI fallback and full outage contingency are now resolved by documented Mode B and Mode C procedures.
- Section 3 now provides a complete operator path for all API/UI availability combinations with required evidence artifacts per mode.
- Remaining uncertainty is operational calibration under live conditions, not missing procedural branches.

**Open Threads**

- Optional: run a separately approved sandboxed live break-glass drill to validate Mode B/Mode C timing assumptions and calibrate 2-hour/15-minute thresholds with empirical evidence.

### 2026-02-23 -- Session 16: Audit Automation, Break-Glass Tabletop, and Escalation Thresholds

**Scope**

- Audit automation script for C1-C7 as a read-only governance operation.
- Documentation-only break-glass drill walkthrough (no live mutations).
- Escalation threshold insertion and severity validation against the governance evidence chain.

**Method**

- Authored `research/adversarial-reward/governance/audit.sh` as executable bash using only `gh`, `jq`, `date`, `git`, and shell built-ins.
- Implemented C1-C7 with pass/fail checks against the locked Section 2 expectations, filled audit-evidence template output for live runs, `0/1` exit semantics (`0` all pass, `1` any fail), and `--dry-run` command-print mode (no check execution, no evidence template, always exit `0`).
- Validated script by running:
  - `bash research/adversarial-reward/governance/audit.sh`
  - `bash research/adversarial-reward/governance/audit.sh --dry-run`
- Executed a tabletop-only break-glass walkthrough of GOVERNANCE.md Section 3 with simulated trigger and phase timing estimates; no override commands were executed.
- Added pre-authored escalation threshold table to GOVERNANCE.md Section 2 (after Audit Cadence) and verified each severity assignment against the Session 11-13 governance evidence chain in GOVERNANCE.md Section 1.
- Reviewed Sessions 11-14 prior open threads to check whether this session newly closes any remaining thread.

**Findings**

- Audit automation live validation passed all checks (`7/7`) with exit code `0` and populated evidence template output:
  - Date (UTC): `2026-02-23T20:19:58Z`
  - Auditor: `aj`
  - Commit SHA observed: `02c67aa9db4800d427aa39dd74b7fdbbae17a803`
  - Results: C1-C7 all `PASS`
- Dry-run validation behaved as required:
  - Printed exactly 7 command strings (C1-C7) with resolved `OWNER/NAME`.
  - Did not print governance evidence template.
  - Exited `0`.
  - Did not evaluate pass/fail checks (command print-only mode).
- Tabletop break-glass drill narrative (documentation-only):
  - Simulated trigger scenario: CI contract gate (`.github/workflows/contract-gate.yml`) is accidentally disabled during GitHub incident response and urgent remediation is required.
  - Phase 1 backup capture (est. 30s): operator would run repo resolution + protection backup command from Section 3 step 1 (`gh api .../protection > /tmp/master-protection-backup-$TS.json`).
  - Phase 2 override (est. 30s): operator would generate break-glass payload with `jq` and apply `gh api .../protection --method PUT --input ...` from Section 3 step 2.
  - Phase 3 emergency change (variable duration): operator would apply the urgent fix (for this scenario, re-enable workflow and verify state).
  - Phase 4 restore + re-audit (est. 5 min): operator would restore `enforce_admins=true`, run full 7-check audit, and close remediation bead per guardrails.
- Tabletop gap analysis:
  - The procedure explicitly documents web UI fallback only for restore failure (GOVERNANCE.md line 236), not for backup-capture or override-apply steps.
  - If `gh api` is unavailable but web UI is reachable, restoration can be done manually, but the runbook lacks explicit manual parity instructions for backup fidelity and override payload reconstruction.
  - If both API and UI are unavailable, the current runbook has no documented fallback path.
- Escalation-threshold validation against governance evidence chain (Section 1) found no required severity changes:
  - `C2`, `C4`, `C5` marked `CRITICAL` are consistent with demonstrated bypass-surface risks from Sessions 12-13 and protected-history integrity baseline requirements.
  - `C3` and `C6` marked `HIGH` are consistent with degraded governance posture without immediate full bypass.
  - `C7` marked `MEDIUM` is consistent with execution anomaly handling while governance controls remain configured.
- Review of Sessions 11-14 open threads: no thread was newly closable by Session 16 changes because all previously listed governance open threads had already been closed in Sessions 12, 13, and 15.

**Implications**

- Governance verification is now automatable via a single read-only command path with deterministic pass/fail and evidence emission.
- Break-glass handling now has a documented tabletop timing model (backup, override, emergency change, restore/re-audit) for operator planning.
- Failure-severity triage is now codified per check, reducing ambiguity in incident response prioritization.

**Open Threads**

- Add explicit web-UI fallback instructions for break-glass backup and override steps (not only restore).
- Define contingency procedure for full GitHub control-plane outage (API and UI both unavailable).
- Optionally run a sandboxed live break-glass drill only if explicitly approved for live mutation.

### 2026-02-23 -- Session 15: First Live Governance Audit Drill and Cadence Policy

**Scope**

- Execute the 7-check governance audit runbook (GOVERNANCE.md Section 2) end-to-end as a fire drill.
- Validate that the runbook is followable by an operator with no prior session memory.
- Define a lightweight audit cadence and record it in GOVERNANCE.md.
- Log all raw command outputs and per-check pass/fail results.

**Method**

- Read GOVERNANCE.md in full before starting any commands.
- Executed each runbook command verbatim in the order specified by Section 2 (steps 1-4).
- Compared raw output against expected values from the pass/fail summary table (lines 149-157).
- Captured UTC timestamp (`2026-02-23T19:25:09Z`) and commit SHA (`20106236ee112ef64a122149ff33affd935c5145`) for the evidence record.
- Populated the audit evidence template (lines 159-175) with live results.
- Added an "Audit Cadence" subsection to GOVERNANCE.md Section 2 defining weekly, pre-merge, and post-incident audit triggers.
- Updated GOVERNANCE.md "Last updated" line to reflect Session 15.

**Findings**

All 7 checks passed with no discrepancies:

| Check | Command | Raw Output | Expected | Result |
| :--- | :--- | :--- | :--- | :--- |
| C1 | `gh repo view --json nameWithOwner --jq '.nameWithOwner'` | `andrewmcadoo/athena` | `andrewmcadoo/athena` | PASS |
| C2 | `gh api .../protection --jq '.required_status_checks.contexts'` | `["contract-verification"]` | `["contract-verification"]` | PASS |
| C3 | `gh api .../protection --jq '.required_status_checks.strict'` | `true` | `true` | PASS |
| C4 | `gh api .../protection --jq '.enforce_admins.enabled'` | `true` | `true` | PASS |
| C5 | `gh api .../protection --jq '.allow_force_pushes.enabled'` | `false` | `false` | PASS |
| C6 | `gh api .../actions/workflows --jq '...contract-gate.yml... \| .state'` | `active` | `active` | PASS |
| C7 | `gh api .../contract-gate.yml/runs?per_page=1 --jq '...\| {status, conclusion}'` | `{"conclusion":"success","status":"completed"}` | `{"status":"completed","conclusion":"success"}` | PASS |

Completed Governance Audit Record:

> ### Governance Audit Record
> - Date (UTC): 2026-02-23T19:25:09Z
> - Auditor: Claude Code / Session 15
> - Repo: andrewmcadoo/athena
> - Commit SHA observed: 20106236ee112ef64a122149ff33affd935c5145
> - C1 Repo identity: PASS
> - C2 Required contexts: PASS
> - C3 Strict mode: PASS
> - C4 Admin enforcement: PASS
> - C5 Force pushes disabled: PASS
> - C6 Workflow active: PASS
> - C7 Latest run success: PASS
> - Notes: First live execution of the audit runbook as a fire drill. All checks passed on first attempt with no operator confusion or ambiguity in instructions. Runbook is followable end-to-end in under 5 minutes.

Audit cadence policy added to GOVERNANCE.md:
- Weekly spot-check (update `Last audited` date on pass; log full record on failure).
- Mandatory pre-merge audit for PRs touching `aggregation-candidates/` or `.github/workflows/`.
- Post-incident audit after any break-glass procedure (already required by Section 3).

**Implications**

- The runbook is confirmed followable end-to-end by an operator executing commands verbatim, with no ambiguity or missing steps. Total wall-clock time was under 5 minutes.
- All governance controls remain intact and undrifted since Session 14 codification.
- Audit cadence is now defined, closing the open thread from Session 14 ("Define a lightweight audit cadence").

**Open Threads**

- None. Cadence is defined; runbook is validated. Future audits follow the established cadence policy.

### 2026-02-23 -- Session 14: Governance Audit Runbook + Break-Glass Procedure Codification

**Scope**

- Convert the Session 11-13 governance chain into a repeatable, operator-friendly audit runbook.
- Verify live branch-protection state before codifying a baseline (source of truth precedence over handoff text).
- Define a constrained break-glass procedure with explicit restoration guardrails.
- Update project governance index (`CLAUDE.md`) to include the new governance artifact.

**Method**

- Re-read this investigation log (including Session 13) before changes.
- Resolved live repository identity: `gh repo view --json nameWithOwner --jq .nameWithOwner` -> `andrewmcadoo/athena`.
- Queried live protection state from GitHub API:
  - `gh api repos/andrewmcadoo/athena/branches/master/protection`
  - Extracted critical fields and normalized baseline object used for documentation.
- Queried workflow state and latest run:
  - `gh api repos/andrewmcadoo/athena/actions/workflows` (selected `.github/workflows/contract-gate.yml`)
  - `gh api repos/andrewmcadoo/athena/actions/workflows/contract-gate.yml/runs?per_page=1`
- Authored `research/adversarial-reward/governance/GOVERNANCE.md` with:
  - Must-stay-true baseline JSON + critical-field rationale table.
  - Five-minute audit runbook with 7 explicit checks (exact commands, jq queries, expected outputs, pass/fail indicators).
  - Break-glass procedure with backup, constrained override, restore + re-audit requirements, and non-examples.
  - Known-issues list seeded with `bd` panic note.
- Checked `bd list --status=open` during this session to confirm prior panic was non-reproducing.

**Findings**

- Live protection state matches Session 13 governance baseline (no discrepancy detected):
  - `required_status_checks.contexts = [\"contract-verification\"]`
  - `required_status_checks.strict = true`
  - `enforce_admins.enabled = true`
  - `allow_force_pushes.enabled = false`
- The runbook now codifies 7 auditable checks with deterministic pass/fail criteria and an evidence template for future audits.
- CI governance remains active:
  - Workflow `.github/workflows/contract-gate.yml` state is `active`.
  - Most recent run was `completed/success` at capture time (`run_id=22295738839`).
- `bd list --status=open` did not reproduce the Session 13 panic in Session 14.

**Implications**

- Governance verification is now operationalized as a repeatable five-minute procedure, reducing dependence on session memory.
- Break-glass handling now has explicit constraints (self-approval with mandatory documentation, one-hour max window, mandatory restore + re-audit), reducing policy-drift risk during emergency operations.
- Because live API verification was used as baseline input, this runbook can be treated as current-state authoritative for the AggregateScore governance contract.

**Open Threads**

- Define a lightweight audit cadence (for example weekly or pre-release) and record it in future session planning once operational load is understood.

### 2026-02-22 -- Session 13: Admin Bypass Closure and Direct-Push Rejection Proof

**Scope**

- Close the remaining governance bypass where repository administrators could push directly to `master` despite required status checks.
- Preserve existing branch-protection semantics while enabling admin enforcement.
- Produce empirical evidence that direct pushes are rejected and PR-based flow remains functional.
- Clean up leftover Session 12 proof PRs/branches and record all evidence in this log.

**Method**

- Resolved repository identity and permissions using `gh`:
  - `gh repo view --json owner,name` -> `andrewmcadoo/athena`
  - `gh api repos/andrewmcadoo/athena/collaborators/andrewmcadoo/permission --jq .permission` -> `admin`
- Captured full pre-change branch protection via:
  - `gh api repos/andrewmcadoo/athena/branches/master/protection`
  - Verified `enforce_admins.enabled=false`.
- Replayed full protection state with a single mutation (`enforce_admins=true`) using `PUT repos/andrewmcadoo/athena/branches/master/protection`, preserving all other fields from the captured state.
- Proved direct-push rejection:
  - Created disposable branch `ci-proof/direct-push`.
  - Committed harmless proof artifact.
  - Attempted `git push origin ci-proof/direct-push:master`.
- Proved PR path still works under admin enforcement:
  - Created disposable branch `ci-proof/pr-path`.
  - Opened PR `#5`: `https://github.com/andrewmcadoo/athena/pull/5`.
  - Watched checks with `gh pr checks 5 --watch`, captured run metadata and mergeability.
  - Closed PR `#5` (unmerged) and deleted remote branch.
- Cleaned Session 12 leftovers:
  - Closed PRs `#3` and `#4`.
  - Deleted remote branches `session12-pass-proof-v2` and `session12-fail-proof-v2`.
  - Verified legacy candidates `ci-proof/pass` and `ci-proof/fail` were already absent.

**Findings**

- Branch protection before Session 13 change:
  ```json
  {
    "required_status_checks": {
      "strict": true,
      "contexts": ["contract-verification"]
    },
    "enforce_admins": false,
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
- Branch protection after Session 13 change:
  ```json
  {
    "required_status_checks": {
      "strict": true,
      "contexts": ["contract-verification"]
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
- Normalized before/after diff confirmed only one changed field: `enforce_admins.enabled` (`false -> true`).
- Direct push to `master` was rejected with protected-branch enforcement:
  - `remote: error: GH006: Protected branch update failed for refs/heads/master.`
  - `remote: - Required status check "contract-verification" is expected.`
  - `! [remote rejected] ci-proof/direct-push -> master (protected branch hook declined)`
- PR path remained functional with admin enforcement enabled:
  - PR: `#5` (`https://github.com/andrewmcadoo/athena/pull/5`)
  - `contract-verification`: `SUCCESS` (`COMPLETED`)
  - Actions run ID: `22295647030`
  - `mergeStateStatus`: `CLEAN`
  - PR was intentionally closed (not merged) after evidence capture; branch `ci-proof/pr-path` deleted.
- Session 12 cleanup is complete:
  - PR `#3` state: `CLOSED`
  - PR `#4` state: `CLOSED`
  - Remote branches deleted: `session12-pass-proof-v2`, `session12-fail-proof-v2`.

**Implications**

- Branch-protection governance is now airtight against administrator direct-push bypass on `master`.
- Required checks now gate both standard collaborators and administrators on the protected branch.
- The governance chain is now complete and empirically evidenced end-to-end:
  - Session 11: workflow gate implementation
  - Session 12: required-check merge enforcement proof
  - Session 13: admin bypass closure + direct-push rejection proof

**Open Threads**

- None for this policy-closure scope. Future changes to branch protection should replay full protection state as done here to avoid accidental weakening.

### 2026-02-23 -- Session 12: Branch Protection Enforcement Proof + Exit-Code Integrity Fix

**Scope**

- Enforce `contract-verification` as a required branch-protection status check on `master`.
- Produce proof evidence with one passing PR and one deliberately failing PR, confirming merge-block behavior.
- Resolve and document any enforcement gaps discovered during proof execution.

**Method**

- Configured branch protection via GitHub API:
  - Enabled protection on `master`.
  - Required status checks: `contract-verification` (strict mode on).
- Ran proof PRs:
  - Pass proof PR: `#3` (`session12-pass-proof-v2`) with harmless marker file change.
  - Fail proof PR: `#4` (`session12-fail-proof-v2`) with deliberate baseline drift in `aggregate_score_recommendation.json` (`S2_unanimous_weak_signal` set to `1e-06`).
- During initial proof attempts (`#2` before fix), observed false-positive success despite local failing acceptance checks.
- Root-caused CI masking to shell pipeline semantics: `python ... | tee ...` without `set -o pipefail` returns `tee` exit status.
- Patched `.github/workflows/contract-gate.yml` to add `set -o pipefail` in both script execution steps so step outcomes reflect Python exit codes.
- Captured evidence artifacts from GitHub Actions run metadata and merge-attempt behavior.

**Findings**

- Branch protection is now active with required status check context `contract-verification` on `master`.
- Pass proof succeeded:
  - PR: `https://github.com/andrewmcadoo/athena/pull/3`
  - Run: `https://github.com/andrewmcadoo/athena/actions/runs/22295089577`
  - Conclusion: `success`
  - Artifact: `contract-gate-output` (`id=5612369204`)
- Fail proof succeeded (as an enforcement test):
  - PR: `https://github.com/andrewmcadoo/athena/pull/4`
  - Run: `https://github.com/andrewmcadoo/athena/actions/runs/22295105551`
  - Conclusion: `failure` (job `contract-verification`)
  - Artifact: `contract-gate-output` (`id=5612374661`)
- Merge-block was verified:
  - `gh pr merge 4 --merge` returned non-zero (`MERGE_EXIT=1`) with message: base branch policy prohibits merge.
- Discovered and fixed a load-bearing CI defect:
  - Without `pipefail`, failing Python checks were masked by `tee`, causing false `success`.
  - With `pipefail`, failing checks correctly propagate non-zero and fail the job.

**Implications**

- Contract enforcement is now both configured and empirically validated: required check policy plus demonstrated pass/fail behavior.
- The Session 11 workflow contract is now semantically correct under failure conditions (no false green due to pipeline masking).
- AggregateScore drift attempts in PRs are now reliably blocked by policy unless checks pass or admin override is used.

**Open Threads**

- Optionally close proof PRs `#3` and `#4` after evidence retention window decisions.
- If direct pushes to `master` should also be blocked, tighten branch-protection rules to disallow bypass paths currently available to admins.

### 2026-02-22 -- Session 11: CI Gate for AggregateScore Contract Enforcement

**Scope**

- Wire `acceptance_test.py` (contract checks) and `monitoring_hooks.py` (drift checks) into GitHub Actions as an unconditional pre-merge gate for `master`.
- Enforce run-both-then-gate semantics so both scripts execute each run and the workflow fails if either returns non-zero.
- Persist script stdout as downloadable CI artifacts for post-failure debugging.

**Method**

- Added `.github/workflows/contract-gate.yml` with:
  - Trigger: `push` and `pull_request` on `master` (no path filters).
  - Single job: `contract-verification` on `ubuntu-latest` using `actions/setup-python@v5` with Python `3.12`.
  - `defaults.run.working-directory` set to `research/adversarial-reward/prototypes/aggregation-candidates`.
  - `acceptance` step: `python acceptance_test.py | tee "$GITHUB_WORKSPACE/acceptance-output.txt"`.
  - `monitoring` step: `if: always() && steps.acceptance.outcome != 'cancelled'` with `python monitoring_hooks.py | tee "$GITHUB_WORKSPACE/monitoring-output.txt"`.
  - Artifact step: upload both output files as `contract-gate-output` via `actions/upload-artifact@v4` with `retention-days: 30`.
  - Final gate step: compares `steps.acceptance.outcome` and `steps.monitoring.outcome` and exits `1` unless both are `success`.
- Executed local verification from `research/adversarial-reward/prototypes/aggregation-candidates`:
  - `python acceptance_test.py`
  - `python monitoring_hooks.py`

**Findings**

- Local acceptance suite passed: `15/15 passed`, exit code `0`.
- Local monitoring suite passed: `20/20 checks passed`, exit code `0`.
- Workflow now runs both verification scripts on every `push`/`pull_request` targeting `master` without fail-fast short-circuiting.
- CI output capture is in place: both script stdout files are uploaded in artifact `contract-gate-output` (30-day retention).
- Contract drift in either suite now deterministically fails CI through the explicit final gate step.

**Implications**

- AggregateScore contract enforcement moved from manual/local verification to automated repository-level governance.
- Debuggability is improved for regressions: CI failures include downloadable script transcripts for fast root-cause review.
- This closes the previously open integration thread from Sessions 9 and 10 to wire acceptance and monitoring checks into CI.

**Open Threads**

- Repository settings should mark `contract-verification` as a required status check for branch protection on `master` (out of scope for in-repo code changes).

### 2026-02-23 -- Session 10: Continuous Monitoring Hooks for Locked AggregateScore Contract

**Scope**

- Implement continuous monitoring hooks for the locked AggregateScore contract in `aggregation-candidates`.
- Instrument revisit triggers `T1`-`T5` as executable checks and add contract metadata validation.
- Produce a deterministic pass/fail monitoring report runnable via `python monitoring_hooks.py`.

**Method**

- Added `research/adversarial-reward/prototypes/aggregation-candidates/monitoring_hooks.py` (stdlib-only, read-only instrumentation).
- Implemented trigger checks mapped to the monitoring trigger contract:
  - `T1`: operating-envelope parity/range checks (`BF_NORM_LOG_SCALED_C`, default absolute-difference sigmoid, custom sigmoid `k/x0` envelope).
  - `T2`: `DivergenceKind` coverage audit against the locked 6-kind normalization dispatch surface.
  - `T3`: Pattern B accepted-limitation metadata check using JSON-driven `observed_value`/`threshold` and `classification`.
  - `T4`: scenario fixture coverage check for locked baseline indices `{1..7}` (no additions/missing entries).
  - `T5`: correlation envelope validation (`max_inflation < 1.5`) plus optional runtime `inflation_ratio` scan from `correlation_results.json`.
- Added contract metadata checks for `version`, `status`, `bf_norm_c` code/JSON parity, `n_terms`, and guardrail enforcement mode.
- Executed:
  - `cd research/adversarial-reward/prototypes/aggregation-candidates`
  - `python monitoring_hooks.py`

**Findings**

- Monitoring run result: `20/20 checks passed`, exit code `0`.
- All trigger groups (`T1`-`T5`) passed with no trigger alerts.
- Contract metadata checks passed:
  - `version=1.0`
  - `status=LOCKED`
  - `bf_norm_c=0.083647` parity with code constant
  - `n_terms=1`
  - guardrail `GR-S2-CUSTOM-SIGMOID-X0-NONNEG` enforcement set to `reject_at_config_construction`.
- Report footer is deterministic and anchored to lockfile date (`2026-02-22`), not wall-clock time.

**Implications**

- AggregateScore contract protection now has continuous executable monitoring, complementing Session 9's one-time acceptance proof.
- Drift in envelope assumptions, enum coverage, scenario catalog, correlation behavior, or lock metadata now fails fast with explicit trigger-tagged action summaries.
- This reduces risk of silent contract erosion between research sessions and future implementation integration work.

**Open Threads**

- Integrate `monitoring_hooks.py` into CI/pre-merge gates so trigger checks run automatically.
- Wire production telemetry inputs so T1/T3/T5 checks can consume live operational diagnostics rather than research artifacts only.

### 2026-02-23 -- Session 9: Automated Acceptance Regression Gates for Locked AggregateScore Contract

**Scope**

- Implement a standalone acceptance regression suite for `aggregation-candidates` that automates Session 8's manual verification contracts.
- Enforce three contracts as executable gates: margin parity (7 scenarios), guardrail rejection (`GR-S2-CUSTOM-SIGMOID-X0-NONNEG`), and hybrid decomposition invariant stability.
- Keep the suite stdlib-only and runnable as `python acceptance_test.py` from the prototype directory.

**Method**

- Added `research/adversarial-reward/prototypes/aggregation-candidates/acceptance_test.py`.
- Implemented three function-level tests plus a `main()` runner:
  - `test_margin_parity()` reads `aggregate_score_recommendation.json::baseline_margins`, evaluates all fixtures with locked hybrid config (`NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)`), and checks `abs(margin - expected) <= 1e-6`.
  - `test_guardrail_rejection()` constructs `NormalizationConfig(custom_sigmoids={"test": SigmoidParams(k=2.0, x0=-0.2)})` and asserts `ValueError` contains `GR-S2-CUSTOM-SIGMOID-X0-NONNEG`.
  - `test_decomposition_invariant()` directly calls `aggregate_hybrid(...)` over every dataset in each fixture and verifies no `RuntimeError` from the decomposition invariant assertion.
- Executed:
  - `cd research/adversarial-reward/prototypes/aggregation-candidates`
  - `python acceptance_test.py`

**Findings**

- Acceptance suite result: `15/15 passed`, exit code `0`.
- Margin parity passed all seven scenarios against JSON baselines with max absolute delta `4.414e-07` (within `1e-6` tolerance).
- Guardrail behavior is correct: invalid custom sigmoid midpoint (`x0=-0.2`) is rejected at config construction with the expected guardrail ID in the error message.
- Decomposition invariant held for every scenario dataset; no runtime decomposition discrepancies were observed.

**Implications**

- Session 8 manual contract verification is now codified as an executable regression gate in-repo.
- The locked AggregateScore recommendation now has a deterministic, runnable acceptance harness suitable for CI integration or release checklists.
- Any future regression in baseline margins, guardrail enforcement, or decomposition accounting will fail fast with scenario-level diagnostics.

**Open Threads**

- Wire `acceptance_test.py` into an automated pre-merge/CI gate so contract checks run continuously.
- Extend acceptance metadata capture if future recommendation versions change locked constants or scenario definitions.

### 2026-02-22 -- Session 8: BF Normalization Seam + Contract Enforcement Implementation

**Scope**

- Implement three locked-recommendation changes in `aggregation-candidates/`: BF normalization seam (athena-4xm), x0 >= 0 config-time guardrail (athena-8b9), decomposition invariant assertion (athena-fgo).
- Verify against 7-scenario suite with baseline margin parity within 1e-6.

**Method**

- Read in full: `normalization.py`, `ceiling_analysis.py`, `candidates.py`, `aggregate_score_recommendation.json`, Session 7 log entry.
- `normalization.py`: Added `bf_norm_log_scaled(bf, c=0.083647)` with `BF_NORM_LOG_SCALED_C` constant. Added `bf_norm_fn: Callable[[float], float]` field to `NormalizationConfig` (frozen dataclass) with `default_factory=lambda: bf_norm_log_scaled`. Replaced hard-coded BayesFactor branch (`1 - 1/(1+bf)`) with `config.bf_norm_fn(bf)`. Non-BF branches untouched.
- `normalization.py`: Added `NormalizationConfig.__post_init__` that iterates `custom_sigmoids` and raises `ValueError` with guardrail ID `GR-S2-CUSTOM-SIGMOID-X0-NONNEG` if any `x0 < 0`. No silent clamping.
- `ceiling_analysis.py`: Imported `bf_norm_log_scaled` from `normalization` and removed local duplicate definition. Kept `bf_norm_current`, `bf_norm_power_law`, `bf_norm_exp_decay` (research candidates).
- `candidates.py`: Added decomposition invariant at end of `aggregate_hybrid()`: `abs(sum(c.contribution for c in contributions) - aggregate) <= 1e-8`, raising `RuntimeError` if violated.
- Ran `evaluate.py` and margin parity script against `aggregate_score_recommendation.json::baseline_margins`.

**Findings**

- 7/7 scenarios PASS for Hybrid with all three changes applied.
- Margin parity confirmed: max absolute delta across 7 scenarios = 4.414e-07 (well within 1e-6 tolerance).
- Guardrail correctly rejects `NormalizationConfig(custom_sigmoids={"test": SigmoidParams(k=1.0, x0=-0.2)})` with descriptive `ValueError`.
- No `RuntimeError` from decomposition invariant during normal evaluation; S6 margin of 1.000e-08 confirms the invariant is tight.
- The BF normalization seam is fully transparent to existing callers: `candidates.py`, `scenarios.py`, `evaluate.py` required no changes.

**Implications**

- The locked AggregateScore recommendation (athena-6ax) is now implemented at the prototype level with `bf_norm_log_scaled` as the configurable default.
- Contract enforcement (guardrail + decomposition invariant) is in-place; the acceptance test suite (athena-3lu) can now be authored against these seams.
- Ceiling analysis can still be run with all BF norm research candidates since only the duplicate `bf_norm_log_scaled` was removed.

**Open Threads**

- athena-3lu (acceptance test suite): Now unblocked. Should formalize the margin parity checks, guardrail rejection test, and decomposition invariant test into a standalone script.
- athena-i4s (monitoring hooks): Still pending; requires production instrumentation decisions.
- `bf_norm_fn` configurability has not been exercised with non-default callables yet. Future calibration work should test alternate BF norms injected via config.

### 2026-02-23 -- WDK#41 Session 7: Architecture Integration for Locked AggregateScore Contract

**Scope**

- Integrate the locked Session 6 AggregateScore recommendation into architecture-level contracts and handoff artifacts without changing any algorithm decisions.
- Update `ARCHITECTURE.md` to carry a normative AggregateScore contract section and aligned risk posture.
- Create implementation-facing artifacts: ADR, implementation beads, acceptance-test spec, and monitoring-trigger spec.

**Method**

- Read in full: `aggregate_score_recommendation.md/.json`, `regime_validity.md`, `guardrail_spec.md`, `ARCHITECTURE.md` Sections 4.4/5.4/8.1, and this FINDINGS log.
- Added `ARCHITECTURE.md` Section 4.4.1 (locked AggregateScore contract), plus cross-references in Sections 5.4 and 8.1; aligned Appendix priority notes to reflect resolved research formalization and pending implementation.
- Added ADR `decisions/002-aggregate-score-contract.md` documenting decision scope, locked parameters, and evidence basis.
- Added integration specs:
  - `aggregate_score_acceptance_test_spec.md`
  - `monitoring_triggers.md` (T1-T5 source/threshold/owner/action contract)
- Created implementation beads and dependency edges:
  - `athena-4xm` (BF normalization seam)
  - `athena-8b9` (x0>=0 config guardrail)
  - `athena-fgo` (decomposition invariant assertion)
  - `athena-3lu` (acceptance test suite; depends on the three implementation beads)
  - `athena-i4s` (monitoring hooks)

**Findings**

- AggregateScore is now represented in architecture as a locked contract with explicit pipeline, BF normalization, parameter table, guardrail reference, and output invariant (`sum(contribution_i)=aggregate_score`).
- `n_terms=1` is now explicitly documented in architecture as intentional (not placeholder), consistent with Session 6 recommendation note 4.
- Adversarial Experiment Designer risk in `ARCHITECTURE.md` moved from open research framing to "specified, pending implementation and monitoring" with downgraded severity.
- Acceptance and monitoring requirements are now codified as implementation contracts tied directly to Session 6 evidence, including baseline margins (`1e-6`) and decomposition tolerance (`1e-8`).

**Implications**

- Session 6 recommendation is now consumable by downstream implementation without reopening research scope.
- Remaining work is implementation fidelity and operational instrumentation (tracked in `athena-4xm`, `athena-8b9`, `athena-fgo`, `athena-3lu`, `athena-i4s`).
- Any future challenge to constants, normalization family, or guardrail scope should route through revisit triggers (T1-T5), not ad-hoc code changes.

**Open Threads**

- Implement and validate the three locked contract checks (BF seam, guardrail enforcement, decomposition invariant).
- Wire acceptance tests into CI using `aggregate_score_acceptance_test_spec.md`.
- Implement telemetry required by `monitoring_triggers.md` so T1-T5 can be enforced post-deployment.

### 2026-02-22 -- WDK#41 Session 6: AggregateScore Recommendation

**Scope**

- Produce the canonical AggregateScore specification document that downstream ATHENA components can build against, synthesizing all evidence from Sessions 1-5 + 4.1 + 4.2/4.3.
- Lock recommended algorithm, fixed parameters, normalization family, guardrail, operating boundaries, and revisit triggers.
- Close `athena-6ax`.

**Method**

- Read all evidence artifacts in full: `perturbation_summary.md`, `ceiling_analysis.md`, `stretch_summary.md`, `regime_validity.md`, `guardrail_spec.md`, source modules (`candidates.py`, `normalization.py`, `models.py`, `ceiling_analysis.py`), and all FINDINGS.md investigation log entries.
- Cross-referenced each design choice to its originating evidence artifact and key quantitative result.
- Classified all failure modes into resolved risks (with design changes) vs accepted limitations (with operating-range justification).
- Defined five revisit triggers with specific conditions for reopening the recommendation.
- Produced `aggregate_score_recommendation.md` (human-readable specification) and `aggregate_score_recommendation.json` (machine-readable locked parameters) in the aggregation-candidates directory.

**Findings**

- The recommended algorithm is HTG-gated Fisher product hybrid with `n_terms=1` and log-scaled BF normalization (`c=0.083647`, `bf_max_target=10000`).
- All seven baseline scenarios pass with positive margins. Tightest margin: S6 at +0.000000 (exact decomposition boundary); widest: S4 at +0.128007.
- Four risks are resolved by design changes included in the recommendation:
  - S5 BF ceiling (log-scaled normalization)
  - S6 compression failures (same normalization change)
  - S2 custom sigmoid fragility (x0>=0 guardrail)
  - Correlation floor-saturation (resolved in S5 stretch probe)
- Two accepted limitations are documented with out-of-range classification at MEDIUM confidence:
  - L1: Pattern B under-response (step_ratio=1.029, threshold >3.0) — 50x isolated single-metric jump not representative of valid DSL outputs
  - L2: S1 SE multiplier fragility at 5x/10x — exceeds realistic uncertainty inflation band of [0.5, 3.0]
- Five revisit triggers defined: empirical range violation (T1), new DivergenceKind (T2), Pattern B becomes blocking (T3), scenario suite expansion (T4), correlation structure change (T5).

**Implications**

- The adversarial reward aggregation function is now specified at architecture-integration readiness. Downstream sessions can implement `AggregateScore` against this contract.
- The specification includes explicit seam notes: BF normalization should be a first-class configurable hook, guardrail enforcement is schema-level validation, decomposition exactness is a required invariant, and `n_terms=1` is intentional (not a placeholder).
- Pattern B under-response remains a documented open thread (not a blocker) that can be revisited if the calibration loop (ARCHITECTURE.md 5.4) requires sharp step-response detection.

**Open Threads**

- Validate operating-range estimates with empirical DSL trace distributions when production-like data becomes available (T1).
- If Pattern B step-response becomes a calibration loop requirement, investigate targeted hybrid adjustments (T3).
- Consider adding adversarial scenarios beyond S1-S7 for expanded coverage before production implementation (T4).

### 2026-02-22 -- WDK#41 Session 4.2/4.3: Regime Validity Analysis + Guardrail Specification

**Scope**

- Close `athena-17c` (regime validity) and `athena-zvg` (guardrail spec) by classifying Session 4/5 failures against realistic DSL operating ranges.
- Answer the central question per failure mode: in-range risk vs stress-test boundary.
- Produce architecture-level guardrail specification for custom sigmoid midpoint constraints.

**Method**

- Read `perturbation_summary.md`, `stretch_summary.md`, `ceiling_analysis.md`, `normalization.py`, `scenarios.py`, and Session 4/4.1/5 context in this log.
- Derived training-knowledge-informed operating ranges (with confidence tags) for: z-score magnitude, Bayes factor, custom sigmoid `x0/k`, SE multipliers, missing uncertainty count, and abrupt single-metric jump factors across OpenMM/GROMACS, CESM, and VASP contexts.
- Produced `research/adversarial-reward/prototypes/aggregation-candidates/regime_validity.md` and `research/adversarial-reward/prototypes/aggregation-candidates/regime_validity.json` with parameter-range tables, failure overlays, and per-domain Pattern B realism checks.
- Produced `research/adversarial-reward/prototypes/aggregation-candidates/guardrail_spec.md` specifying custom sigmoid `x0 >= 0`, scope, and reject-on-violation enforcement behavior.

**Findings**

- Unresolved failures from Sessions 4/5 are out-of-range stress conditions under the derived DSL operating bands:
  - S2 fragility appears only at negative midpoint (`x0=-0.2`) with higher steepness (`k>=2.0`).
  - S1 fragility appears only at extreme uncertainty inflation (`SE_mult=5.0`, `10.0`).
  - Pattern B under-response is measured under an intentionally extreme 50x isolated one-metric jump (`0.1 -> 5.0`), not representative of nominal valid DSL output behavior.
- S5 BF ceiling and S6 joint compression are resolved via Session 4.1's log-scaled BF normalization (`bf_max_target=10000`), corroborated by Session 5 baseline gate and stretch artifacts.
- Guardrail decision locked: custom sigmoid midpoint must satisfy `x0 >= 0` (hard validation, reject invalid configs, no silent clamping).

**Implications**

- `athena-6ax` is unblocked to proceed with Session 6 recommendation synthesis using explicit accepted-boundary language for S1/Pattern B stress regions.
- The only mandatory new architecture-level control from this session is config-time enforcement of `x0 >= 0` for `custom_sigmoids`.

**Open Threads**

- Validate the estimated operating ranges with empirical DSL trace distributions once production-like data is available.
- If future operating traces regularly enter currently out-of-range regions, re-run regime-validity classification before changing guardrail scope.

### 2026-02-22 -- WDK#41 Session 5: Hybrid Stretch Analyses (Calibration + Correlation)

**Scope**

- Execute post-ceiling stretch validation on the hybrid candidate using the approved BF normalization family (`log_scaled`, `c=0.083647`, `bf_max_target=10000`).
- Re-run Session 2 stretch protocols against the hybrid for direct comparability:
  - calibration patterns A/B/C (50 cycles each)
  - correlation robustness at rho `{0.0, 0.3, 0.5, 0.7, 0.9}` with Brown-style diagnostic correction.
- Enforce a baseline pre-gate against all seven fixtures before any stretch interpretation.

**Method**

- Added `research/adversarial-reward/prototypes/aggregation-candidates/stretch_test.py` as a standalone four-phase harness:
  - **Phase 0**: baseline re-verification via `evaluate_fixture(...)` + `margin_from_cell(...)`; strict pass/label checks, margin comparison to `ceiling_analysis.json` baseline with tolerance `1e-6` (floating-point drift from rounded `c`).
  - **Phase 1**: calibration simulation on S6 fixture using imported `run_pattern_a/b/c` and helper metrics (`spearman_rho`, `pearson_r`, `rank_values`).
  - **Phase 2**: correlation robustness on S6 fixture, 400 samples/rho, `random.seed(42)` set at Phase 2 start; per-sample log evidence extracted as `sum(contribution.diagnostics["log_evidence"])`.
  - **Phase 3**: deterministic artifact generation to `stretch_results.json` + `stretch_summary.md`.
- Executed `python stretch_test.py` and confirmed deterministic JSON behavior across two runs (timestamp excluded).

**Findings**

- Phase 0 gate passed (`7/7` scenarios). All margins remained consistent with post-ceiling baseline directionality and matched labels exactly.
- Phase 1 calibration (hybrid):
  - Pattern A: `spearman_rho=-1.0000`, `max_delta=0.0493` -> **PASS**
  - Pattern B: `step_ratio=1.0290`, `max_delta=0.0282` -> **FAIL** (`non-responsive but smooth`)
  - Pattern C: `pearson_r=-0.9341`, `max_delta~1.25e-7` -> **PASS**
- Key question result (Pattern B): hybrid did **not** recover sudden regime-change responsiveness; it remains smoother than Session 2 Fisher but under threshold on step response.
- Phase 2 correlation (hybrid on S6) resolved Session 2 floor pathology:
  - `floor_saturated=False` at all rho levels (`floor_count=0/400` everywhere)
  - rho=0.5 pass criterion satisfied: `inflation_ratio=1.0035 <= 1.5` and not floor-saturated
  - inflation ratios across rho stayed near 1 (`~1.003-1.048`).

**Implications**

- Post-ceiling hybrid is robust on baseline suite and correlation robustness under non-floor-saturated conditions.
- The prior Session 2 correlation ambiguity (all-floor aggregates) is removed in this S6-based probe, making inflation diagnostics interpretable.
- Hybrid still inherits Pattern B regime-change under-response (low step ratio), so calibration responsiveness remains an unresolved risk despite smoothness.

**Open Threads**

- Investigate whether targeted hybrid adjustments can lift Pattern B `step_ratio` above 3.0 without reintroducing Fisher-like non-smooth jumps.
- Evaluate Pattern B behavior under additional fixture families (beyond S6) to distinguish structural under-response from fixture-specific dynamics.
- Keep S6-based correlation probe as the default correlation harness for non-floor-saturated diagnostics in follow-up sessions.

### 2026-02-22 -- WDK#41 Session 4.1: BF Ceiling Analysis for Hybrid S5 Saturation

**Scope**

- Isolate and quantify the BayesFactor normalization ceiling that drives the S5 failure transition above BF~110 in Session 4 perturbations.
- Test calibrated alternative BF normalization families while preserving all non-BF normalization behavior and full hybrid aggregation mechanics.
- Determine whether S6 failing compression cells are fundamentally decomposition-share failures (vs reconstruction failures) and whether BF normalization changes can provide side-benefits.

**Method**

- Added `research/adversarial-reward/prototypes/aggregation-candidates/ceiling_analysis.py` as a four-phase deterministic analysis harness.
- Reused existing module APIs (`candidates.py`, `evaluate.py`, `scenarios.py`, `perturbation_test.py`, `normalization.py`, `models.py`) and duplicated BF-only post-processing logic in one local hook (`normalize_component_with_alt_bf`) because `normalize_component(...)` does not expose a raw-BF substitution seam.
- Executed `python ceiling_analysis.py` in `research/adversarial-reward/prototypes/aggregation-candidates/`, generating:
  - `ceiling_analysis.json` (machine-readable phase outputs, curves, suite results)
  - `ceiling_analysis.md` (decision-oriented summary with tables and recommendation)
- Enforced sanity gate first: patched hybrid using current BF norm (`1 - 1/(1+BF)`) had to match baseline `aggregate_hybrid(...)` raw scenario scores within `1e-12` across all seven scenarios.

**Findings**

- Sanity gate passed exactly (`max_abs_diff = 0.0` over all 7 scenarios), validating patched-hybrid equivalence under current BF normalization.
- S6 decomposition check on the five known failing cells confirmed all failures are dominant-share failures, not reconstruction failures:
  - `failure_is_dominant_share = True` for all 5
  - `failure_is_recon_error = False` for all 5
- Current BF normalization ceiling was measured at `BF=110` for the strict criterion `score < 0.991` (consistent with the observed BF~111 transition boundary).
- Alternative normalization sweep (15 calibrated candidates):
  - Pre-filter (`score@100 >= 0.3`) passed: `13/15`
  - Full baseline suite on pre-filtered set: `13/13` achieved `7/7`
- Best candidate by ceiling extension while retaining `7/7`: `log_scaled_bfmax_10000`
  - `bf_ceiling = 9999`
  - S5 BF sweep pass at all tested points (`80, 100, 120, 200, 500, 1000`)
  - Positive S5 margins maintained through BF=1000
- S6 side-benefit was positive for multiple candidates; highest-ceiling candidates (`log_scaled_bfmax_10000`, `power_law_bfmax_10000`) recovered all five previously failing S6 compression cells (`dominant_share >= 0.35`).

**Implications**

- The S5 BF ceiling is a normalization artifact, not a hybrid gating/Fisher-combination artifact.
- Replacing only the BF raw normalization mapping can remove the S5 operating-range ceiling while preserving the full 7-scenario baseline behavior.
- Recommendation for athena-e2a decision: adopt a calibrated log-scaled BF normalization with `bf_max_target=10000` (per `ceiling_analysis.md`), which maximizes operating headroom and preserves baseline criteria.

**Open Threads**

- Validate behavior for BF ranges beyond 1000 in additional stress fixtures to ensure ranking stability and avoid over-compression at very large BF.
- Decide whether to lock the target at `10000` (max headroom) or choose a lower calibrated target (e.g., `5000`) for a more conservative transition curve.
- If promoted beyond prototype, refactor normalization to expose a first-class BF raw-score hook and remove the local BF-branch duplication used in this research artifact.

### 2026-02-22 -- WDK#41 Session 4: Hybrid Robustness Under Targeted Fixture Perturbation

**Scope**

- Determine whether Session 3's `Hybrid` `7/7` result is structurally robust or narrowly tuned by running targeted perturbations around known weak margins.
- Stress the two pre-identified likely failure points:
  - S2 custom-sigmoid sensitivity (`s2.custom.1`)
  - S5 upper-bound ceiling pressure near component score `0.991`
- Probe secondary axes (S6 joint compression, S7 boundary SE, S4 missing-uncertainty count, S1 SE scaling, S2 non-custom SE scaling) under fixed baseline Hybrid config.

**Method**

- Added `research/adversarial-reward/prototypes/aggregation-candidates/perturbation_test.py` as a dedicated Session 4 sweep harness.
- Reused existing prototype modules only (`candidates.py`, `evaluate.py`, `scenarios.py`, `normalization.py`, `models.py`) with private scenario helpers (`_metric`, `_summary`, `_no_uncertainty`) for fixture variants.
- Enforced baseline sanity checks before sweeps:
  - S2 margin `((aggregate/max_single)/1.5)-1 = +0.072804`
  - S4 relative delta `= 0.071993`
- Executed `python perturbation_test.py` producing:
  - `perturbation_results.json` (full structured run output)
  - `perturbation_summary.md` (pass-rate matrix, critical grids, tipping points)
- Total perturbation evaluations: `70` (S3 direction-asymmetry axis intentionally dropped as lowest-priority per session constraints).

**Findings**

- Top-level robustness map (pass counts):
  - `s2_custom_sigmoid` (S2): `20/24` pass (83.3%)
  - `s2_non_custom_se_scale` (S2): `5/5` pass (100.0%)
  - `s5_bayes_factor` (S5): `4/9` pass (44.4%)
  - `s6_joint_compress` (S6): `11/16` pass (68.8%)
  - `s7_boundary_se` (S7): `7/7` pass (100.0%)
  - `s4_missing_count` (S4): `4/4` pass (100.0%)
  - `s1_se_mult` (S1): `3/5` pass (60.0%)
- S2 critical-axis tipping behavior:
  - Exact PASS->FAIL transition at `x0=-0.2` between `k=1.5` (PASS) and `k=2.0` (FAIL).
  - Failing combinations were only `{k in [2.0, 2.2, 2.5, 3.0], x0=-0.2}`.
- S5 critical-axis tipping behavior:
  - Exact PASS->FAIL transition between `BF=110` (PASS, margin `+0.000009`) and `BF=120` (FAIL, margin `-0.000736`).
  - All `BF >= 120` failed due to upper-bound pressure (`0.991 - max(component_score) < 0`).
- S6 joint compression tipping behavior:
  - Failures concentrated in high-compression/high-BF corners:
    - `d_mid=3.0,bf_strong=500`
    - `d_mid=3.0,bf_strong=1000`
    - `d_mid=4.0,bf_strong=100`
    - `d_mid=4.0,bf_strong=500`
    - `d_mid=4.0,bf_strong=1000`
- Expected-robust axes held (S2 non-custom SE scale, S4 missing count, S7 boundary SE), but S1 showed an additional fragility under extreme SE inflation:
  - S1 failed at `SE mult = 5.0` and `10.0`.

**Implications**

- Session 3's `7/7` is not a pure one-point fluke, but it is not globally robust either; fragility is concentrated in specific high-leverage parameter regions.
- The dominant structural risks are now empirically localized:
  - S2 custom sigmoid over-aggressiveness when `x0` is shifted negative
  - S5 BayesFactor ceiling overshoot once BF crosses ~`120`
  - S6 decomposition stress when both mid-effect and BF-strong terms are jointly amplified
- Candidate recommendation should include explicit operating constraints and guardrails, not just baseline pass/fail status.

**Open Threads**

- Determine whether S1 failure under large SE multipliers reflects realistic DSL uncertainty regimes or only pathological scaling.
- Test whether small constraint adjustments (without changing core hybrid architecture) can widen S5/S6 safety margins while retaining S2 compounding.
- Extend perturbation map to correlated weak-signal regimes where Fisher-like aggregation is not floor-saturated.

### 2026-02-21 -- WDK#41 Session 3: HTG-Gated Fisher Product Hybrid (n_terms=1)

**Scope**

- Implement one cross-family hybrid candidate that composes HTG-style per-component confidence gating with Fisher product combination (`n_terms=1`) in the existing aggregation prototype.
- Preserve backward compatibility for existing candidates (`IVW-CDF`, `HTG-Max`, `Fisher-UP`) under the default Session 2 evaluator path.
- Validate all seven scenario gates with special focus on S2 compounding, S4 missing-data stability, S6 decomposition reconstruction, and boundedness/finite outputs.

**Method**

- Updated `research/adversarial-reward/prototypes/aggregation-candidates/candidates.py`:
  - Added `HybridConfig(alpha=1.5, tau=5.0, c_floor=0.1, c_missing=0.7, p_eps=1e-12, eps=1e-12)`.
  - Added `aggregate_hybrid()` with this pipeline per component:
    - normalize via `normalize_component(...)` with normalization-level SE dampening left OFF.
    - compute precision via `gate_precision(component, eps)`.
    - confidence rule: `max(c_floor, sigmoid(...))` when precision exists, else `c_missing`.
    - gated score to p-value via `p=max(p_eps,1-gated)`, evidence `-2*log(p)`.
    - aggregate with `chi_square_cdf_even_df(total_log_evidence, n_terms=1)` (product method).
    - exact decomposition weights from `log_evidence_i * (aggregate / sum(log_evidence_j * score_j))` when denominator > `eps`.
  - Registered `"Hybrid"` in `get_candidate_registry(...)` with `hybrid_cfg` parameter.
- Updated `research/adversarial-reward/prototypes/aggregation-candidates/evaluate.py`:
  - Added `HybridConfig` import.
  - Passed `hybrid_cfg=HybridConfig(normalization=normalization)` into `get_candidate_registry(...)`.
- Executed `python evaluate.py` in `research/adversarial-reward/prototypes/aggregation-candidates/`, then read `results.json` for exact metric checks.

**Findings**

- Backward compatibility held for existing candidates:
  - `IVW-CDF`: `5/7`
  - `HTG-Max`: `5/7`
  - `Fisher-UP`: `3/7`
- Hybrid passed all seven scenarios (`7/7`) in the default harness.
- S2 sensitivity numbers (Hybrid):
  - aggregate = `0.9234566367020085`
  - max_single = `0.5738586978538172`
  - ratio = `1.6092056113389201`
  - margin = `(aggregate / (1.5 * max_single)) - 1.0 = +7.280374%`
- S2 threshold-driver detail:
  - `s2.custom.1` is the highest single contributor by normalized score (`0.5817593768418363`) and highest single aggregate (`0.5738586978538172` after confidence gating).
  - Other S2 normalized component scores remain substantially lower (max non-custom = `0.382252125230751`).
- S4 missing-data robustness (Hybrid):
  - relative delta = `0.0719926034986539` (passes `<= 0.20`).
- S6 decomposition exactness (Hybrid):
  - reconstruction error = `1.1102230246251565e-16` (passes `<= 1e-8`).
- Boundedness and finiteness:
  - all Hybrid scenario aggregates and comparator values remained finite and within `[0,1]`.

**Implications**

- The cross-family hybrid resolves the Session 1/2 tradeoff in this fixture set: HTG-style front-end gating suppresses noisy components while Fisher product back-end compounds weak concordant evidence strongly enough to clear S2.
- Keeping normalization-level SE dampening off and avoiding Fisher reliability exponentiation did not prevent robustness; confidence gating alone was sufficient in this run.
- No S2 fallback sweep was required because the default hybrid configuration exceeded the 1.5x threshold with positive margin.

**Open Threads**

- Verify whether `7/7` is stable under broader perturbations (fixture resampling, stronger correlation structure, and alternative uncertainty-missingness patterns).
- Decide whether Session 4 should stress-test this hybrid against calibration-pattern criteria used in Session 2 stretch analyses.
- Determine whether the architecture-facing `AggregateScore` recommendation should now target this hybrid directly or require one additional robustness session.

### 2026-02-22 — WDK#41 Session 2: Structural Fixes + Two-Stage Sweep + Calibration + Correlation Robustness

**Scope**

- Implement Session 2 structural knobs in normalization and candidate aggregators while preserving Session 1 default behavior.
- Run a two-stage sweep using all seven scenarios for every candidate-config combination.
- Execute stretch analyses: deterministic 50-cycle calibration simulation and Fisher-UP correlation robustness with Brown-style correction.
- Update research artifacts and verify backward compatibility (`python evaluate.py`) remains exactly Session 1 with defaults.

**Method**

- Structural changes in `research/adversarial-reward/prototypes/aggregation-candidates/`:
  - `normalization.py`: added optional SE dampening (`se_dampen_enabled`, `se_dampen_k`, `se_dampen_x0`) applied at final score stage using raw `component.value / standard_error`.
  - `candidates.py`:
    - `IVW-CDF`: multiplicity bonus (`multiplicity_bonus_enabled`, threshold, scale).
    - `HTG-Max`: `soft_sum` mode with `soft_sum_boost=2.0` (configurable, default unchanged).
    - `Fisher-UP`: optional SE-aware reliability factor (`se_reliability_*`).
  - `candidates.py`: stabilized `chi_square_cdf_even_df` with recurrence-series evaluation to avoid overflow under large term counts.
- New prototype runners:
  - `sweep.py`: Stage 1 normalization sweep (81 normalization configs x 3 candidates = 243 candidate-configs) + Stage 2 candidate sweeps with best Stage 1 normalization (480 candidate-configs including Fisher isolation).
  - `calibration_sim.py`: deterministic 50-cycle patterns A/B/C with stdlib Spearman and Pearson implementations.
  - `correlation_test.py`: S2-like correlated weak signals at rho `{0.0,0.3,0.5,0.7,0.9}`, Cholesky generation, Brown-style corrected df (capped terms at 1000).
- Compatibility and execution checks:
  - `python evaluate.py` (before and after Session 2 changes) confirmed exact Session 1 matrix.
  - `python sweep.py`, `python calibration_sim.py`, and `python correlation_test.py` completed and wrote artifacts.

**Findings**

- Backward compatibility held exactly with default flags disabled:
  - `IVW-CDF`: 5/7 (S1 FAIL, S2 FAIL)
  - `HTG-Max`: 5/7 (S2 FAIL, S4 FAIL)
  - `Fisher-UP`: 3/7 (S1 FAIL, S2 FAIL, S4 FAIL, S7 FAIL)
- Stage 1 normalization sweep selected `N061` (`abs_diff_k=2000`, `abs_diff_x0=5e-4`, `se_dampen_k=8`, `se_dampen_x0=1`) with `10/21` passes (top by pass-count then avg-pass-score).
- Stage 2 best results (no 7/7 found):
  - `IVW-CDF`: best `2/7` (fails S1,S2,S4,S5,S6) despite multiplicity bonus sweep.
  - `HTG-Max`: best `5/7` (fails S5,S6), strongest overall in Session 2 sweep.
  - `Fisher-UP` main sweep (`se_dampen=True`): best `4/7` (fails S1,S2,S5).
  - `Fisher-UP` isolation (`se_dampen=False`, SE-reliability on): best `5/7` (fails S1,S2), indicating overlap/tension between normalization-level dampening and Fisher reliability scaling.
- S2 sensitivity frontier on 6/7-with-only-S2-fail configs was empty for all three candidates (no qualifying configs), so no feasible multiplier frontier from 1.0 to 2.0 could be established under that criterion subset.
- Calibration simulation (Pattern A/B/C) with best configs:
  - IVW: failed all three patterns (`rho=-0.8728`, `step_ratio=2.9533`, `r=0.0000`).
  - HTG: passed A and C, failed B (`step_ratio=1.0036`).
  - Fisher: passed A and C, failed B on smoothness (step jump too sharp; `max_delta=0.9996`).
- Fisher correlation robustness results:
  - Inflation ratios were near 1.0 across all rhos (`1.0000`, `1.0000`, `1.0000`, `1.0025`, `1.0000` for rho `0.0..0.9`).
  - No flag at rho=0.5 (`inflation_ratio > 1.5` condition not met).
  - In this setup both corrected and uncorrected aggregates were at floor-level (~`1e-12`), limiting interpretability of inflation magnitude.

**Implications**

- Session 2 did not produce a 7/7 candidate within the constrained single-candidate families.
- HTG remains the best single-family performer in overall pass count, but improved S2 compounding still trades off against other scenario gates.
- Fisher behaves better on missing-data/boundary than Session 1 under isolation, but Noisy-TV (S1) and weak-signal compounding (S2) remain unresolved.
- Correlation-inflation risk was not observed in the tested S2-like regime, but this result is confounded by aggregate floor saturation.

**Open Threads**

- Session 3 should focus on cross-family designs (explicitly out of scope for Session 2) because single-family tuning did not reach 7/7.
- Revisit S2 fixture regime for Fisher correlation stress where aggregates are not floor-saturated; otherwise inflation diagnostics are weak.
- Investigate why Session 2 normalization winner degrades IVW/HTG S5-S6 behavior despite helping S1 suppression.

### 2026-02-22 — WDK#41 Session 1: Candidate Aggregation Prototype + Adversarial Stress Test

**Scope**

- Implement three prototype aggregation candidates mapping `Vec<MetricComponent> -> AggregateScore in [0,1]` under contract-preserving dataclasses.
- Enforce direction-aware CDF normalization (`Agreement` inversion, unsigned handling when direction absent/`None` variant).
- Implement uncertainty-aware weighting/gating to test Noisy-TV resistance and calibration decomposability constraints from `ARCHITECTURE.md` Section 5.4.
- Run a full 3x7 stress-test matrix (3 candidates x 7 scenarios), recording raw scores and per-component `(score, weight)` decomposition.

**Method**

- Added throwaway prototype package at `research/adversarial-reward/prototypes/aggregation-candidates/`:
  - `models.py`: contract-mirroring dataclasses/enums for `MetricComponent`, `UncertaintySummary`, `PointUncertainty`, and `EffectDirection` variants (`Contradiction`, `Agreement`, `None`).
  - `normalization.py`: kind-specific normalization to `[0,1]`:
    - `ZScore`, `EffectSize`: `2 * Phi(|x|) - 1` (stdlib `erf` CDF)
    - `BayesFactor`: `1 - 1/(1+BF)`
    - `KLDivergence`: `1 - exp(-kl)`
    - `AbsoluteDifference`: configurable sigmoid
    - `Custom`: required configurable sigmoid by `method_ref`; missing params => metric excluded + warning (no silent defaults).
  - `candidates.py`:
    - C1 `IVW-CDF`: inverse-variance weighted mean with decomposition-friendly normalized weights.
    - C2 `HTG-Max`: confidence-gated per-kind maxima + hard max across kinds (primary variant).
    - C3 `Fisher-UP`: reliability-adjusted p-value transform + Fisher-style chi-square CDF combination.
    - Optional exploratory variant: `HTG-Max` with LogSumExp + re-bounding (`1-exp(-LSE)`).
  - `scenarios.py`: 7 scenario fixtures with explicit comparator datasets where needed.
  - `evaluate.py`: executes 3x7 matrix, writes `results.json` and `results.md`.
- Removed all SciPy dependencies after runtime import failure; replaced with pure stdlib math:
  - `norm.cdf(z) = 0.5 * (1 + erf(z/sqrt(2)))`
  - `chi2.cdf(x, 2N) = 1 - exp(-x/2) * sum((x/2)^k / k!, k=0..N-1)`.

**Findings**

- Boundedness gate passed globally: all primary candidate outputs were finite and in `[0,1]` across all 21 cells.
- Primary 3x7 matrix:

| Candidate | S1 Noisy TV | S2 Unanimous weak signal | S3 Mixed signal | S4 Missing data | S5 Scale heterogeneity | S6 Calibration decomposability | S7 Boundary-seeking |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| IVW-CDF | base=0.6457, doubled=0.8849 (**FAIL**) | agg=0.2698, max1=0.5818, target=0.8726 (**FAIL**) | mixed=0.5171, allC=0.6761, allA=0.3239 (**PASS**) | missing=0.5300, baseline=0.5261, delta=0.007 (**PASS**) | agg=0.8445, scores=[0.9545, 0.9901, 0.589] (**PASS**) | agg=0.8684, recon=0.8684, dom_share=0.657 (**PASS**) | boundary=0.6344, non_boundary=0.7686 (**PASS**) |
| HTG-Max | base=0.1165, doubled=0.0238 (**PASS**) | agg=0.3033, max1=0.3033, target=0.4549 (**FAIL**) | mixed=0.3795, allC=0.3795, allA=0.1774 (**PASS**) | missing=0.1593, baseline=0.3423, delta=0.535 (**FAIL**) | agg=0.5712, scores=[0.9545, 0.9901, 0.589] (**PASS**) | agg=0.9618, recon=0.9618, dom_share=1.000 (**PASS**) | boundary=0.3022, non_boundary=0.5085 (**PASS**) |
| Fisher-UP | base=0.5639, doubled=0.8227 (**FAIL**) | agg=0.0056, max1=0.5818, target=0.8726 (**FAIL**) | mixed=0.4002, allC=0.7134, allA=0.0454 (**PASS**) | missing=0.0662, baseline=0.3250, delta=0.796 (**FAIL**) | agg=0.9914, scores=[0.9545, 0.9901, 0.589] (**PASS**) | agg=0.9784, recon=0.9784, dom_share=0.568 (**PASS**) | boundary=0.9917, non_boundary=0.9917 (**FAIL**) |

- Pass counts:
  - `IVW-CDF`: 5/7
  - `HTG-Max`: 5/7
  - `Fisher-UP`: 3/7
- No candidate passed all seven scenarios in Session 1.
- Calibration decomposition (S6) is now explicit and reconstructs aggregate in all primary candidates, with dominant component identification:
  - Dominant component for all three was `s6.z.strong`; dominance share: IVW `0.657`, HTG `1.000`, Fisher `0.568`.
- Scale heterogeneity ranking stability held across all three candidates (S5): `s5.bf.1 > s5.z.1 > s5.abs.1`.
- Exploratory `HTG-Max-LSE` remained bounded but failed S6 decomposition reconstruction under current decomposition mapping (`recon=0.9180` vs `agg=0.6263`), so it is retained as exploratory only.

**Implications**

- `HTG-Max` is strongest on Noisy-TV resistance in this session (passes S1 and S7), but currently too brittle under missing/partial uncertainty (S4 fail).
- `IVW-CDF` remains attractive for calibration diagnostics and missing-data robustness, but currently fails the Noisy-TV stressor (S1).
- `Fisher-UP` compounds evidence and is calibratable, but in this fixture set it is the least Noisy-TV resistant and most sensitive to missing uncertainty defaults.
- Since no candidate passed all seven criteria, Session 2 should focus on parameter sensitivity and uncertainty-floor tuning before a recommendation in Session 3.

**Open Threads**

- Session 2 (not executed): parameter sweeps for `HTG alpha/tau/c_floor`, `IVW w_default`, and `Fisher n_ref/r_floor`.
- Session 2 (not executed): uncertainty fallback policy ablations for `NoUncertainty` to reduce S4 degradation in HTG/Fisher.
- Session 2 (not executed): criterion sensitivity check for S2 compounding target under bounded aggregators.
- Session 3 (not executed): recommendation and `AggregateScore` type definition for architecture integration.

## Accumulated Findings

### What We Know

- **Break-glass outage-mode coverage is now complete in the governance runbook.** GOVERNANCE.md Section 3 now defines Mode A/Mode B/Mode C, an explicit outage decision tree with time windows, and per-mode evidence requirements (JSON backup, screenshots, outage log + timestamps).
  Evidence: Investigation Log entry `2026-02-23 -- Session 17: Break-Glass Outage Resilience (Tabletop, Read-Only)`.
- **Session 16 outage-gap threads are now resolved in documented procedure text.** The web-UI fallback for backup/override is resolved by Mode B, and the full API+UI outage contingency is resolved by Mode C.
  Evidence: Investigation Log entry `2026-02-23 -- Session 17: Break-Glass Outage Resilience (Tabletop, Read-Only)`.
- **Governance audit is now automatable via a read-only operations script.** `research/adversarial-reward/governance/audit.sh` executes C1-C7, reports PASS/FAIL per check, emits the filled governance evidence template on live runs, and supports `--dry-run` command preview mode without evaluating checks.
  Evidence: Investigation Log entry `2026-02-23 -- Session 16: Audit Automation, Break-Glass Tabletop, and Escalation Thresholds`.
- **Escalation thresholds are now explicitly codified per governance check.** GOVERNANCE.md Section 2 now defines severity, failure meaning, owner action, and response-time targets for C1-C7, with severity assignments validated against the Session 11-13 governance evidence chain.
  Evidence: Investigation Log entry `2026-02-23 -- Session 16: Audit Automation, Break-Glass Tabletop, and Escalation Thresholds`.
- **The governance audit runbook has been executed end-to-end as a live fire drill with 7/7 checks passing.** All branch-protection and CI controls remain intact and undrifted. Audit cadence is now defined (weekly spot-check, mandatory pre-merge, post-incident). The runbook is confirmed followable in under 5 minutes.
  Evidence: Investigation Log entry `2026-02-23 -- Session 15: First Live Governance Audit Drill and Cadence Policy`.
- **Governance operations are now codified into a repeatable audit + incident procedure.** `research/adversarial-reward/governance/GOVERNANCE.md` defines a 7-check five-minute audit runbook (exact commands, jq filters, expected outputs, pass/fail criteria), an evidence template, and break-glass guardrails with mandatory restoration + re-verification.
  Evidence: Investigation Log entry `2026-02-23 -- Session 14: Governance Audit Runbook + Break-Glass Procedure Codification`.
- **Live strict-mode state remains confirmed and documented from source-of-truth API.** Session 14 re-verified `required_status_checks.strict=true` directly from `gh api .../branches/master/protection`; no drift from Session 13 baseline was observed.
  Evidence: Investigation Log entry `2026-02-23 -- Session 14: Governance Audit Runbook + Break-Glass Procedure Codification`.
- **The prior `bd list` panic is non-reproducing in the current session.** `bd list --status=open` completed successfully in Session 14, so no immediate remediation is required beyond documenting the known issue and workaround.
  Evidence: Investigation Log entry `2026-02-23 -- Session 14: Governance Audit Runbook + Break-Glass Procedure Codification`.
- **Admin enforcement is now active on `master`.** Branch protection now has `enforce_admins=true`, with required status checks unchanged (`contract-verification`, strict mode enabled), closing the administrator direct-push bypass.
  Evidence: Investigation Log entry `2026-02-22 -- Session 13: Admin Bypass Closure and Direct-Push Rejection Proof`.
- **Direct push to `master` is rejected under protection policy.** Attempted push `ci-proof/direct-push -> master` failed with `GH006` and required-check enforcement (`Required status check "contract-verification" is expected.`).
  Evidence: Investigation Log entry `2026-02-22 -- Session 13: Admin Bypass Closure and Direct-Push Rejection Proof`.
- **PR path remains functional with admin enforcement enabled.** Disposable PR `#5` passed `contract-verification` (run `22295647030`) and reported `mergeStateStatus=CLEAN`, confirming policy tightening did not break normal PR flow.
  Evidence: Investigation Log entry `2026-02-22 -- Session 13: Admin Bypass Closure and Direct-Push Rejection Proof`.
- **Governance chain is now complete.** Workflow gate implementation (Session 11) -> required-check PR enforcement proof (Session 12) -> admin bypass closure and direct-push rejection proof (Session 13).
  Evidence: Investigation Log entries `2026-02-22 -- Session 11`, `2026-02-23 -- Session 12`, and `2026-02-22 -- Session 13: Admin Bypass Closure and Direct-Push Rejection Proof`.
- **Branch-protection enforcement is now proven with empirical pass/fail evidence.** `master` requires `contract-verification`; PR `#3` passed (`run 22295089577`) and PR `#4` failed (`run 22295105551`), with merge attempt on `#4` blocked by base branch policy.
  Evidence: Investigation Log entry `2026-02-23 -- Session 12`.
- **The CI gate had an exit-code masking defect that is now fixed.** In Session 12, `python ... | tee ...` was shown to mask non-zero script exits without `set -o pipefail`; workflow steps were updated so contract-check failures now propagate correctly to step outcomes.
  Evidence: Investigation Log entry `2026-02-23 -- Session 12`.
- **AggregateScore contract enforcement now includes automated CI gating.** Workflow `.github/workflows/contract-gate.yml` runs `acceptance_test.py` and `monitoring_hooks.py` on every `push`/`pull_request` to `master`, uploads both stdout logs as artifact `contract-gate-output`, and fails the job unless both checks succeed.
  Evidence: Investigation Log entry `2026-02-22 -- Session 11`.
- **Session 7 architecture integration is complete.** The locked recommendation is now encoded in architecture contracts and handoff artifacts: `ARCHITECTURE.md` (Sections 4.4.1, 5.4, 8.1, Appendix), ADR `decisions/002-aggregate-score-contract.md`, acceptance-test spec, and monitoring-trigger spec.
  Evidence: Investigation Log entry `2026-02-23 -- WDK#41 Session 7`.
- **Core implementation beads are complete and verified.** `athena-4xm` (BF normalization seam), `athena-8b9` (x0 >= 0 guardrail), and `athena-fgo` (decomposition invariant) are implemented and pass 7-scenario evaluation with max margin delta 4.414e-07 against locked baselines. `athena-3lu` (acceptance test suite) is now unblocked; `athena-i4s` (monitoring hooks) remains pending.
  Evidence: Investigation Log entry `2026-02-22 -- Session 8`.
- **AggregateScore recommendation is locked.** The HTG-gated Fisher product hybrid with log-scaled BF normalization (`c=0.083647`, `bf_max_target=10000`) is the recommended aggregation function for architecture integration. Specification: `aggregate_score_recommendation.md` and `aggregate_score_recommendation.json`.
  Evidence: Investigation Log entry `2026-02-22 -- WDK#41 Session 6`.
- All four previously identified risks are resolved by design changes in the recommendation: S5 BF ceiling (log-scaled normalization), S6 compression failures (same change), S2 sigmoid fragility (x0>=0 guardrail), correlation floor-saturation (S6-based probe).
  Evidence: same Session 6 log entry (`aggregate_score_recommendation.md` Section 3.1).
- Two accepted limitations are documented with MEDIUM-confidence out-of-range classification: Pattern B under-response (step_ratio=1.029 under 50x jump) and S1 SE fragility at 5x/10x.
  Evidence: same Session 6 log entry (`aggregate_score_recommendation.md` Section 3.2).
- Five revisit triggers are defined for reopening the recommendation: empirical range violation (T1), new DivergenceKind (T2), Pattern B becomes blocking (T3), scenario expansion (T4), correlation structure change (T5).
  Evidence: same Session 6 log entry (`aggregate_score_recommendation.md` Section 4).
- A cross-family hybrid candidate (`Hybrid`) satisfies all seven stress scenarios in the default harness while preserving baseline behavior of prior candidates (`IVW 5/7`, `HTG 5/7`, `Fisher 3/7`).
  Evidence: Investigation Log entry `2026-02-21 -- WDK#41 Session 3` (`results.json`).
- Hybrid S2 compounding clears the fixed threshold without criterion relaxation: aggregate `0.9234566367020085`, max_single `0.5738586978538172`, ratio `1.6092056113389201`, margin `+7.280374%`.
  Evidence: same Session 3 log entry (`results.json` S2 raw scores).
- Hybrid meets S4 and S6 integrity gates with margin: S4 relative delta `0.0719926034986539` (`<=0.20`), S6 reconstruction error `1.1102230246251565e-16` (`<=1e-8`).
  Evidence: same Session 3 log entry (`results.json` S4/S6 raw scores and decomposition).
- Session 4 perturbation sweeps (70 runs) show localized but real fragility around three axes:
  - S2 custom sigmoid: `20/24` pass with failures only at `x0=-0.2` and `k>=2.0`
  - S5 BayesFactor: `4/9` pass with transition at `BF 110->120`
  - S6 joint compression: `11/16` pass with failures concentrated at high `d_mid` and high `bf_strong`.
  Evidence: Investigation Log entry `2026-02-22 -- WDK#41 Session 4` (`perturbation_results.json`, `perturbation_summary.md`).
- S2 sensitivity to `s2.custom.1` is now verified, not conjectural: moving to `x0=-0.2` introduces immediate failures for `k>=2.0`, while non-custom SE scaling remained `5/5` pass.
  Evidence: same Session 4 log entry (`perturbation_summary.md` S2 grid and axis pass rates).
- S5 upper-bound margin is confirmed as a tight failure boundary: PASS at `BF=110` (margin `+0.000009`), FAIL at `BF=120` (margin `-0.000736`).
  Evidence: same Session 4 log entry (`perturbation_summary.md` S5 sweep table).
- S7 boundary behavior and S4 missing-uncertainty behavior remained robust across sampled perturbations (`7/7` and `4/4` respectively).
  Evidence: same Session 4 log entry (`perturbation_results.json` axis stats).
- Session 4.2/4.3 regime-validity synthesis classifies remaining unresolved failure loci as out-of-range stress boundaries (S2 negative `x0`, S1 `SE_mult>=5`, Pattern B 50x isolated jump), while keeping BF-related failures as resolved.
  Evidence: Investigation Log entry `2026-02-22 -- WDK#41 Session 4.2/4.3` (`regime_validity.md`, `regime_validity.json`).
- Custom sigmoid guardrail is now specified at architecture level: `x0 >= 0` for all `custom_sigmoids`, enforced by config validation with reject-on-violation behavior.
  Evidence: same Session 4.2/4.3 log entry (`guardrail_spec.md`).
- All three candidates are bounded in practice for Session 1 fixtures: no NaN and no out-of-range scores in the full 3x7 matrix.
  Evidence: Investigation Log entry `2026-02-22 -- WDK#41 Session 1` (`results.json`, boundedness check).
- Session 2 structural flags are backward-compatible: with defaults, `evaluate.py` exactly reproduces Session 1 pass/fail outputs (`5/7`, `5/7`, `3/7`).
  Evidence: Investigation Log entry `2026-02-22 -- WDK#41 Session 2` (`evaluate.py` rerun before/after modifications).
- No candidate satisfies all seven stress scenarios after Session 2 sweeps; best pass counts are `IVW 2/7`, `HTG 5/7`, `Fisher 4/7` in main Stage 2 runs, with Fisher isolation at `5/7`.
  Evidence: same Session 2 log entry (`sweep_summary.md`, `sweep_results.json`).
- `HTG-Max` remains the strongest single-family performer by aggregate pass count in Session 2 sweeps, but still fails two gates in best settings (S5,S6).
  Evidence: same Session 2 log entry, Stage 2 top-table.
- Calibration decomposability is workable for all primary candidates after normalized decomposition weights in IVW (`sum(w_i*u_i) ~= aggregate`).
  Evidence: same log entry, S6 reconstruction values.
- S2 criterion-sensitivity frontier (for configs at 6/7 failing only S2) yielded no qualifying configs for any candidate in Session 2.
  Evidence: same Session 2 log entry (`sweep_summary.md`, S2 frontier table).
- Calibration pattern B is unstable across all best-per-candidate configs (either insufficient step response or excessive jump/smoothness failure).
  Evidence: same Session 2 log entry (`calibration_summary.md`).
- Fisher correlation inflation flag did not trigger at rho=0.5 (`inflation_ratio` did not exceed 1.5), but the test aggregates were floor-saturated.
  Evidence: same Session 2 log entry (`correlation_results.json`).

### What We Suspect

- Joint use of normalization-level SE dampening and Fisher SE-reliability scaling may be over-attenuating evidence in some regimes.
  Evidence basis: Fisher isolation (`se_dampen=False`) improved from 4/7 to 5/7 vs. main sweep (Session 2).

### What We Don't Know

- Whether the Session 17 outage timing windows (`<=1h` override for Modes A/B; `<=2h` containment with 15-minute probes for Mode C) hold under live operational conditions, because Session 17 was tabletop-only.
  Evidence: Investigation Log entry `2026-02-23 -- Session 17: Break-Glass Outage Resilience (Tabletop, Read-Only)`.
- Whether derived operating-range estimates in `regime_validity.md` match observed parameter distributions in real production DSL traces (OpenMM/GROMACS/CESM/VASP), rather than training-knowledge priors.
- Whether the hybrid remains `7/7` outside the current fixed fixture set under adversarial scenarios not covered by S1-S7 (e.g., metric-count scaling, temporal autocorrelation).
- Whether Pattern B step-response can be improved above 3.0 without reintroducing Fisher-like non-smooth jumps (open thread from Session 5, documented as revisit trigger T3).

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| `research/adversarial-reward/governance/audit.sh` | Governance audit automation script (co-located with GOVERNANCE.md; operational tooling, not an aggregation prototype) | Complete (Session 16) | Read-only single-command C1-C7 governance audit with PASS/FAIL reporting, evidence-template output, and `--dry-run` command-preview mode |
| `research/adversarial-reward/prototypes/aggregation-candidates/models.py` | Contract-preserving dataclasses/enums for aggregation prototype inputs and outputs | Complete (Session 1) | Input contract mirrored without mutating schema semantics |
| `research/adversarial-reward/prototypes/aggregation-candidates/normalization.py` | Shared CDF normalization, direction handling, uncertainty extraction, and weight helpers | Complete (Session 1) | Uniform `[0,1]` mapping across heterogeneous divergence kinds; agreement inversion implemented |
| `research/adversarial-reward/prototypes/aggregation-candidates/candidates.py` | C1 IVW-CDF, C2 HTG-Max, C3 Fisher-UP candidate implementations + exploratory HTG-LSE | Complete (Session 1) | Three bounded aggregation formulations with per-component decomposition output |
| `research/adversarial-reward/prototypes/aggregation-candidates/scenarios.py` | Seven adversarial scenario fixtures for stress testing | Complete (Session 1) | Standardized scenario coverage for Noisy-TV, calibration, heterogeneity, and missing-data stressors |
| `research/adversarial-reward/prototypes/aggregation-candidates/evaluate.py` | Matrix runner and artifact generator for candidate-by-scenario evaluation | Complete (Session 1) | 3x7 matrix, pass/fail adjudication, decomposition capture, exploratory variant execution |
| `research/adversarial-reward/prototypes/aggregation-candidates/results.json` | Raw machine-readable Session 1 outputs | Complete (Session 1) | Full per-cell scores, pass/fail, warnings/skips, and decompositions |
| `research/adversarial-reward/prototypes/aggregation-candidates/results.md` | Human-readable Session 1 matrix summary | Complete (Session 1) | Compact 3x7 evidence table for research log integration |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep.py` | Session 2 two-stage parameter sweep driver (normalization + candidate sweeps + S2 sensitivity) | Complete (Session 2) | Exhaustive scenario evaluation over 723 candidate-configs (243 Stage 1 + 480 Stage 2) |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep_results.json` | Full Session 2 sweep records for all evaluated configs and scenarios | Complete (Session 2) | Machine-readable pass/fail matrices, raw scores, and config metadata |
| `research/adversarial-reward/prototypes/aggregation-candidates/sweep_summary.md` | Human-readable Session 2 sweep rankings and frontier summary | Complete (Session 2) | Top-5 per candidate, no-7/7 finding, Fisher isolation comparison |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_sim.py` | Deterministic 50-cycle calibration stress simulation for patterns A/B/C | Complete (Session 2 Stretch) | Pattern metrics (Spearman/step-ratio/Pearson) + smoothness diagnostics per candidate |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_results.json` | Raw cycle-level calibration outputs | Complete (Session 2 Stretch) | Per-cycle scores and pass/fail stats for each pattern/candidate |
| `research/adversarial-reward/prototypes/aggregation-candidates/calibration_summary.md` | Human-readable calibration summary | Complete (Session 2 Stretch) | Pattern-by-candidate pass matrix and smoothness outcomes |
| `research/adversarial-reward/prototypes/aggregation-candidates/correlation_test.py` | Fisher-UP correlation robustness probe with Cholesky generation + Brown-style correction | Complete (Session 2 Stretch) | Inflation-ratio diagnostics across rho levels with overflow-safe corrected CDF terms |
| `research/adversarial-reward/prototypes/aggregation-candidates/correlation_results.json` | Correlation robustness outputs | Complete (Session 2 Stretch) | Inflation ratios at rho `{0.0,0.3,0.5,0.7,0.9}` and rho=0.5 flag status |
| `research/adversarial-reward/prototypes/aggregation-candidates/perturbation_test.py` | Session 4 targeted robustness sweep harness for Hybrid under fixture perturbations | Complete (Session 4) | 70-run robustness map, exact tipping-point detection, and baseline sanity reproduction |
| `research/adversarial-reward/prototypes/aggregation-candidates/perturbation_results.json` | Raw Session 4 perturbation outputs | Complete (Session 4) | Per-axis/per-point pass status, margins, raw scores, and tipping metadata |
| `research/adversarial-reward/prototypes/aggregation-candidates/perturbation_summary.md` | Human-readable Session 4 robustness summary | Complete (Session 4) | Scenario-by-axis pass rates, S2 grid, S5 frontier, and S6 transition boundaries |
| `research/adversarial-reward/prototypes/aggregation-candidates/stretch_test.py` | Session 5 post-ceiling hybrid stretch harness (Phase 0 gate, calibration patterns, correlation robustness, artifact emission) | Complete (Session 5 Stretch) | Deterministic 4-phase execution with strict baseline gate and S6 non-floor-saturated correlation diagnostics |
| `research/adversarial-reward/prototypes/aggregation-candidates/stretch_results.json` | Session 5 machine-readable stretch outputs | Complete (Session 5 Stretch) | Verified Phase 0 `7/7`, Pattern B under-response classification, and rho=0.5 inflation pass with floor_saturated=False |
| `research/adversarial-reward/prototypes/aggregation-candidates/stretch_summary.md` | Session 5 human-readable stretch summary | Complete (Session 5 Stretch) | Side-by-side Session 2 comparison tables and explicit verdict on calibration vs correlation stretch behavior |
| `research/adversarial-reward/prototypes/aggregation-candidates/regime_validity.md` | Session 4.2 regime-validity writeup for DSL-realistic parameter ranges and failure overlays | Complete (Session 4.2/4.3) | Explicit in-range/out-of-range/resolved classification for S2, S1, Pattern B, S5, and S6 with confidence tags |
| `research/adversarial-reward/prototypes/aggregation-candidates/regime_validity.json` | Machine-readable Session 4.2 regime-validity artifact | Complete (Session 4.2/4.3) | Structured parameter ranges, domain assessments, failure classifications, and verdict for downstream automation |
| `research/adversarial-reward/prototypes/aggregation-candidates/guardrail_spec.md` | Session 4.3 architecture-level guardrail specification (`x0 >= 0`) for custom sigmoids | Complete (Session 4.2/4.3) | Constraint statement, rationale, scope, and reject-on-violation enforcement contract for future production integration |
| `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md` | Session 6 canonical AggregateScore specification for architecture integration | Complete (Session 6) | Locked algorithm, parameters, evidence map, operating boundaries, and revisit triggers |
| `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.json` | Machine-readable Session 6 locked parameters and spec | Complete (Session 6) | Structured parameters, guardrails, envelope, limitations, and triggers for downstream automation |
| `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_acceptance_test_spec.md` | Session 7 acceptance-gate contract translated from Session 6 evidence | Complete (Session 7) | Blocking/non-blocking test matrix with locked tolerances and evidence-linked assertions |
| `research/adversarial-reward/prototypes/aggregation-candidates/monitoring_triggers.md` | Session 7 revisit-trigger monitoring contract for T1-T5 | Complete (Session 7) | Source/threshold/owner/action path for reopening locked recommendation under operational drift |

## Next Steps

0. **WDK#41 Step 0 (updated): Session 3 bridge from sweep outcomes** — Design and evaluate cross-family/hybrid candidates explicitly targeting the unresolved `(S1,S2)` and `(S5,S6)` tradeoffs identified in Session 2. Scope: 1-2 sessions.

1. **Survey formalizations in active learning and Bayesian experimental design** — Review how information gain is formalized in discriminative active learning, Bayesian optimization (expected improvement, knowledge gradient), and optimal experimental design. Identify which formalizations handle bounded search spaces. Scope: 2-3 sessions.

2. **Characterize the conservatism-vs-boundary-seeking failure spectrum** — Formally describe the two failure modes as functions of reward function properties. Under what conditions does a given formalization collapse to conservatism? To boundary-seeking? Identify the design parameters that control this tradeoff. Scope: 1-2 sessions.

3. **Analyze Noisy TV in DSL simulation contexts** — The Noisy TV problem is well-studied in RL. Characterize how it manifests specifically in DSL simulation environments: what sources of irreducible stochasticity exist in OpenMM/GROMACS/VASP? How does the DSL's deterministic subspace constraint interact with noise sources? Scope: 1-2 sessions.

4. **Investigate calibration loop constraints on functional form** — The calibration feedback (ARCHITECTURE.md 5.4) compares predicted vs. actual surprise. What constraints does this calibration mechanism place on the reward function's functional form? Must the function be decomposable in specific ways for calibration to be meaningful? Scope: 1-2 sessions.

5. **Draft candidate reward function specifications** — Propose 2-3 candidate formalizations with explicit tradeoff profiles across the failure spectrum. Each should be evaluable against the calibration loop requirements. Scope: 2-3 sessions.
