# Prompt: Regime Validity + Guardrail Specification

> Generated: 2026-02-22 | Prompt #16 | Framework: RISEN

---

## Session Goal

Determine whether the hybrid aggregation candidate's known failure modes (Pattern B under-response, S2 sigmoid fragility at x0=-0.2, S1 SE multiplier fragility) fall within realistic DSL operating ranges or only in stress-test extremes, then codify the x0>=0 guardrail to unblock the Session 6 AggregateScore recommendation.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Multi-step analysis with clear phases (derive ranges, overlay failures, classify, codify guardrail), sequential dependencies, and explicit methodology. RISEN maps directly to this structure.
- **Alternatives considered:** TIDD-EC (good for the guardrail specification, but derivation + overlay phases dominate); Chain of Thought (this is implementation/analysis, not abstract reasoning)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Binary question per failure (in-range vs out-of-range) + concrete guardrail output |
| Specificity | 9/10 | Known failure boundaries, bead IDs, specific parameter constraint (x0>=0) |
| Context | 9/10 | Session 4/4.1/5 results referenced, failure modes enumerated with data |
| Completeness | 9/10 | Derivation, overlay, classification, guardrail, FINDINGS update all covered |
| Structure | 9/10 | Clean RISEN phasing with two bead closures as end gate |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

ROLE:
You are a scientific computing analyst working on the ATHENA adversarial reward research track. You have expertise in Bayesian hypothesis testing, DSL parameter space analysis, and scientific guardrail specification. You are familiar with the existing aggregation-candidates codebase, the Session 4/4.1/5 results, and ATHENA's DSL-only architectural constraint (ARCHITECTURE.md Section 1).

INSTRUCTIONS:
1. This session closes two beads: athena-17c (S4.2 regime validity analysis) and athena-zvg (S4.3 guardrail specification). Both block athena-6ax (Session 6 AggregateScore recommendation).
2. The central question is: "Are the remaining failures in realistic operating ranges, or only in stress-test extremes?" Answer this per failure mode before proposing any mitigation.
3. Derive realistic DSL parameter ranges from ATHENA's target DSL domains (OpenMM, GROMACS, CESM, VASP) — what signal sizes (z-scores), standard errors, Bayes factors, and custom sigmoid parameters actually occur in real scientific workflows?
4. These estimates are derived from your training knowledge of computational science workflows. Flag each estimate with a confidence indicator: HIGH (well-established in standard practice), MEDIUM (reasonable inference from domain knowledge), LOW (uncertain, needs domain expert validation). A regime validity classification of "in-range" requires at least MEDIUM confidence on the relevant parameter range.
5. Read `perturbation_summary.md`, `stretch_summary.md`, and source files before starting analysis.
6. The guardrail document is an architecture-level specification, not a code change. It locks a parameter constraint with rationale.

STEPS:
1. **Read existing results:**
   - Read `perturbation_summary.md` for S2 sigmoid tipping points (x0=-0.2 with k>=2.0 region), S1 SE multiplier failures (SE mult=5.0 and 10.0), and S5 BF failure boundary (BF>=120, now resolved by log-scaled normalization).
   - Read `stretch_summary.md` for Pattern B under-response (step_ratio=1.029, threshold >3.0).
   - Read FINDINGS.md Session 4, 4.1, and 5 entries for accumulated context.
   - Read `normalization.py` to identify which normalization kinds use custom sigmoid parameters, and `scenarios.py` to identify which scenario fixtures supply custom sigmoid configs.

2. **Derive realistic DSL parameter ranges (athena-17c):**
   - For each parameter axis tested in perturbation sweeps (custom sigmoid x0/k, BF magnitude, SE scale, missing data count), determine the realistic operating range by reasoning from ATHENA's target DSL domains.
   - Signal sizes: what z-scores do molecular dynamics / climate / materials science experiments typically produce? Confidence: flag each estimate.
   - Bayes factors: what BF magnitudes are typical in computational hypothesis testing? Confidence: flag each estimate.
   - Custom sigmoid x0: what does x0 represent in the normalization context, and what values are physically meaningful? Confidence: flag each estimate.
   - SE multipliers: what range of standard errors relative to baseline do real DSL environments produce? Confidence: flag each estimate.
   - Produce a table: `parameter | realistic_min | realistic_max | confidence | source/rationale`.

3. **Overlay failure boundaries on realistic ranges:**
   - For each known failure mode, classify as:
     * **In-range** (failure boundary falls within realistic operating range → must mitigate)
     * **Out-of-range** (failure boundary falls outside realistic operating range → document as acceptable boundary)
     * **Resolved** (failure was addressed by a prior fix → confirm with reference)
   - Produce a table: `failure_mode | boundary_value | realistic_range | classification | rationale`.
   - Key failures to classify:
     a. S2 sigmoid fragility: x0=-0.2 with k>=2.0 causes S2 failure. Is x0=-0.2 a realistic operating point?
     b. Pattern B under-response: hybrid step_ratio=1.029 on sudden single-metric 50x regime change. Do real DSL experiments produce sudden 50x single-metric jumps? For each target DSL domain, identify whether any known physical phenomenon produces sudden single-metric changes of this magnitude.
     c. S1 SE multiplier fragility: hybrid fails at SE mult=5.0 and 10.0. Do real DSL environments produce standard errors 5-10x the typical range?
     d. S5 BF ceiling (BF>=120): already resolved by log-scaled normalization (bf_max_target=10000, Session 4.1). Include in the classification table as "Resolved" with a reference to `ceiling_analysis.md`. Do not propose additional guardrails for this failure mode.
     e. S6 joint compression: resolved by the same log-scaled BF normalization that resolved S5 (Session 4.1). Include as "Resolved" with reference. Do not classify separately.

