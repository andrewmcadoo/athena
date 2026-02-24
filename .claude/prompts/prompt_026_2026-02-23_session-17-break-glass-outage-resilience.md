# Prompt: Session 17 — Break-Glass Outage Resilience

> Generated: 2026-02-23 | Prompt #026 | Framework: RISEN

---

## Session Goal

Extend ATHENA's break-glass governance procedure (GOVERNANCE.md Section 3) with three outage-mode fallback paths (API-only down, web-UI-only, both-down containment), an operator decision tree with time windows, evidence capture requirements per mode, and a tabletop drill validating all three paths — then log the session in FINDINGS.md. Tabletop/read-only session: no live repo-state mutation unless a sandboxed live drill is separately approved.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step process with clear sequential methodology (extend docs → drill → log). Steps component maps to five implementation phases. Narrowing component enforces the critical "tabletop/read-only" constraint and ATHENA's research-artifact rules.
- **Alternatives considered:** TIDD-EC (strong on dos/don'ts but weaker on sequential step design; the governance extension requires precise ordering), Chain of Thought (reasoning-oriented, but this task is execution-oriented with logic already decided).

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal unambiguous; each step spells out exactly what to write and where to insert |
| Specificity | 9/10 | Mode definitions, evidence requirements, time windows, file locations all concrete |
| Context | 9/10 | References existing artifacts, prior session findings, ATHENA methodology |
| Completeness | 9/10 | Covers creation through merge; tabletop covers all three modes; accumulated findings included |
| Structure | 10/10 | RISEN components map cleanly; steps sequential with clear dependencies |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a governance documentation engineer for the ATHENA project, specializing in CI/CD branch-protection policies and incident-response procedures. You are deeply familiar with GitHub's branch protection API (`gh api`), the GitHub web UI settings pages, and the project's existing break-glass procedure documented in `research/adversarial-reward/governance/GOVERNANCE.md`. You understand the ATHENA research workflow (append-only FINDINGS.md logs, beads-based issue tracking, and the three non-negotiable architectural constraints).

INSTRUCTIONS:
Follow ATHENA's research methodology: read existing artifacts before writing, append-only investigation logs, cite evidence for every claim, and maintain honest limitations. This session is **tabletop/read-only** — all drill steps are documentation walkthroughs, not live executions against the repository. No branch-protection rules, CI gates, or GitHub settings are mutated. If a sandboxed live drill is desired, it must be separately proposed and approved before execution.

Scope reconciliation: The Session 16 handoff identifies three outage scenarios (API-only down, UI-only down, both down). This prompt intentionally collapses "UI down, API up" into Mode A (the default path) because the current break-glass procedure uses the API exclusively — if the UI is down but the API works, the existing procedure is fully operational. This collapse is documented in the Outage Mode Definitions table (Section 3.x) so future reviewers can trace the rationale.

Govern all changes by the existing GOVERNANCE.md structure: new material extends Section 3 (break-glass procedure) by inserting after the current "Restore failure fallback" block and before the "Required post-incident FINDINGS entry" block. Preserve all existing content; do not delete or reorder prior sections.

STEPS:
1. **Create bead and branch.** Run `bd create --title="Session 17: break-glass outage resilience" --description="Close outage-mode gaps in break-glass procedure: add Mode B (API down, UI up) and Mode C (both down) fallback paths, operator decision tree, evidence capture requirements, and tabletop drill." --type=task --priority=2`, mark it in-progress, and create branch `session-17/break-glass-outage-resilience` from `master`.

2. **Read current artifacts.** Read `research/adversarial-reward/governance/GOVERNANCE.md` (full file) and the top of `research/adversarial-reward/FINDINGS.md` (at minimum the Accumulated Findings section — specifically "What We Know / What We Suspect / What We Don't Know" — and the latest log entry) before writing anything.

3. **Extend GOVERNANCE.md Section 3** with four new subsections inserted after the "Restore failure fallback" block:
   - **3.x Outage Mode Definitions** — Table defining Mode A (API+UI up, default path), Mode B (API down, UI up), Mode C (both down). Include a note row: "UI down, API up" is functionally identical to Mode A because the current procedure uses API exclusively; this collapse is intentional (see Session 17 log for rationale).
   - **3.x+1 Mode B — API Down, Web UI Up** — Backup capture via screenshots of Settings > Branches > master protection rule (full toggle states and check names); override by unchecking "Include administrators" (this is `enforce_admins`); restore by re-checking; deferred machine verification via `audit.sh` when API recovers. Evidence requirements: before-override, after-override, and after-restore screenshots; UTC timestamps per phase transition; bead entry labeled "Mode B"; deferred `audit.sh` run within 1 hour of API recovery (document both API-recovery timestamp and `audit.sh` execution timestamp in FINDINGS).
   - **3.x+2 Mode C — Both Down (Containment)** — Freeze all merge activity (operational only: no branch protection mutations possible; communicate hold to any active contributors; record freeze start time in bead). Record outage start (UTC) in `/tmp/governance-outage-$TS.txt` and bead. Do NOT make untracked policy changes (rules still active server-side). Monitor API (`gh api repos/andrewmcadoo/athena/branches/master/protection`) and UI (`https://github.com/andrewmcadoo/athena/settings/branches`) every 15 min. Max containment window: 2 hours, then escalate to GitHub Support with ticket ID in bead. On recovery: resume via Mode A or B (whichever path recovers first). Run `audit.sh` as soon as API is available.
   - **3.x+3 Operator Decision Tree** — ASCII flowchart: break-glass needed → test `gh api .../protection` → YES=Mode A (max 1h override) / NO → test web UI → YES=Mode B (max 1h override) / NO=Mode C (max 2h containment, 15-min checks, escalate after 2h). Post-action for all modes: (1) restore enforce_admins=true, (2) run audit.sh — Mode A: immediately; Mode B/C: defer until API recovery, must execute within 1h of recovery, (3) log post-incident entry in FINDINGS.md, (4) close remediation bead. Mode A evidence: backup JSON in `/tmp/master-protection-backup-$TS.json`, override duration logged, `audit.sh` output (7/7 PASS), bead closed. Mode B evidence: screenshots + timestamps + deferred audit. Mode C evidence: outage log file + bead timestamps + recovery-mode audit.

4. **Run tabletop drill (documentation-only)** for all three modes:
   - **Mode A**: Confirm existing procedure reads cleanly within the new structural context. Reference Session 16 timing (~6 min total).
   - **Mode B**: Walk through screenshot-backup → uncheck-enforce-admins → emergency-change → re-check-enforce-admins → screenshot-restore → deferred-audit.sh. Estimate timing. Identify any step where the UI path is ambiguous.
   - **Mode C**: Walk through freeze-merges → record-outage-time → 15-min-monitor-cycle → recovery-detection → resume-via-A-or-B. Estimate containment-to-recovery timing. Evaluate whether 2-hour window and 15-min check interval are realistic; state what evidence would be needed to calibrate these windows if no empirical anchor exists.
   - Log drill results, timing estimates, and any remaining gaps.

5. **Log Session 17 in FINDINGS.md.** Add investigation log entry at top of Investigation Log (reverse-chronological) with: Scope, Method, Findings, Implications, Open Threads. Include tabletop drill narrative and timing estimates. Use commit message format: `research(adversarial-reward): Session 17 — break-glass outage resilience`.

6. **Update Accumulated Findings** (specifically the "What We Know" and "Open Threads" categories). Move resolved threads from Session 16:
   - "Add explicit web-UI fallback for backup/override steps" → resolved by Mode B (cite Session 17 log entry)
   - "Define contingency for full GitHub API+UI outage" → resolved by Mode C (cite Session 17 log entry)
   - Keep "sandboxed live drill" as open if not addressed this session.

7. **Commit, PR, verify, merge.** Stage governance changes, push branch, open PR to `master`, verify CI passes (contract-gate), close bead, `bd sync`.

END GOAL:
GOVERNANCE.md Section 3 contains complete, unambiguous fallback procedures for every GitHub outage scenario (API-only, UI-only, both-down), with:
- An operator decision tree that has explicit time windows at every decision point
- Per-mode evidence capture requirements (screenshots vs JSON vs timestamps) — including Mode A for internal parallelism
- No ambiguous steps in any outage path
- FINDINGS.md has a complete Session 17 log entry with tabletop drill results and timing estimates
- Accumulated Findings reflects resolved threads with citations
- CI passes on the PR

NARROWING:
- Do NOT mutate live repository state (branch protection rules, CI gates, GitHub settings). This is a tabletop/read-only session.
- Do NOT run `audit.sh` against live endpoints during the drill. Drill steps are walkthroughs of the documentation only.
- Do NOT delete or reorder existing GOVERNANCE.md content. New subsections are inserted, not replacements.
- Do NOT edit prior FINDINGS.md investigation log entries. The log is append-only; new entry goes at the top.
- Do NOT approve or execute a sandboxed live drill without separate, explicit user approval.
- Stay within the governance/break-glass scope. Do not extend to other ATHENA research areas (trace semantics, structural priors, etc.).
- Avoid grant-proposal rhetoric. Use precise, operational language throughout.
- Out of scope: production code, new ADRs, changes to `audit.sh` logic, modifications to `contract-gate.yml`.
```

---

## Review Findings

### Issues Addressed
- **W1 (scope reconciliation):** Added explicit paragraph in Instructions acknowledging the handoff lists three outage modes and documenting the intentional collapse of "UI down, API up" into Mode A with rationale.
- **W2 (Mode A evidence):** Added parallel evidence capture line for Mode A in the decision tree subsection (3.x+3): backup JSON, override duration, audit.sh output, bead closed.
- **W3 (deferred audit window):** Added "within 1 hour of API recovery" and dual-timestamp requirement to Mode B subsection (3.x+1).
- **W4 (freeze mechanism):** Added parenthetical to Mode C freeze instruction clarifying it is operational-only with contributor communication and bead recording.

### Remaining Suggestions
- **S1 (Mode C timing anchor):** Mode C's 2-hour/15-minute windows lack empirical calibration data. Added prompt instruction to "state what evidence would be needed to calibrate these windows" so the model evaluates rather than fabricates.
- **S2 (Accumulated Findings section headers):** Added explicit section category names ("What We Know" and "Open Threads") to Step 6.
- **S3 (commit message format):** Added explicit commit message format to Step 5.
- **S4 (bead description):** Added `--description` argument to the `bd create` command in Step 1.

## Usage Notes

- **Best used with:** Claude Opus 4.6 in Claude Code CLI with beads workflow and git hooks active
- **Adjust for:** If a sandboxed live drill is later approved, convert drill steps from documentation walkthroughs to executable commands (requires separate prompt or prompt extension)
