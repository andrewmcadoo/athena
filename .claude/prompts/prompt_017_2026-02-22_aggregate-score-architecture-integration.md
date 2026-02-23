# Prompt: AggregateScore Architecture Integration (Session 7)

> Generated: 2026-02-22 | Prompt #17 | Framework: RISEN

---

## Session Goal

Produce a structured prompt for ATHENA Session 7 that integrates the locked AggregateScore contract into architecture — updating ARCHITECTURE.md, decomposing into implementation beads (BF normalization seam, x0>=0 guardrail, decomposition invariant), defining acceptance tests from Session 6 evidence, and wiring 5 revisit triggers into monitoring — with explicit non-goal language prohibiting any retuning of AggregateScore math or parameters.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step process with clear methodology (4 sequential deliverables), specific role expertise needed (architecture integration), and explicit narrowing constraints (no retuning). RISEN's Role/Instructions/Steps/End Goal/Narrowing structure maps directly onto these requirements.
- **Alternatives considered:** TIDD-EC (good for do/don't constraints but weaker on sequential workflow), CO-STAR (no audience/tone concern here)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal is unambiguous; locked-contract language is explicit throughout |
| Specificity | 9/10 | Parameters, artifact paths, section references, and acceptance criteria are quantitative |
| Context | 9/10 | Full traceability to Session 6 artifacts; ARCHITECTURE.md sections identified |
| Completeness | 9/10 | Covers all 4 deliverables + FINDINGS update + session close; monitoring definitions include 4 required fields |
| Structure | 9/10 | Clean RISEN mapping; steps are sequential with substeps; narrowing is specific and justified |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are an architecture integration engineer for the ATHENA project (falsification-driven AI co-scientist). You have deep familiarity with ATHENA's ARCHITECTURE.md component model, the adversarial-reward research track, and the locked AggregateScore recommendation from Session 6 (`aggregate_score_recommendation.md`). Your job is to translate a validated research specification into architecture-level contracts, implementation work items, acceptance tests, and operational monitoring — without reopening or modifying the research decisions.

INSTRUCTIONS:
- This is an integration session, not a research session. The AggregateScore algorithm, parameters, normalization family, and guardrails are LOCKED per `aggregate_score_recommendation.md` (version 1.0, 2026-02-22, bead athena-6ax). Treat that document as the source of truth.
- Follow CLAUDE.md governance: read before writing, append-only FINDINGS.md log, prototype code in `prototypes/` only, cite evidence for claims.
- Use beads (`bd create`, `bd update`, `bd close`) for all task tracking. No TodoWrite or markdown task files. Beads workflow commands are provided by the project's bead tracking tool; usage follows the pattern established in Sessions 1-6 (see `.claude/handoffs/` for prior examples).
- Every deliverable must trace back to a specific section or parameter in the Session 6 recommendation. No invented requirements.
- When updating ARCHITECTURE.md, preserve existing structure and style. Add; do not reorganize.

STEPS:

1. Read the locked recommendation and all supporting evidence.
   - Read `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md` and `.json` in full.
   - Read ARCHITECTURE.md Sections 4.4, 5.4, and 8.1 for current adversarial reward references.
   - Read `regime_validity.md` and `guardrail_spec.md` for operating boundary and guardrail detail.
   - Read FINDINGS.md accumulated findings for Session 6 context.

2. Update ARCHITECTURE.md with the AggregateScore contract.
   - Add a new subsection defining the AggregateScore contract (e.g., 4.4.1 or appended to the Adversarial Experiment Designer description in Section 4.4), depending on which preserves existing document flow. Cross-reference from Sections 5.4 and 8.1.
   - Include the hybrid pipeline definition (recommendation Section 1.1), locked parameters table (Section 1.3), and BF normalization spec (Section 1.2).
   - Reference guardrail GR-S2-CUSTOM-SIGMOID-X0-NONNEG (Section 1.4).
   - Reference the output contract (Section 1.5) including the decomposition invariant and the note that `n_terms=1` is intentional (recommendation Section 6 note 4), not a placeholder.
   - Update the risk entry in Section 8.1: downgrade status from "open research problem" to "specified, pending implementation" and adjust severity accordingly (e.g., High → Medium with annotation). Ensure any Appendix priority references stay consistent.
   - This change introduces a locked contract that downstream components must build against. Create `decisions/002-aggregate-score-contract.md` as an ADR documenting the decision, evidence basis, and locked parameters.

3. Create implementation beads from the spec. Decompose into at least these three beads:
   - BF normalization seam: Expose `bf_norm_log_scaled(bf, c)` as a first-class configurable hook in the normalization path, not a hard-coded branch. Default to `c=0.083647`. Reference: recommendation Section 1.2 and Section 6 note 1.
   - x0 >= 0 config-time guardrail: Implement `GR-S2-CUSTOM-SIGMOID-X0-NONNEG` as schema-level validation at `NormalizationConfig` construction. Reject with explicit error (guardrail ID, offending method_ref, offending x0, expected constraint). No silent clamping. Reference: recommendation Section 1.4, `guardrail_spec.md`.
   - Decomposition invariant check: Implement runtime assertion or validation that `sum(contribution_i) == aggregate_score` within tolerance `1e-8` (per S6 gate from Session 3). Reference: recommendation Section 1.5 and Section 6 note 3.
   - Set up dependency edges between beads where appropriate (e.g., normalization seam may need to exist before acceptance tests can run).

4. Define acceptance tests directly from Session 6 evidence. Create a test specification (as a bead or document) covering:
   - Baseline scenario gates: All 7 scenarios must pass with margins matching `aggregate_score_recommendation.json` field `baseline_margins` within tolerance `1e-6` (per Session 5 `stretch_test.py` Phase 0 precedent).
   - Resolved-risk regressions: S5 must pass through BF=1000 (evidence: `ceiling_analysis.md` Section 3). S6 must pass at all 5 previously-failing (d_mid, bf) cells (evidence: `ceiling_analysis.md` Section 4). S2 must reject configs with x0 < 0 (evidence: `guardrail_spec.md`).
   - Accepted-boundary behavior: Pattern B step_ratio < 3.0 is documented expected behavior, not a failure (evidence: `regime_validity.md`, `stretch_summary.md`). S1 at SE_mult >= 5.0 is documented expected behavior, not a failure (evidence: `regime_validity.md`). These are assertion-of-limitation tests, not pass/fail gates.
   - Reference specific evidence artifacts for each test case as shown above.

5. Wire revisit triggers into monitoring. Since ATHENA has no production monitoring infrastructure yet, produce a monitoring specification document (e.g., `monitoring_triggers.md` in the aggregation-candidates directory or a project-level location) that defines each trigger as a future-implementation contract. For each of the 5 triggers in `aggregate_score_recommendation.md` Section 4 (T1-T5), define:
   - Source signal: What data or event would indicate the trigger condition? (e.g., T1: distribution statistics from production DSL trace ingestion pipeline)
   - Threshold: What quantitative or qualitative boundary trips the trigger?
   - Owner: Who or what process is responsible for checking?
   - Action path: What happens when triggered? (e.g., create a bead, notify research lead, re-run regime validity analysis)

6. Update FINDINGS.md. Write a Session 7 investigation log entry documenting scope, method, findings, implications, and open threads. Update accumulated findings to reflect architecture integration status.

7. Session close protocol. `git status` → `git add` → `bd sync` → `git commit` → `bd sync` → `git push`.

END GOAL:
Session 7 is complete when:
- ARCHITECTURE.md contains the locked AggregateScore contract with all parameters, normalization spec, guardrail reference, decomposition invariant, n_terms=1 rationale, and updated risk severity.
- ADR `decisions/002-aggregate-score-contract.md` exists documenting the locked decision.
- At least 3 implementation beads exist (BF seam, guardrail, decomposition invariant) with clear descriptions, acceptance criteria, and dependency edges.
- A test specification exists covering baseline gates (7 scenarios, tolerance 1e-6), resolved-risk regressions (S5/S6/S2), and accepted-boundary assertions (Pattern B, S1 extreme SE).
- A monitoring specification exists with all 5 revisit triggers defined (source signal, threshold, owner, action path).
- FINDINGS.md has a Session 7 log entry.
- All changes are committed and pushed.

NARROWING:
- Do NOT retune, modify, or question the AggregateScore algorithm, parameters, or normalization constants. They are locked. If you find an issue during integration, document it as a potential revisit trigger — do not fix it in-session.
- Do NOT re-run any aggregation experiments, sweeps, or robustness analyses. Session 7 produces architecture documents and work items, not prototype code or experimental results.
- Do NOT change the operating envelope boundaries in `regime_validity.md` or `aggregate_score_recommendation.md`. Those are Session 4.2/6 outputs.
- Do NOT create production implementation code. Implementation beads describe what to build; the building happens in subsequent sessions.
- Do NOT weaken or relax the guardrail (GR-S2-CUSTOM-SIGMOID-X0-NONNEG). It is reject-only, no silent clamping, as specified.
- Out of scope: Pattern B step-ratio improvement, new scenario development, correlation probe extensions, empirical DSL trace validation. These are documented future work, not Session 7 deliverables.
- Stay within: CLAUDE.md governance, beads workflow, existing ARCHITECTURE.md structure and style conventions.
```

---

## Review Findings

### Issues Addressed
1. **ARCHITECTURE.md insertion point** (Warning) — Added concrete placement guidance: "new subsection (e.g., 4.4.1 or appended to Section 4.4), cross-reference from 5.4 and 8.1"
2. **Acceptance test tolerance** (Warning) — Locked to `1e-6` for baseline margins (Session 5 precedent) and `1e-8` for decomposition invariant (S6 gate)
3. **ADR decision criterion** (Warning) — Resolved by directing ADR creation: "Create `decisions/002-aggregate-score-contract.md`"
4. **Beads workflow assumption** (Warning) — Added reference note to INSTRUCTIONS
5. **Monitoring infrastructure assumption** (Warning) — Specified document-form output with acknowledgment of no production infra

### Remaining Suggestions
- Risk severity level could be more precisely specified (e.g., "High → Medium" vs leaving as annotated downgrade). Current phrasing gives reasonable guidance.
- Step 1 read list expanded to include `regime_validity.md` and `guardrail_spec.md` per reviewer suggestion.
- `n_terms=1` preservation note added to Steps 2 and END GOAL per reviewer suggestion.
- `git push` in Step 7 is intentional per established session close protocol in this project.

## Usage Notes

- **Best used with:** Claude Opus 4.6 with full project context (CLAUDE.md, ARCHITECTURE.md, Session 6 artifacts)
- **Adjust for:** If ATHENA gains production monitoring infrastructure before Session 7, Step 5 can target that infrastructure directly instead of producing a spec document
