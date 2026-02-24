# Session Handoff: Break-Glass Outage Resilience

> Generated: 2026-02-23 | Handoff #24 | Previous: handoff_023_2026-02-23_first-live-audit-drill-session15.md

---

## Continuation Directive

Session 17 should focus on break-glass outage resilience — the "fire escape map" for when GitHub controls are partially unavailable. Update GOVERNANCE.md Section 3 with explicit fallback paths for three outage scenarios (API down/UI up, UI down/API up, both down), define evidence capture requirements per path, add an operator decision tree with time windows, run a tabletop drill against the new text, and log results in FINDINGS.md. Exit criterion: no ambiguous steps remain for backup, override, or restore during outage conditions.

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist. The adversarial-reward research track has a locked AggregateScore contract enforced by CI (`contract-gate.yml`) and branch protection on `master`.

**Goal:** Close the last high-value governance gap from Session 16: the break-glass procedure (GOVERNANCE.md Section 3) assumes GitHub API is available. Define what to do when it isn't.

**Success criteria:**
1. GOVERNANCE.md Section 3 has fallback paths for three outage modes (API-only down, UI-only down, both down)
2. Each fallback path specifies what evidence to capture (screenshots, timestamps, fields to verify)
3. An operator decision tree ("if this fails, do this next") with max time windows is documented
4. A tabletop drill against the new fallback text is logged in FINDINGS.md with timing estimates and gap analysis
5. No ambiguous steps remain for backup, override, or restore under any outage condition

## Key Decisions & Rationale

1. **Non-mutating governance sessions by default.** All audit/drill work is read-only unless the user explicitly approves live mutation. Rationale: governance artifacts protect `master`; accidental mutation during testing would be ironic. Established Session 16.

