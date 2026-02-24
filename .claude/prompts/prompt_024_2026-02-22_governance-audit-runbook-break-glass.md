# Prompt: Governance Audit Runbook and Break-Glass Procedure

> Generated: 2026-02-22 | Framework: RISEN

---

## Session Goal

Codify the governance chain built across Sessions 11-13 (CI gate, branch protection, admin bypass closure) into a repeatable audit runbook with exact commands, expected pass/fail output, and an evidence template for reproducibility; plus a break-glass procedure with explicit guardrails (approval authority, max duration, restore steps, required post-incident log entry). Update FINDINGS.md and CLAUDE.md to reference it, and ship via PR.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step procedural task (6 sequential steps with dependencies) requiring both process methodology and explicit constraints. RISEN's Steps + Narrowing components cover both the workflow and the precision requirements.
- **Alternatives considered:** TIDD-EC — strong on dos/don'ts but weaker at expressing sequential workflow with dependencies.

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Unambiguous goal, each step defined with exact artifacts |
| Specificity | 9/10 | Exact commands, field names, file paths, section contents |
| Context | 8/10 | References Sessions 11-13, CLAUDE.md workflow, handoff discrepancy |
| Completeness | 9/10 | All 6 steps, verification, user's two additions (exact commands + guardrails) |
| Structure | 9/10 | Clean RISEN hierarchy with nested subsections |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a DevOps governance engineer specializing in branch protection, CI/CD security, and operational runbook design. You have expertise in GitHub branch protection APIs, shell scripting for audits, and incident response procedures for solo and small-team repositories.

INSTRUCTIONS:
Follow the Athena project's research workflow (CLAUDE.md): read FINDINGS.md before starting, append-only investigation log (reverse chronological), cite evidence for all claims. Use beads (`bd`) for task tracking. Ship changes via feature branch PR. FINDINGS.md is authoritative over session handoffs for discrepancies — always verify live state before embedding baselines.

STEPS:
1. **Verify live branch protection state.** Run `gh api repos/{owner}/{repo}/branches/{branch}/protection` to capture the current protection configuration. Resolve repo identity first (`gh repo view --json nameWithOwner --jq .nameWithOwner`). Confirm critical fields: `required_status_checks.contexts`, `required_status_checks.strict`, `enforce_admins.enabled`, `allow_force_pushes.enabled`. If live state differs from both handoff and FINDINGS.md, log the discrepancy as a finding, investigate the cause, and use live state as the authoritative baseline.

2. **Create `research/adversarial-reward/governance/GOVERNANCE.md`** with four sections:

   **Section 1 — Must-Stay-True Baseline:**
   - Embedded JSON block with the exact expected branch protection state (from verified live output)
   - Table of critical fields: field name, required value, rationale for each
   - CI workflow contract table: triggers, scripts, gate logic, pipefail requirement
   - Evidence chain table linking prior sessions to their proofs

   **Section 2 — Five-Minute Audit Runbook:**
   - 5 numbered steps, each with exact `gh api` command + `--jq` filter + expected output
     1. Resolve repo identity (`gh repo view --json nameWithOwner --jq .nameWithOwner`)
     2. Check branch protection fields — 4 individual checks, one per field (contexts, strict, enforce_admins, force_pushes), yielding 7 total checks across all steps
     3. Verify CI workflow is active (`gh api .../actions/workflows --jq ...`)
     4. Check most recent workflow run conclusion
     5. Record result: pass → update "Last audited" date; fail → stop, open bead, log in FINDINGS.md
   - Pass/fail summary table: 7 checks, each with jq path, expected pass value, fail indicator
   - Evidence template: date, auditor, commit SHA, pass/fail per check, notes field

   **Section 3 — Break-Glass Procedure:**
   - Prerequisites: admin access, document reason BEFORE override
   - Override steps: capture backup JSON → disable enforce_admins via PUT → make emergency change → document
   - Guardrails:
     - Approval: self-approval with mandatory documentation (solo project), upgradeable to two-admin approval if team grows
     - Max duration: 1 hour — mandatory restoration within this window
     - Restore steps: re-enable enforce_admins → re-verify via full audit runbook → close bead
     - Restore failure fallback: if API restore fails, manually verify via GitHub web UI (Settings > Branches) and document the manual restore path
     - Required post-incident log entry in FINDINGS.md with: trigger, duration, what was changed, restore verification
   - Explicit "what is NOT a break-glass" list: CI failures from real drift, `bd` panics, unrelated PR failures

   **Section 4 — Known Issues:**
   - `bd list` panic: non-reproducing, workaround (`bd list --status=open`), no fix needed
   - Note: this is a living list, intentionally minimal at initial creation

