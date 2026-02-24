# Prompt: Session 16 — Audit Automation, Break-Glass Tabletop, Escalation Thresholds

> Generated: 2026-02-23T19:30:00Z | Prompt #025 | Framework: RISEN

---

## Session Goal

Build a Session 16 plan that (1) automates the governance audit via a read-only C1-C7 script with printed evidence template, (2) rehearses the break-glass procedure as a tabletop/simulated drill with no live mutation unless explicitly approved, and (3) defines escalation thresholds mapping each failed check (C1-C7) to severity, owner action, and response time. All work is non-mutating by default.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step process with clear methodology (script authoring, drill execution, threshold definition), sequential dependencies, explicit narrowing constraints (non-mutating default), and a concrete end goal with three discrete deliverables.
- **Alternatives considered:** TIDD-EC (good for dos/don'ts emphasis on non-mutating constraints, but weaker at capturing the sequential step structure and role context).

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Three discrete deliverables with explicit mutation constraints; no ambiguity |
| Specificity | 9/10 | C1-C7 checks fully defined in GOVERNANCE.md; escalation table schema specified per-check |
| Context | 9/10 | Full governance chain (Sessions 11-15) provides complete background; file paths and line references available |
| Completeness | 9/10 | Covers what/why/how; output format for each deliverable specified; review findings incorporated |
| Structure | 9/10 | RISEN components map cleanly to 10 sequential steps with clear dependencies |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are continuing the ATHENA adversarial-reward governance work (Sessions 11-15) on andrewmcadoo/athena. You have full context on the AggregateScore contract governance chain, branch protection on `master`, and the CI gate (`contract-gate.yml`). You follow the research workflow in CLAUDE.md and the append-only FINDINGS.md protocol.

INSTRUCTIONS:
- **Non-mutating by default.** All automation is read-only. The break-glass drill is tabletop/simulated. No repository state changes (branch protection, workflow config, admin settings) occur unless the user explicitly approves live mutation.
- Read GOVERNANCE.md and FINDINGS.md in full before any work. Do not start from scratch.
- Follow the FINDINGS.md append-only protocol: new log entries go at top of Investigation Log (reverse chronological). Do not edit previous entries.
- Every claim in Accumulated Findings must cite a log entry or evidence source.

STEPS:

1. **Create bead and branch.**
   - `bd create --title="Session 16: audit automation, break-glass tabletop, escalation thresholds" --description="Automate C1-C7 audit as read-only script, run tabletop break-glass drill, define per-check escalation thresholds" --type=task --priority=2`
   - `bd update <id> --status=in_progress`
   - Create branch `session-16/audit-automation-drill` from `master`.

2. **Write the audit automation script.**
   - Create `research/adversarial-reward/governance/audit.sh`.
   - **Placement rationale:** `audit.sh` is co-located with `GOVERNANCE.md` because it implements the runbook defined there and is operationally coupled to it. It is not a throwaway research prototype — it is a governance operations tool. This is a deliberate exception to the CLAUDE.md "prototypes live in `prototypes/` only" rule, which applies to research artifacts, not operational tooling.
   - The script must:
     - Be executable (`chmod +x`), stdlib-only (bash + `gh` CLI + `jq` + `date` + `git`).
     - Run C1-C7 verbatim from GOVERNANCE.md Section 2 (lines 67-141).
     - Compare each check's output against expected values and print PASS/FAIL per check.
     - On completion, print the filled evidence template (GOVERNANCE.md lines 159-175) with: UTC date, auditor set to `$(whoami)`, commit SHA from `git rev-parse HEAD`, and per-check results.
     - Exit 0 if all checks pass, exit 1 if any check fails.
     - Be strictly read-only: no `gh api --method PUT/PATCH/POST/DELETE`, no `git push`, no `git commit`, no branch protection mutations.
   - Support a `--dry-run` flag:
     - Resolve repo variables via `gh repo view` (this is read-only).
     - Print each check's full command string with variables substituted, but do NOT execute `gh api` calls.
     - Always exit 0 in dry-run mode (no checks are evaluated, so no pass/fail).
     - Do NOT print the evidence template in dry-run mode (no results exist to populate it).

3. **Validate the audit script.**
   - Run `bash research/adversarial-reward/governance/audit.sh` and capture output.
   - Verify all 7 checks PASS and evidence template is correctly populated.
   - Run with `--dry-run` and confirm: 7 command strings printed, no `gh api` calls executed, exit code 0, no evidence template printed.
   - This step must complete before Step 7 (FINDINGS entry references script output).

4. **Run tabletop break-glass drill.**
   - This is documentation-only. Do NOT execute any override commands.
   - Walk through GOVERNANCE.md Section 3 (Break-Glass Procedure) step by step:
     a. State the simulated trigger scenario (e.g., "CI workflow accidentally disabled by GitHub incident").
     b. Document what the operator would run at each step (backup capture, override, emergency change, restore).
     c. Time each phase conceptually: backup (est. 30s), override (est. 30s), emergency change (variable), restore + re-audit (est. 5min).
     d. Identify gaps or ambiguities in the break-glass procedure discovered during walkthrough. At minimum, evaluate: does the procedure address the scenario where `gh api` itself is unavailable (e.g., GitHub API outage)? Is the web UI fallback (GOVERNANCE.md line 236) sufficient for ALL override steps, not just the restore step? Document findings.
     e. Log the drill narrative, timing estimates, and any discovered gaps as a subsection in the Session 16 FINDINGS entry.
   - This step must complete before Step 7 (FINDINGS entry logs the drill narrative).
   - If the user explicitly approves live mutation for a sandboxed drill, create a disposable branch and test admin-enforcement toggle + restore. Otherwise, tabletop only.

5. **Insert and validate escalation thresholds.**
   - The following escalation table is pre-authored based on governance impact analysis from Sessions 11-15. The session's role is to insert it into GOVERNANCE.md and validate that each severity assignment is consistent with the governance evidence chain (GOVERNANCE.md Section 1).
   - Add a new subsection to GOVERNANCE.md Section 2 (after "Audit Cadence", before Section 3):

     ```markdown
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
     ```

   - After insertion, cross-check each row against the governance evidence chain in GOVERNANCE.md Section 1 to confirm severity assignments are warranted. Note any adjustments in the FINDINGS entry.

6. **Update GOVERNANCE.md metadata.**
   - Change the "Last updated" line to: `Last updated: 2026-02-23 (Session 16 — audit automation + escalation thresholds)`

7. **Write Session 16 investigation log entry in FINDINGS.md.**
   - Prepend at top of Investigation Log with:
     - **Scope**: Audit automation script, tabletop break-glass drill, escalation threshold definition.
     - **Method**: Script authored and validated with live run; break-glass walked through as tabletop exercise; escalation table inserted from pre-authored specification and validated against governance evidence chain.
     - **Findings**: Script output (7/7 or failures), tabletop drill narrative with timing estimates and discovered gaps, escalation table validation notes.
     - **Implications**: Audit is now automatable in a single command; break-glass procedure timing is estimated; failure severity is codified for operator triage.
     - **Open Threads**: Any gaps discovered during tabletop drill; any escalation assignments that need revision after real-world testing.
   - Review open threads from Sessions 11-14 to confirm none are newly closable by the escalation thresholds or audit automation.

8. **Update Accumulated Findings.**
   - Add bullet under "What We Know": governance audit is now automatable via `audit.sh` with read-only execution and evidence output; escalation thresholds codify per-check severity and response times.
   - If tabletop drill revealed gaps, add bullet under "What We Suspect" or "What We Don't Know" as appropriate.

9. **Update Prototype Index.**
   - Add `research/adversarial-reward/governance/audit.sh` to the Prototype Index table in FINDINGS.md with Purpose: "Governance audit automation script (co-located with GOVERNANCE.md; operational tooling, not an aggregation prototype)".

10. **Commit, push, PR, verify CI, merge.**
    - Stage: `GOVERNANCE.md`, `FINDINGS.md`, `audit.sh`
    - Commit: `research(adversarial-reward): Session 16 — audit automation, break-glass tabletop, escalation thresholds`
    - Push branch, open PR targeting `master`, wait for CI (`contract-verification` — expected to pass since the PR contains a new bash script and modified markdown, not changes to the Python contract scripts).
    - Merge PR. `bd close <id>` + `bd sync`.
    - If `bd` panics, document the command attempted and continue without bead tracking.

END GOAL:
Three deliverables verified and merged to `master`:
1. `audit.sh` — executable, read-only, runs C1-C7, prints filled evidence template, exits 0/1. Has `--dry-run` mode. Validated by a live run producing 7/7 PASS and a dry-run producing command strings only.
2. Tabletop break-glass drill — narrative walkthrough logged in FINDINGS.md with per-phase timing estimates, at least one evaluated gap scenario (API outage fallback), and any discovered procedure gaps.
3. Escalation thresholds — pre-authored per-check severity/action/response-time table inserted into GOVERNANCE.md and validated against governance evidence chain, with severity definitions.
CI passes on the PR. Bead closed.

NARROWING:
- Do NOT mutate repository state (branch protection, workflow settings, admin config) unless the user explicitly approves a live sandboxed drill.
- Do NOT execute break-glass override commands. Tabletop only.
- Do NOT add dependencies beyond bash, gh CLI, jq, date, git for the audit script.
- Do NOT change the audit runbook checks themselves (C1-C7 definitions are locked in GOVERNANCE.md Section 2).
- Do NOT edit previous FINDINGS.md Investigation Log entries.
- Do NOT write production code. `audit.sh` is a governance operations artifact co-located with GOVERNANCE.md.
- Out of scope: changing branch protection settings, modifying CI workflow, altering the AggregateScore contract.
```

---

## Review Findings

### Issues Addressed
1. **Critical — `audit.sh` placement:** Added explicit rationale for co-location with GOVERNANCE.md as operational tooling exception to prototype rule. Updated Prototype Index entry to clarify categorization.
2. **Critical — Pre-written escalation table:** Made explicit that table is pre-authored from Sessions 11-15 analysis; session's role is insertion and validation. Updated FINDINGS method language to "inserted from pre-authored specification and validated."
3. **Warning — Prior open threads:** Added instruction in Step 7 to scan Sessions 11-14 open threads for newly closable items.
4. **Warning — `--dry-run` underspecified:** Added detailed behavior spec: resolves variables via `gh repo view`, prints command strings, skips `gh api` calls, always exits 0, no evidence template.
5. **Warning — Tabletop drill thin:** Added concrete gap-finding requirement (API outage fallback evaluation for all override steps).
6. **Warning — Prototype Index categorization:** Purpose field clarifies governance artifact, not aggregation prototype.

### Remaining Suggestions
- Step ordering dependencies could be made more explicit (noted in Steps 3 and 4 with "must complete before Step 7").
- CI failure handling in Step 10 could specify fallback (currently notes expected pass reason).
- `bd` panic fallback added to Step 10.

## Usage Notes

- **Best used with:** Claude Code in the ATHENA repository with `gh` CLI authenticated and `bd` available.
- **Adjust for:** If the user approves live break-glass drill, Step 4 expands to include disposable branch creation and admin-enforcement toggle test.