2. **`audit.sh` co-located with `GOVERNANCE.md`, not in `prototypes/`.** It's operational tooling, not a throwaway research prototype. Rationale: the script implements the runbook defined in GOVERNANCE.md and is operationally coupled to it. Deliberate exception to CLAUDE.md's "prototypes in `prototypes/` only" rule. Established Session 16 prompt (#25).

3. **Escalation thresholds are per-check, not per-severity-only.** Each C1-C7 failure maps to a specific severity, owner action, and response time. Rationale: different checks fail in different ways with different governance impact. A blanket "CRITICAL = fix now" isn't specific enough for operator triage. Established Session 16.

4. **C7 comparison by fields, not serialized JSON.** `jq` key ordering is not guaranteed, so `audit.sh` compares `status == "completed"` AND `conclusion == "success"` separately. Rationale: string equality on serialized objects caused a false FAIL during Session 16 live validation. Fixed in `audit.sh` line 73.

5. **FINDINGS.md is authoritative over handoff documents.** For any discrepancy, FINDINGS.md is source of truth. Established Session 14.

6. **Governance scoped to adversarial-reward track.** `governance/` lives under `research/adversarial-reward/`, not at repo top-level. Rationale: artifacts are specific to the AggregateScore contract.

## Current State

### Completed
- **Session 16** (commit `a798d61`, PR #9, merged): Created `audit.sh` (read-only C1-C7 automation with `--dry-run`), inserted escalation thresholds table into GOVERNANCE.md, ran tabletop break-glass drill (documented timing estimates), logged Session 16 in FINDINGS.md. Bead `athena-x3i` closed.
- **Session 15** (commit `02c67aa`, PR #8, merged): First live audit drill — all 7/7 checks PASS. Defined audit cadence (weekly spot-check, mandatory pre-merge, post-incident). Bead `athena-wm3` closed.
- **Sessions 11-14**: Built governance chain: CI gate (S11) → branch protection enforcement proof (S12) → admin bypass closure (S13) → runbook codification (S14).
- **Sessions 1-10**: AggregateScore research (reward function formalization, hybrid candidate, locked recommendation, acceptance tests, monitoring hooks).

### In Progress
- Nothing. Session 17 is the next piece of work.

### Blocked / Open Questions
- **Session 16 open thread #1:** Add explicit web-UI fallback for backup/override steps, not only restore. (This is the primary target for Session 17.)
- **Session 16 open thread #2:** Define contingency for full GitHub API+UI outage. (Also targeted by Session 17.)
- **`bd list` panic**: Non-reproducing since Session 14. Workaround: `bd list --status=open`.

## Key Code Context

**`research/adversarial-reward/governance/GOVERNANCE.md`** — The break-glass procedure to extend (Section 3, starting at line 200)

Current Section 3 assumes `gh api` is available for backup capture, override, and restore. The web UI fallback (line 236) is mentioned only for restore failure:
```markdown
- Restore failure fallback:
  - If API restore fails, manually restore in GitHub web UI: `Settings` -> `Branches` -> `master` protection rule.
  - Document manual restore path and verification evidence in FINDINGS.
```

This is the gap: backup and override steps have no UI fallback, and there's no guidance for when both API and UI are down.

**`research/adversarial-reward/governance/audit.sh`** — Read-only audit automation

Session 17 may need to reference this for post-incident re-audit steps in the fallback paths. Key: the script exits 0 on all-pass, 1 on any failure. Supports `--dry-run`.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/adversarial-reward/governance/GOVERNANCE.md` | Audit runbook + baseline + break-glass + escalation | To be extended S17 |
| `research/adversarial-reward/governance/audit.sh` | Read-only C1-C7 automation | Complete S16, reference only |
| `research/adversarial-reward/FINDINGS.md` | Investigation log (Sessions 1-16) | Needs S17 entry |
| `CLAUDE.md` | Project governance index | Updated this handoff (audit.sh added) |
| `.github/workflows/contract-gate.yml` | CI gate for contract enforcement | Stable since S11 |
| `research/adversarial-reward/prototypes/aggregation-candidates/` | Locked research prototypes | Stable since S10 |

## Loop State

N/A — single-session work, not a Claude→Codex loop.

## Next Steps

1. **Read GOVERNANCE.md Section 3** (lines 200-252) and the Session 16 FINDINGS entry (tabletop drill findings) to understand the current break-glass procedure and identified gaps.
2. **Create bead and branch** (`session-17/break-glass-outage-resilience`).
3. **Add three fallback subsections to Section 3:**
   - API down, Web UI up: manual steps via `Settings → Branches → master protection rule` for backup (screenshot current state), override (uncheck enforce_admins), and restore (re-check enforce_admins).
   - Web UI down, API up: all steps remain as-is (current procedure uses API). Note explicitly that this is the default path.
   - Both down: containment rules — pause all merges, document the outage start time, do NOT make untracked policy changes. Resume procedure when either path becomes available. Max containment window before escalating (e.g., 2 hours).
4. **Define evidence capture per path:** what to screenshot, what timestamps to log, what fields to verify post-restoration.
5. **Add operator decision tree:** "if `gh api` fails → try web UI → if web UI fails → containment mode" with max time windows at each decision point.
6. **Run tabletop drill** against the new text for all three outage scenarios. Document narrative, timing, and any remaining gaps.
7. **Log Session 17** in FINDINGS.md (append-only, top of Investigation Log).
8. **Update Accumulated Findings** — move open threads to "What We Know" if resolved.
9. **Commit, push, PR, verify CI, merge, close bead, `bd sync`.**

## Session Artifacts

- **Prompt #25:** `.claude/prompts/prompt_025_2026-02-23_session-16-audit-automation-drill.md` (RISEN prompt for Session 16 — includes escalation table and audit.sh spec)
- **Handoff #23:** `.claude/handoffs/handoff_023_2026-02-23_first-live-audit-drill-session15.md` (Session 15 handoff)
- **PR #9:** https://github.com/andrewmcadoo/athena/pull/9 (Session 16 merged)
- **PR #8:** https://github.com/andrewmcadoo/athena/pull/8 (Session 15 merged)

## Documentation Updated

| Document | Change Summary | Status |
|----------|---------------|--------|
| `CLAUDE.md` | Added `audit.sh` to Key Artifacts and Directory Structure | Approved and applied |