3. **Update `research/adversarial-reward/FINDINGS.md`:**
   - Prepend a Session 14 investigation log entry (before Session 13) in standard format: Scope, Method, Findings, Implications, Open Threads
   - Key findings: 7 auditable checks codified, `strict` field state confirmed against live API (if live state differs from Accumulated Findings, update the living synthesis section accordingly), `bd list` panic non-reproducing
   - Append to Accumulated Findings "What We Know": governance audit and break-glass procedures codified

4. **Update `CLAUDE.md`:**
   - Add `governance/` directory and `GOVERNANCE.md` to the directory structure tree under `adversarial-reward/`
   - Add `GOVERNANCE.md` to Key Artifacts section with description

5. **Beads management:** `bd create` → `bd update --status=in_progress` → work → `bd close`

6. **Ship via PR:**
   - Branch: `session-14/governance-runbook`
   - Stage the three files: GOVERNANCE.md, FINDINGS.md, CLAUDE.md
   - Commit: `research(adversarial-reward): Session 14 — governance audit runbook and break-glass procedure`
   - Push, verify `contract-verification` CI passes (GOVERNANCE.md is Markdown-only, should not affect contract scripts), merge, delete branch

END GOAL:
A merged PR containing:
- `GOVERNANCE.md` with a reproducible 5-minute audit runbook (7 checks, each with exact command, jq filter, expected output, and pass/fail criteria), an evidence template for audit logging, and a break-glass procedure with explicit guardrails (self-approval + documentation, 1-hour max override, mandatory restore + re-verify, post-incident log entry, manual fallback path)
- Updated FINDINGS.md with Session 14 investigation log and accumulated findings
- Updated CLAUDE.md with governance directory and artifact reference
- All 7 audit checks passing against live repo after merge
- CI (`contract-verification`) green

NARROWING:
- Do NOT create production code — GOVERNANCE.md is a reference document, not a prototype
- Do NOT edit previous FINDINGS.md log entries (append-only protocol)
- Do NOT place governance/ at repo top-level — it scopes to the adversarial-reward research track (governance artifacts are specific to the AggregateScore contract, not repo-wide governance)
- Do NOT treat CI failures from real contract drift as break-glass scenarios — those require investigation
- Do NOT soften limitations or downgrade severity ratings from VISION.md
- Do NOT use `bd edit` (opens $EDITOR, blocks agents)
- Do NOT skip live verification in Step 1 — handoff discrepancies exist and must be resolved from the source of truth
- Stay within the Athena research workflow: beads tracking, feature branch PRs, FINDINGS.md protocol
```

---

## Review Findings

### Issues Addressed
1. **`strict` discrepancy pre-assumption** (Warning) — Rephrased Step 3 finding to "confirmed against live API" with guidance for living synthesis update
2. **5 steps vs 7 checks ambiguity** (Warning) — Added parenthetical mapping: Step 2 contains 4 individual checks
3. **No divergence handling in Step 1** (Warning) — Added conditional: log, investigate, use live state as baseline
4. **Break-glass restore failure path** (Warning) — Added manual GitHub web UI fallback

### Remaining Suggestions
- Section 4 could note it's a living list (addressed in prompt)
- Post-merge audit could be explicit Step 7
- CI pass assumption could note Markdown-only change (addressed in prompt)
- Repo identity resolution ordering (runbook itself orders correctly)

## Usage Notes

- **Best used with:** Claude Opus or Sonnet in Claude Code with `gh` CLI authenticated
- **Adjust for:** Different repo owner/name if forked; team size changes affect break-glass approval policy
