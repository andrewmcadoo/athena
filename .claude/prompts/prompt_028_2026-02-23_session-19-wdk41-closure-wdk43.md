# Prompt: Session 19 — WDK#41 Closure + WDK#43 Convergence-Summary Derivation

> Generated: 2026-02-23 | Prompt #028 | Framework: RISEN

---

## Session Goal

Close stale WDK#41 bookkeeping in trace-semantics FINDINGS.md (work completed in adversarial-reward Sessions 1–7), then begin WDK#43 (GROMACS/OpenMM convergence-summary derivation rules) as the next trace-semantics execution step — with no adversarial-reward/governance work unless it's a blocker.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Two-phase sequential process (close bookkeeping → begin WDK#43 research) with strict methodology constraints (ATHENA's append-only logs, read-before-write, cite evidence). Narrowing component enforces the scope guard against adversarial-reward drift. Steps component handles the phase transition and dependency ordering.
- **Alternatives considered:** TIDD-EC (good for scope-guard dos/don'ts but weaker on sequential step design), RTF (too lightweight for a two-phase session with cross-file consistency requirements).

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Two-phase structure with explicit gate (Phase 1 before Phase 2); each phase has concrete deliverables |
| Specificity | 9/10 | Bead IDs, file paths, line numbers, WDK numbers, cross-references all concrete |
| Context | 9/10 | References existing prototype state (92/92 tests), adapter architecture, WDK dependency chain |
| Completeness | 9/10 | Covers bookkeeping closure, prototype extension, testing, FINDINGS update, accumulated findings sync |
| Structure | 10/10 | RISEN components map cleanly; strict phase ordering enforced in Instructions |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a trace-semantics research engineer for the ATHENA project, working on the LEL+DGR Hybrid IR prototype. You are resuming the trace-semantics research track after adversarial-reward governance work (project Sessions 11–18). Within the adversarial-reward track, WDK#41 itself progressed through adversarial-reward internal Sessions 1–7 and is fully resolved there. You are familiar with: the LEL IR type system (EventKind variants including ConvergencePoint in `event_kinds.rs`, EnergyRecord, NumericalStatus, ExecutionStatus), the three adapter implementations (OpenMM, GROMACS, VASP), the CausalOverlay construction pipeline, and the WDK tracking system in FINDINGS.md.

INSTRUCTIONS:
This session has two phases with a strict ordering: bookkeeping closure first, then new research execution. Do not begin WDK#43 work until WDK#41 is cleanly closed in both the bead tracker and trace-semantics FINDINGS.md.

Follow ATHENA's research methodology: read existing artifacts before writing, append-only investigation logs, cite evidence for every claim. For WDK#43, the investigation approach is prototype-driven: extend the existing Rust prototype crate to derive convergence summaries from GROMACS and OpenMM event streams, validate with tests, and log findings.

Scope guard: no adversarial-reward or governance work in this session unless it is a direct blocker for trace-semantics progress. If a governance issue surfaces, log it as an open thread and move on.

STEPS:
1. **Create bead and branch.** Run `bd create --title="Session 19: WDK#41 closure + WDK#43 convergence-summary derivation" --description="Phase 1: close stale WDK#41 bookkeeping (bead athena-apb, trace-semantics FINDINGS.md). Phase 2: begin WDK#43 — derive convergence summaries for GROMACS/OpenMM from existing event streams." --type=task --priority=1`, mark it in-progress, and create branch `session-19/wdk43-convergence-summaries` from `master`.

2. **Read current artifacts.** Read:
   - `research/trace-semantics/FINDINGS.md` — full Accumulated Findings section ("What We Know / What We Suspect / What We Don't Know") and the latest investigation log entry.
   - `research/adversarial-reward/FINDINGS.md` — the WDK#41 Session 7 log entry (line ~643) and accumulated findings entries referencing WDK#41 resolution (lines ~1138–1174), to source cross-references.
   - `research/trace-semantics/prototypes/lel-ir-prototype/src/event_kinds.rs` — ConvergencePoint type definition (struct fields: iteration, metric_name, metric_value, converged).
   - `research/trace-semantics/prototypes/lel-ir-prototype/src/lel.rs` — TraceEvent, EventLog, EventIndexes, and TraceEventBuilder types.
   - `research/trace-semantics/prototypes/lel-ir-prototype/src/gromacs_adapter.rs` — current GROMACS adapter (no ConvergencePoint emission).
   - `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs` — DslAdapter trait and OpenMM adapter (no ConvergencePoint emission).
   - `research/trace-semantics/prototypes/lel-ir-prototype/src/vasp_adapter.rs` — VASP adapter (has ConvergencePoint emission — reference implementation).

3. **Phase 1: Close WDK#41 bookkeeping.**
   - 3a. Close bead `athena-apb` with reason citing adversarial-reward resolution: `bd close athena-apb --reason="WDK#41 resolved in adversarial-reward track (Sessions 1-7). AggregateScore recommendation locked, architecture integration complete, CI gate and governance enforcement verified. See adversarial-reward FINDINGS.md and locked recommendation at research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md."`.
   - 3b. In trace-semantics FINDINGS.md, move WDK#41 (line 1855) from "What We Don't Know" to "Resolved / Narrowed — No Longer Blocking" with strikethrough and resolution note: "RESOLVED: Work completed in adversarial-reward track. See adversarial-reward/FINDINGS.md WDK#41 Sessions 1–7, locked recommendation at `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md`, and architecture integration in Session 7."
   - 3c. Verify consistency in the **Accumulated Findings sections only** (What We Know, What We Suspect, What We Don't Know): no item references WDK#41 as still open. If any do, update them with the resolution citation. **Do not touch any investigation log entry** — the append-only rule applies. If a log entry's Open Threads section lists WDK#41 as open, that is historical record; do not edit it.
   - 3d. Flag bead `athena-fom` ("Branch protection enforcement proof for contract gate") as potentially stale in the Session 19 FINDINGS.md open threads. Defer its closure to a dedicated cleanup session. Do not investigate it further in this session, even if evidence appears conclusive — closure is out of scope for this session.

4. **Phase 2: WDK#43 investigation — convergence-summary derivation rules.**
   - 4a. Study the VASP adapter's ConvergencePoint emission pattern as the reference implementation. Identify: what input data triggers ConvergencePoint creation, what fields are populated, how it integrates with the event stream, and what causal references it carries.
   - 4b. For GROMACS: identify which existing events (`EnergyRecord`, `NumericalStatus`, `ExecutionStatus`) can serve as source data for convergence summary derivation. Design derivation rules that: (i) detect convergence/stalling/oscillation patterns from energy time series, (ii) produce ConvergencePoint events without introducing synthetic certainty, and (iii) preserve provenance (which source events contributed to the summary). Define the minimum input data condition explicitly (e.g., minimum window length for trend detection) — this condition governs the "insufficient data" test case in Step 4e.
   - 4c. For OpenMM: same analysis as 4b, adapted to OpenMM's event vocabulary and reporter-based output structure. Define minimum input data condition.
   - 4d. Implement the derivation logic in the prototype crate. Default to the per-adapter approach for consistency with the existing VASP pattern (inline emission during parsing). Only choose a shared derivation module if the derivation logic is demonstrably identical across both adapters after the analysis in 4b/4c. Document the choice and rationale.
   - 4e. Write tests validating: (i) convergence detection on a synthetic stable energy series, (ii) stall/oscillation detection on a synthetic non-converging series, (iii) no ConvergencePoint emitted when input data is below the minimum condition defined in 4b/4c, (iv) provenance references are correct (source event IDs traced to derived ConvergencePoint).
   - 4f. Run `cargo test` and `cargo clippy -- -D warnings` to confirm the prototype remains clean.

5. **Log Session 19 in trace-semantics FINDINGS.md.** Add investigation log entry at top of Investigation Log (reverse-chronological) with:
   - **Scope**: WDK#41 bookkeeping closure + WDK#43 convergence-summary derivation rules for GROMACS and OpenMM.
   - **Method**: Cross-track resolution verification (WDK#41), reference-implementation study (VASP ConvergencePoint), prototype extension with derivation rules and tests.
   - **Findings**: WDK#41 closure evidence, derivation rule design, minimum data conditions, test results, prototype state (test count, clippy status).
   - **Implications**: How WDK#43 resolution affects WDK#42 (taxonomy) and WDK#44 (placement).
   - **Open Threads**: Remaining gaps from derivation rule design, any WDK#42/#44 items unblocked, `athena-fom` stale-bead flag.

6. **Update Accumulated Findings.**
   - Add new "What We Know" entries for WDK#43 findings with evidence citations.
   - If WDK#43 is fully resolved, move it from "What We Don't Know" to "Resolved" with citation.
   - If only narrowed, update the WDK#43 entry in "What We Don't Know" with current status and reduced scope.
   - Update WDK#42 and WDK#44 entries if WDK#43 findings change their scope or unblock them.

7. **Update Prototype Index.** If new source files or significant modifications were made, update the Prototype Index table in trace-semantics FINDINGS.md.

8. **Commit, PR, verify, merge.** Commit message: `research(trace-semantics): Session 19 — WDK#41 closure + WDK#43 convergence-summary derivation`. Push branch, open PR to `master`, verify CI passes (contract-gate), merge, close bead, `bd sync`.

END GOAL:
- WDK#41 is cleanly closed: bead `athena-apb` closed, trace-semantics FINDINGS.md entry moved to "Resolved" with cross-track citation to `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md`.
- WDK#43 investigation is complete or narrowed: derivation rules for GROMACS and OpenMM convergence summaries are designed, prototyped, and tested.
- Prototype crate compiles clean with all tests passing and clippy zero warnings.
- FINDINGS.md has Session 19 log entry with full evidence chain.
- Accumulated Findings reflect both the WDK#41 closure and WDK#43 outcomes.
- WDK#42 and WDK#44 status updated if unblocked by WDK#43.
- `athena-fom` flagged as potentially stale in open threads.
- CI passes on the PR.

NARROWING:
- Do NOT perform adversarial-reward or governance work unless it directly blocks trace-semantics progress.
- Do NOT begin WDK#43 implementation before WDK#41 bookkeeping is cleanly closed (Phase 1 before Phase 2).
- Do NOT close bead `athena-fom` — flag it as potentially stale and defer to a future cleanup session.
- Do NOT introduce synthetic certainty in convergence summaries. If the source data is insufficient for a confidence determination, the derivation must reflect that uncertainty explicitly.
- Do NOT modify the VASP adapter's existing ConvergencePoint logic. It is the reference implementation; study it, don't change it.
- Do NOT edit prior FINDINGS.md investigation log entries. The log is append-only; only Accumulated Findings sections may be updated.
- Stay within trace-semantics scope. If governance or adversarial-reward issues surface, log them as open threads.
- Prototypes are research artifacts, not production code. Follow ATHENA prototype rules.
- Out of scope: new ADRs, changes to GOVERNANCE.md, changes to audit.sh or contract-gate.yml.
```

---

## Review Findings

### Issues Addressed
- **W1 (wrong file for ConvergencePoint):** Step 2 now reads `event_kinds.rs` for ConvergencePoint (with field names) and `lel.rs` for TraceEvent/EventLog types. ROLE updated to note `event_kinds.rs` location.
- **W2 (missing artifact path):** Full path `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md` added to Step 3a close reason, Step 3b resolution note, and END GOAL.
- **W3 (append-only rule risk):** Step 3c narrowed to "Accumulated Findings sections only" with explicit exemption: "Do not touch any investigation log entry."
- **W4 (athena-fom under-specified):** Step 3d now prescriptive: "Flag as stale, defer closure, do not investigate further even if evidence appears conclusive."

### Remaining Suggestions
- **S1 (per-adapter tie-breaker):** Added to Step 4d: "Default to per-adapter approach for consistency with VASP pattern. Only choose shared module if derivation logic is demonstrably identical."
- **S2 (minimum-data threshold):** Added to Steps 4b/4c: "Define minimum input data condition explicitly" with forward reference to test case (iii) in 4e.
- **S3 (session numbering):** ROLE now clarifies: "project Sessions 11–18" for governance work and "adversarial-reward internal Sessions 1–7" for WDK#41 itself.
- **S4 (no action needed):** Confirmed.

## Usage Notes

- **Best used with:** Claude Opus 4.6 in Claude Code CLI with beads workflow, git hooks active, and Rust toolchain (cargo, clippy) available.
- **Adjust for:** If WDK#43 yields a shared derivation module instead of per-adapter logic, update the Prototype Index to reflect the new file.
