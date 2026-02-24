# Prompt: CI Gate for AggregateScore Contract Enforcement

> Generated: 2026-02-22 | Prompt #21 | Framework: RISEN

---

## Session Goal

Implement a GitHub Actions CI gate that runs both acceptance_test.py (15 contract checks) and monitoring_hooks.py (20 drift checks) from `research/adversarial-reward/prototypes/aggregation-candidates` on every push/PR to master, failing on any non-zero exit and uploading logs as artifacts. Session wrap-up includes FINDINGS.md Session 11 entry, bead management, and commit/push.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step process (create YAML → verify locally → update FINDINGS.md → manage beads → commit/push) with clear sequential dependencies, explicit methodology, and important constraints. RISEN's Steps + End Goal + Narrowing map directly.
- **Alternatives considered:** TIDD-EC (good for explicit dos/don'ts but the task is fundamentally sequential/procedural, not constraint-heavy precision work)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal, steps, and gate logic are unambiguous |
| Specificity | 9/10 | Exact file paths, action versions, expected counts, commit message all specified |
| Context | 9/10 | Project context established; handoff #21 referenced; bead syntax explicit |
| Completeness | 9/10 | All 5 steps + end goal + narrowing; Open Threads and Accumulated Findings included |
| Structure | 9/10 | Clean RISEN sections, numbered steps with sub-bullets, bulleted end goal |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a DevOps-aware research engineer working on the ATHENA project. You have expertise in GitHub Actions CI/CD, Python testing scripts, and research-project governance (FINDINGS.md append-only logs, bead-based issue tracking). You understand that prototypes in this project are research artifacts, not production code.

INSTRUCTIONS:
Wire two existing verification scripts — acceptance_test.py (15 contract checks) and monitoring_hooks.py (20 drift checks) — into a GitHub Actions CI gate so that the locked AggregateScore contract cannot drift without failing CI. Follow a run-both-then-gate pattern: both scripts execute regardless of individual failure, and a final gate step fails the job if either returned non-zero. Capture script output as downloadable artifacts for debugging. All session work must be recorded in FINDINGS.md (Session 11 log entry, plus Accumulated Findings update) and tracked via beads before committing and pushing.

STEPS:
1. Create `.github/workflows/contract-gate.yml` with:
   - Trigger: push and pull_request on master branch
   - Single job `contract-verification` on ubuntu-latest, Python 3.12
   - `defaults.run.working-directory` set to `research/adversarial-reward/prototypes/aggregation-candidates` (scripts import local modules and resolve paths from __file__)
   - Step: Checkout via actions/checkout@v4
   - Step: Setup Python via actions/setup-python@v5 (python-version: "3.12")
   - Step: Run `python acceptance_test.py` with `tee` to `$GITHUB_WORKSPACE/acceptance-output.txt` (id: acceptance)
   - Step: Run `python monitoring_hooks.py` with `tee` to `$GITHUB_WORKSPACE/monitoring-output.txt` (id: monitoring); use `if: always() && steps.acceptance.outcome != 'cancelled'` so it runs even if acceptance fails (skip only if the job was cancelled by user/concurrency)
   - Step: Upload both output files as artifact `contract-gate-output` (retention 30 days) via actions/upload-artifact@v4
   - Step: Gate result — use this exact pattern:
     ```yaml
     - name: Gate result
       if: always() && steps.acceptance.outcome != 'cancelled'
       run: |
         echo "acceptance: ${{ steps.acceptance.outcome }}"
         echo "monitoring: ${{ steps.monitoring.outcome }}"
         if [ "${{ steps.acceptance.outcome }}" != "success" ] || [ "${{ steps.monitoring.outcome }}" != "success" ]; then
           echo "::error::Contract gate failed. See uploaded artifacts for details."
           exit 1
         fi
         echo "Contract gate passed."
     ```
   - No pip install (all imports are stdlib or local)
   - No path filters (scripts take <2s; unconditional runs avoid silent skips)

2. Verify locally by running both scripts from the `aggregation-candidates/` directory:
   - `python acceptance_test.py` → expect 15/15, exit 0
   - `python monitoring_hooks.py` → expect 20/20, exit 0

3. Add Session 11 log entry to `research/adversarial-reward/FINDINGS.md`:
   - Insert at top of Investigation Log (reverse chronological, per CLAUDE.md protocol)
   - Header: `### 2026-02-22 -- Session 11: CI Gate for AggregateScore Contract Enforcement`
   - Required sections: Scope, Method, Findings (filled after local verification), Implications, Open Threads
   - Follow exact format of Sessions 9-10
   - Also update Accumulated Findings "What We Know" section to reflect that contract enforcement now includes automated CI gating, citing Session 11

4. Manage beads:
   - Check `bd ready` / `bd list --status=open` for a bead related to CI gate or contract enforcement (context: handoff #21 from commit e7c502b titled "CI gate for contract enforcement" may have created one)
   - If a relevant bead exists: `bd update <id> --status=in_progress`
   - If none exists: `bd create --title="CI gate for AggregateScore contract enforcement" --description="Wire acceptance_test.py and monitoring_hooks.py into GitHub Actions CI pipeline. Session 11 of adversarial-reward investigation." --type=task --priority=2`
   - After commit: `bd close <id>`
   - If `bd sync` fails at any point, report the error and proceed — do not block on sync failures

5. Commit and push:
   - Stage: `.github/workflows/contract-gate.yml` and `research/adversarial-reward/FINDINGS.md`
   - Run `bd sync` before and after commit (non-blocking on failure)
   - Commit message: `research(adversarial-reward): Session 11 — CI gate for contract enforcement`
   - Push to remote

END GOAL:
After execution, ALL of the following must be true:
- `.github/workflows/contract-gate.yml` exists and is syntactically valid
- Every push/PR to master triggers both acceptance_test.py and monitoring_hooks.py
- CI fails if either script exits non-zero; passes only when both succeed
- Script stdout is captured as downloadable GitHub Actions artifacts
- Both scripts pass locally (15/15 acceptance, 20/20 monitoring) on the current commit
- FINDINGS.md contains a Session 11 entry with Scope, Method, Findings, Implications, and Open Threads
- FINDINGS.md Accumulated Findings "What We Know" is updated to reflect CI gating
- A bead tracks this session and is closed
- Changes are committed and pushed to remote

NARROWING:
- Do NOT add pip install steps — all dependencies are stdlib or local
- Do NOT use path filters on the workflow trigger — run unconditionally
- Do NOT use fail-fast behavior — both scripts must always run (run-both-then-gate)
- Do NOT edit or delete previous Investigation Log entries — FINDINGS.md is append-only
- Do NOT modify acceptance_test.py or monitoring_hooks.py — they are locked contract verification scripts
- Do NOT write production code — this is a CI configuration for research artifacts
- Do NOT add concurrency controls — scripts take <2s, not worth the complexity
- Stay within: Python 3.12 (scripts use 3.10+ syntax like `X | Y` unions)
- Avoid: using `bd edit` (opens $EDITOR which blocks agents); use `bd update`/`bd close` instead
```

---

## Review Findings

### Issues Addressed
1. **[Warning] Bead handling underspecified** — Added explicit `bd create` syntax with title/description, reference to handoff #21, and conditional logic for existing vs. new beads
2. **[Warning] Missing Accumulated Findings update** — Added sub-step under Step 3 to update "What We Know" section, citing Session 11
3. **[Warning] Missing "Open Threads" section** — Added to required sections list in Step 3
4. **[Warning] `bd sync` failure handling** — Added "non-blocking on failure" guidance to Steps 4 and 5
5. **[Suggestion] Gate step YAML snippet** — Incorporated inline YAML example in Step 1 for zero-ambiguity implementation

### Remaining Suggestions
- ROLE section could be tightened (redundancy with INSTRUCTIONS) — minor, not harmful
- `if:` condition rationale for `!= 'cancelled'` — added inline parenthetical explanation
- SHA pinning for actions — tag pinning is appropriate for research project; not worth the maintenance overhead
- Concurrency exclusion — added to NARROWING section

## Usage Notes

- **Best used with:** Claude Code or any agentic coding assistant with file editing, bash execution, and git access
- **Adjust for:** Different Python version if scripts are updated; different bead ID if handoff #21 already created one
