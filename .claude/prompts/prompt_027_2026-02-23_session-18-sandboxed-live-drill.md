# Prompt: Session 18 — Sandboxed Live Drill of Outage Playbook

> Generated: 2026-02-23 | Prompt #027 | Framework: RISEN

---

## Session Goal

Execute a sandboxed live drill of the Mode B and Mode C break-glass outage procedures, capturing real timestamps and evidence artifacts. Mode B tests the full web-UI override/restore cycle. Mode C validates containment discipline, 15-minute monitoring cadence, escalation timing, and recovery handoff to Mode A/B — without mutating branch protection while both interfaces are down. Validate or adjust policy time windows against measured reality, closing the last open thread in FINDINGS.md.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step process with strict methodology (sandbox setup → Mode B live drill → Mode C live drill → timing comparison → findings log). Narrowing component critical for Mode C no-mutation constraint and sandbox boundaries.
- **Alternatives considered:** TIDD-EC (good dos/don'ts but weaker on sequential step orchestration), Chain of Thought (reasoning already done in Session 17 tabletop; Session 18 is execution with measurement).

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Each drill phase has explicit timestamps, evidence, and success criteria |
| Specificity | 9/10 | Timestamp labels (T_B0–T_B4, T_C0–T_C3), headroom bands, table schema all defined |
| Context | 9/10 | References GOVERNANCE.md Sections 3.1–3.4, Session 17 log, audit.sh, existing evidence chain |
| Completeness | 9/10 | Both drills, timing comparison, FINDINGS update, accumulated findings closure all covered |
| Structure | 10/10 | RISEN components map cleanly; steps sequential with clear operator handoff points |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are the drill orchestrator for a pre-approved sandboxed live drill of the ATHENA project's break-glass outage procedures (GOVERNANCE.md Section 3, Sections 3.1–3.4). You manage the drill sequence: creating the bead and branch, running pre-drill audits, issuing timestamped instructions, recording evidence, computing durations, and writing the FINDINGS.md log entry. For Mode B steps requiring browser-UI interaction (navigating GitHub settings, toggling checkboxes, capturing screenshots), you pause and instruct the human operator to execute the UI action, then record the operator's confirmation and timestamp before proceeding. You are familiar with: the Mode A/B/C outage taxonomy, the operator decision tree, the per-mode evidence requirements, the `audit.sh` automation script, and the ATHENA research workflow (append-only FINDINGS.md logs, beads-based issue tracking).

INSTRUCTIONS:
This is a **sandboxed live drill** — pre-approved for controlled execution against the real `andrewmcadoo/athena` repository, not a tabletop exercise. The drill captures real timestamps, real evidence artifacts, and real `audit.sh` output.

Operator handoff protocol for Mode B browser steps:
- The model cannot interact with the GitHub web UI. When a step requires browser navigation, screenshot capture, or toggle-click, the model MUST pause, present the exact action required to the human operator, and wait for confirmation (including timestamp) before logging the result and proceeding.
- Evidence hierarchy: (1) screenshots preferred; (2) if screenshots unavailable, use `gh api repos/andrewmcadoo/athena/branches/master/protection --jq '.enforce_admins.enabled'` as terminal-environment evidence and note the substitution; (3) operator verbal confirmation with timestamp is minimum acceptable evidence. Propose any evidence-format adaptation as a GOVERNANCE.md clarification in Step 7.

Drill authorization scope:
- **Mode B drill**: Full live execution of the web-UI override/restore cycle. The human operator navigates to the GitHub settings page, captures screenshots (or API-based evidence), unchecks "Include administrators" (`enforce_admins`), re-checks it, and confirms each action. The model records timestamps, runs `audit.sh` for restore verification, and computes durations. No emergency change is made between override and restore — the drill tests procedure mechanics only.
- **Mode C drill**: Live execution of containment discipline. Simulate the outage condition by treating API and UI as "unavailable" (do not actually call `gh api` for protection queries or navigate to settings during the simulated outage window). Execute containment steps: freeze declaration, outage-start recording, monitoring cadence (compressed cycles — see Step 5d), recovery detection simulation, and handoff to Mode A or B. **No branch-protection mutations during Mode C.** Mode C tests operational discipline and timing, not settings changes.

Timing measurement protocol:
- Record UTC timestamps at every phase transition using `date -u +%Y-%m-%dT%H:%M:%SZ`.
- Measure wall-clock duration between transitions.
- Compare measured durations against policy windows using these headroom bands:
  - **< 50% of policy limit**: validates window with strong headroom.
  - **50%–80% of policy limit**: validates but flags for monitoring in future drills.
  - **> 80% of policy limit**: challenges the window; propose adjusted limit with justification.
- Policy windows under test: 1-hour override max (Modes A/B), 15-minute monitoring interval (Mode C), 2-hour containment max (Mode C), 1-hour deferred audit window (Modes B/C).

Follow ATHENA's research methodology throughout: read existing artifacts before executing, append-only investigation logs, cite evidence for every claim.

STEPS:
1. **Create bead and branch.** Run `bd create --title="Session 18: sandboxed live drill — outage playbook" --description="Pre-approved sandboxed live drill of Mode B (web-UI override/restore) and Mode C (containment/monitoring/recovery handoff). Capture real timestamps and evidence, validate or adjust policy time windows, close the remaining sandboxed-drill open thread." --type=task --priority=2`, mark it in-progress, and create branch `session-18/sandboxed-live-drill` from `master`.

2. **Read current artifacts.** Read `research/adversarial-reward/governance/GOVERNANCE.md` (full file, especially Sections 3.1–3.4) and `research/adversarial-reward/FINDINGS.md` (read through the Accumulated Findings section — specifically "What We Know / What We Suspect / What We Don't Know" categories — and the Session 17 log entry) before executing anything.

3. **Pre-drill baseline audit.** Run `bash research/adversarial-reward/governance/audit.sh` and confirm 7/7 PASS. Record the output as the pre-drill baseline. If any check fails, STOP — do not proceed with drill until baseline is clean.

4. **Mode B live drill — web-UI override/restore cycle.**
   - 4a. Record drill start timestamp (`T_B0`).
   - 4b. Instruct human operator: "Navigate to `https://github.com/andrewmcadoo/athena/settings/branches`, open the `master` protection rule."
   - 4c. Instruct human operator: "Capture a full screenshot of the protection rule page showing all toggle states. Confirm when done and provide the timestamp." Record timestamp (`T_B1`). This is the "before-override" evidence.
   - 4d. Instruct human operator: "Uncheck 'Include administrators' and click 'Save changes'. After saving, verify the page refreshes showing 'Include administrators' unchecked. If the page still shows it checked, record the failed save and attempt once more before proceeding. Confirm when done and provide the timestamp." Record timestamp (`T_B2`).
   - 4e. Instruct human operator: "Capture a screenshot showing the unchecked state. Confirm when done." This is the "after-override" evidence.
   - 4f. Instruct human operator: "**Immediately** re-check 'Include administrators' and click 'Save changes'. Verify the page refreshes showing it checked. Confirm when done and provide the timestamp." Record timestamp (`T_B3`).
   - 4g. Instruct human operator: "Capture a screenshot showing the restored state. Confirm when done." This is the "after-restore" evidence.
   - 4h. Run `bash research/adversarial-reward/governance/audit.sh` to verify 7/7 PASS post-restore. Record timestamp (`T_B4`). Note: during the Mode B drill, the GitHub API is not actually unavailable — Mode B simulates API-down only for the override window context. Running `audit.sh` at this step uses the real API and is expected to succeed.
   - 4i. Compute durations: override window = `T_B3 - T_B2`; total drill time = `T_B4 - T_B0`.
   - 4j. Record all timestamps, durations, and evidence references in bead notes.

5. **Mode C live drill — containment, monitoring cadence, and recovery handoff.**
   - 5a. Record drill start timestamp (`T_C0`).
   - 5b. Declare operational merge freeze. Record freeze timestamp in bead.
   - 5c. Create outage log file: `printf "mode=Mode C\noutage_start_utc=%s\n" "$(date -u +%Y%m%dT%H%M%SZ)" > "/tmp/governance-outage-$(date -u +%Y%m%dT%H%M%SZ).txt"`. Record timestamp (`T_C1`).
   - 5d. Execute monitoring cycles. Use a compressed cadence to keep total drill time practical; choose a compression ratio that keeps the Mode C drill under 15 minutes total and document the chosen ratio explicitly (e.g., "2-min intervals simulate 15-min intervals, compression ratio 7.5:1"). Execute at least 3 monitoring cycles:
     - Each cycle: record timestamp, note "API: simulated unavailable, UI: simulated unavailable" (do NOT actually probe), log in bead notes.
   - 5e. After the monitoring cycles, simulate recovery detection: declare "API recovered at T_C_recovery" (or "UI recovered" for Mode B handoff variant). Record timestamp (`T_C2`).
   - 5f. Execute recovery handoff: since API is "recovered," follow Mode A restore path — run `bash research/adversarial-reward/governance/audit.sh` to confirm 7/7 PASS as recovery verification. Record timestamp (`T_C3`).
   - 5g. Compute durations: containment setup = `T_C1 - T_C0`; per-cycle monitoring time; total containment = `T_C2 - T_C0`; recovery handoff = `T_C3 - T_C2`.
   - 5h. Lift merge freeze. Record in bead.

6. **Timing comparison and window validation.**
   - Compare Mode B measured override window against the 1-hour policy limit. Compute headroom percentage and classify per the headroom bands in Instructions.
   - Compare Mode C measured containment setup, monitoring cadence, and total containment against the 2-hour policy limit and 15-minute check interval. Account for the compression ratio when extrapolating to real-world durations.
   - For each time window: state whether the measured value validates the policy window or challenges it, using the headroom classification. If challenged (>80%), propose an adjusted window with justification.
   - Note: the compressed Mode C drill confirms the monitoring cadence is operationally executable (steps are completable within the interval). It does not statistically validate that 15 minutes is the optimal interval — that requires real outage data or a longer uncompressed drill.
   - Assess whether any step was ambiguous or caused operator hesitation during live execution. If so, flag for Step 7.

7. **Fix any ambiguous steps.** If Step 6 identified ambiguous or unclear steps in GOVERNANCE.md Sections 3.1–3.4, make targeted edits to clarify them. Each edit must be minimal and cited in the Session 18 FINDINGS entry. If evidence-format adaptations are needed (e.g., terminal-environment API-based evidence as alternative to screenshots), propose as a GOVERNANCE.md addition.

8. **Log Session 18 in FINDINGS.md.** Add investigation log entry at top of Investigation Log (reverse-chronological) with:
   - **Scope**: Sandboxed live drill of Mode B and Mode C outage procedures.
   - **Method**: Live drill protocol with timestamps, human-operator handoff for UI steps, compressed Mode C cadence, recovery handoff.
   - **Findings**: All measured timestamps, durations, compression ratios, and timing comparisons in tabular format:
     ```
     | Event | Timestamp (UTC) | Duration from prior event | Policy limit | Headroom |
     ```
     Include separate tables for Mode B and Mode C.
   - **Implications**: Which policy windows are validated, which need adjustment, and what the drill proved about procedural clarity.
   - **Open Threads**: Any remaining gaps. If none, state explicitly: "The sandboxed-drill open thread from Session 17 is closed."

9. **Update Accumulated Findings.** In the "What We Know" / "Open Threads" categories:
   - Move "run a separately approved sandboxed live break-glass drill to validate Mode B/Mode C timing assumptions" from Open Threads to What We Know, citing Session 18 log entry and measured evidence.
   - If windows were adjusted, note the adjustment in What We Know with measured headroom data.

10. **Commit, PR, verify, merge.** Commit message: `research(adversarial-reward): Session 18 — sandboxed live drill of outage playbook`. Push branch, open PR to `master`, verify CI passes (contract-gate), merge, close bead, `bd sync`.

END GOAL:
- Mode B and Mode C procedures have been executed live with real timestamps and evidence artifacts.
- Every timestamp and duration is logged in FINDINGS.md in tabular format (Event | Timestamp | Duration | Policy limit | Headroom).
- Policy time windows (1h override, 15-min checks, 2h containment, 1h deferred audit) are either validated with measured headroom classification or adjusted with justification and updated in GOVERNANCE.md.
- Any ambiguous steps discovered during live execution are clarified in GOVERNANCE.md.
- The "sandboxed live drill" open thread in FINDINGS.md is closed with citation to Session 18 evidence.
- CI passes on the PR.

NARROWING:
- Do NOT make an actual emergency change during Mode B. The drill tests override/restore mechanics only — uncheck enforce_admins, immediately re-check, verify via audit.sh.
- Do NOT mutate branch protection during Mode C. Mode C is containment-only; no settings changes are possible or attempted when both interfaces are "down."
- Do NOT actually take down the GitHub API or UI. Mode C simulates unavailability by not using those interfaces during the containment window.
- Do NOT skip the pre-drill baseline audit (Step 3). If baseline is not 7/7 PASS, the drill does not proceed.
- Do NOT skip the post-restore audit after Mode B (Step 4h). This is the mandatory verification that the live toggle actually restored correctly.
- Do NOT use full 15-minute wait cycles if impractical. Use compressed cadence but document the compression ratio explicitly and acknowledge the drill confirms operational executability, not optimal interval calibration.
- Do NOT execute GitHub settings UI actions directly. All browser-UI steps are human-operator actions; the model orchestrates, timestamps, and logs.
- Do NOT edit prior FINDINGS.md investigation log entries. Append-only.
- Stay within governance/break-glass scope. No changes to audit.sh logic, contract-gate.yml, or other ATHENA research areas.
- Avoid grant-proposal rhetoric. Report measured facts.
- Out of scope: Mode A drill (already validated by Session 16 tabletop + every prior live audit run), production code, new ADRs.
```

---

## Review Findings

### Issues Addressed
- **C1 (browser steps unreachable by model):** Reframed Mode B as human-operator checkpoints. ROLE changed from "drill operator" to "drill orchestrator." Each Step 4b–4g now explicitly pauses and instructs the human operator. Evidence hierarchy defined (screenshots > API query > verbal confirmation). Operator handoff protocol added to Instructions.
- **W1 (screenshot fallback):** Evidence hierarchy added with API-based terminal-environment substitute and GOVERNANCE.md clarification trigger.
- **W2 (audit.sh context):** Clarifying sentence added to Step 4h: API is not actually down, audit.sh expected to succeed.
- **W3 (save failure):** Verification substep added to Step 4d: confirm page shows unchecked before proceeding.
- **W4 (3-cycle overstates validation):** Step 6 now says "confirms cadence is operationally executable" with explicit caveat about statistical validation.
- **W5 (validation threshold undefined):** Explicit headroom bands added to Instructions: <50% strong, 50-80% monitor, >80% challenge.
- **W6 (timestamp table schema):** Column spec added to Step 8: Event | Timestamp (UTC) | Duration from prior event | Policy limit | Headroom.

### Remaining Suggestions
- **S1 (logging target distinction):** Bead notes are live capture, FINDINGS.md is permanent record. Could add explicit note but distinction is clear from workflow.
- **S2 (Step 7 cross-reference in Step 8):** If GOVERNANCE.md edits made in Step 7, Step 8 Method should reference them. Added implicitly via "each edit must be cited in the Session 18 FINDINGS entry."
- **S3 (FINDINGS.md read depth):** Changed to "read through the Accumulated Findings section" with named categories.
- **S4 (compression ratio example as default):** Changed to "choose a compression ratio that keeps total drill time under 15 minutes" with documentation requirement.

## Usage Notes

- **Best used with:** Claude Opus 4.6 in Claude Code CLI with beads workflow and git hooks active. Requires a human operator with browser access to the repository settings page for Mode B UI steps.
- **Adjust for:** If running in a fully terminal environment, use the API-based evidence fallback and propose a GOVERNANCE.md update for terminal-environment evidence requirements.