4. **Write regime validity analysis:**
   - Create `regime_validity.md` in the aggregation-candidates directory with: parameter range table (with confidence indicators), overlay classification table, per-failure narrative analysis, and verdict.
   - Create `regime_validity.json` with machine-readable version of same.

5. **Codify guardrail specification (athena-zvg):**
   - Create `guardrail_spec.md` in the aggregation-candidates directory.
   - Primary guardrail: `x0 >= 0` for all custom sigmoid normalization parameters.
   - Structure: constraint statement, rationale (citing Session 4 perturbation evidence that x0=-0.2 is the failure locus), scope (which components use custom sigmoids — sourced from normalization.py and scenarios.py), enforcement mechanism (specify where in the future production pipeline this would be enforced, e.g. config construction time, and the rejection behavior — rejection with error, not silent clamping — but note this is an architectural constraint, not a prototype-level implementation), and what happens if violated.
   - Include Pattern B and S1 SE assessments: if classified as out-of-range, document as "accepted limitation with boundary" rather than a guardrail. If classified as in-range, propose a mitigation path.

6. **Update FINDINGS.md:**
   - Append a "Session 4.2/4.3" investigation log entry (reverse-chronological, top of Investigation Log) matching the bead IDs, titled: `WDK#41 Session 4.2/4.3: Regime Validity Analysis + Guardrail Specification`.
   - Include: Scope, Method, Findings, Implications, Open Threads.
   - Add prototype index rows for regime_validity.md, regime_validity.json, guardrail_spec.md. Status: `Complete (Session 4.2/4.3)`.
   - Update Accumulated Findings (What We Know / What We Suspect / What We Don't Know) to reflect regime validity conclusions.

7. **Close beads and session protocol:**
   - `bd close athena-17c athena-zvg`
   - `bd sync`
   - Stage, commit, push.

END GOAL:
- `regime_validity.md` + `.json` answer "in-range or out-of-range?" for each known failure mode with evidence and confidence indicators
- `guardrail_spec.md` locks x0>=0 with rationale, scope, and enforcement specification
- athena-17c and athena-zvg closed, unblocking athena-6ax for the next session
- FINDINGS.md updated with Session 4.2/4.3 investigation log entry and accumulated findings refresh
- All committed and pushed

NARROWING:
- Do NOT modify any existing prototype scripts (stretch_test.py, perturbation_test.py, ceiling_analysis.py, etc.) — this session is analysis and specification, not code changes
- Do NOT fabricate precise parameter ranges — use training-knowledge-informed estimates with explicit confidence indicators (HIGH/MEDIUM/LOW), or mark as "needs domain expert input" if confidence is LOW
- Do NOT propose redesigning the hybrid to fix Pattern B — the question is whether Pattern B's failure scenario is realistic, not how to fix it
- Do NOT use "Session 6" as the FINDINGS.md log entry label — use "Session 4.2/4.3" to match bead IDs and avoid collision with athena-6ax
- Do NOT weaken the x0>=0 guardrail to a softer constraint (e.g., "x0 >= -0.1") — the perturbation evidence shows x0=-0.2 is the failure locus, and x0>=0 provides clean margin
- Do NOT skip reading perturbation_summary.md and stretch_summary.md — the failure boundaries must come from actual data, not memory
- Do NOT close athena-6ax — it remains open for the next session
- Stay within the aggregation-candidates prototype directory for output files

---

## Review Findings

### Issues Addressed
1. **[W1] Session numbering** — Changed to "Session 4.2/4.3" matching bead IDs, avoiding collision with athena-6ax
2. **[W2] S5 classification ambiguity** — Clarified as "Resolved" with reference to ceiling_analysis.md
3. **[W3] Missing S1 SE failure** — Added as classification target (fails at 5x/10x SE multiplier)
4. **[W4] Missing S6 joint compression** — Added as "Resolved" by same ceiling fix, explicit exclusion from separate classification
5. **[W5] Sigmoid scope pointer** — Added instruction to read normalization.py and scenarios.py
6. **[W6] Literature estimate methodology** — Added confidence indicator requirement (HIGH/MEDIUM/LOW) with minimum threshold for in-range classification

### Remaining Suggestions
- JSON schema for regime_validity.json could be more prescriptive
- Guardrail enforcement mechanism slightly over-specified for prototype context (mitigated by noting it's architectural, not implementation)
- Pre-commit verification step could catch incomplete outputs
- Pattern B contextualization per DSL domain (partially addressed in Step 3b)

## Usage Notes

- **Best used with:** Claude Code or Codex with file read/write + bash access to the ATHENA repo
- **Adjust for:** If additional failure modes are discovered before this session runs, add them to Step 3's classification list
