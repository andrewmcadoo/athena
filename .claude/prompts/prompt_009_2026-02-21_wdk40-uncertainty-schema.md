# Prompt: WDK#40 UncertaintySummary Schema Investigation

> Generated: 2026-02-21 | Prompt #1 | Framework: RISEN

---

## Session Goal

Investigate WDK#40 by surveying external UQ/V&V frameworks, tracing 6 consumers against 3 candidate schemas, assessing cross-adapter feasibility, steel-manning/stress-testing candidates, specifying the recommended `UncertaintySummary` schema, and writing the Step 14 FINDINGS.md entry — all following the ATHENA research methodology.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step research investigation with clear methodology (survey → consumer trace → feasibility → stress-test → specification → write-up). RISEN's Role/Instructions/Steps/End Goal/Narrowing maps directly to the investigation structure.
- **Alternatives considered:** TIDD-EC (good for dos/don'ts but less natural for sequential methodology), Chain of Thought (fits reasoning but lacks role/scope framing for research protocol compliance)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal unambiguous: investigate WDK#40, produce Step 14 entry with recommended schema |
| Specificity | 9/10 | Consumer list, candidate definitions, evaluation criteria, file paths all concrete |
| Context | 9/10 | Key tensions, current state (no uncertainty types), architectural constraints stated |
| Completeness | 9/10 | Covers what/why/how, output format specified via FINDINGS.md protocol |
| Structure | 9/10 | RISEN components well-separated, steps sequential and numbered, narrowing explicit |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a research scientist with dual expertise in (1) uncertainty quantification / V&V frameworks (ASME V&V 10/20, Bayesian posterior summaries, sensitivity analysis) and (2) scientific software schema design. You are operating within the ATHENA project — a falsification-driven AI co-scientist — investigating how to represent measurement uncertainty on comparison metrics. You are familiar with ATHENA's three non-negotiable architectural constraints (DSL-only environments, warm-started causal priors, bounded adversarial design) and its research methodology: steel-man then stress-test, every claim needs a mechanism and conditions, honest limitations are non-optional, distinguish proven from conjectured.

INSTRUCTIONS:
Investigate WDK#40: "What minimal UncertaintySummary schema should accompany each divergence metric so one comparison profile can support both V&V/effect-size reporting and Bayesian/active-learning reward calibration without adapter-specific branching."

By "without adapter-specific branching" we mean: consumers of UncertaintySummary should not need to know which adapter produced the data in order to interpret it. The schema must be adapter-agnostic at the type level.

Key tensions to navigate:
- Five literature domains (V&V, Bayesian experimental design, hypothesis testing, replication, active learning) impose conflicting uncertainty requirements
- The current prototype has NO uncertainty types — DivergenceMeasure stores bare f64 values; existing ConfidenceMeta covers data completeness, not measurement uncertainty
- The schema must serve both point-estimate consumers (LFI, Mode Controller) and distributional consumers (BSE, experiment designer) without adapter-specific branching
- Adapters (VASP, GROMACS, OpenMM) have varying ability to produce uncertainty metadata

Follow the ATHENA FINDINGS.md protocol: append-only investigation log (reverse chronological), living synthesis updates, cite evidence for all claims, tag findings as [PROVEN] or [CONJECTURE].

Note: WDK#40 refers specifically to What We Don't Know #40 (UncertaintySummary schema). Do not confuse with What We Know #40 (structural trace properties), which is a separate numbered item.

STEPS:
1. External Survey (Scoped): Survey how established frameworks represent uncertainty on comparison metrics — ASME V&V 10/20 (tolerance bands, CIs, model form uncertainty), ArviZ/PyMC (az.summary() output: mean, sd, hdi, ess, r_hat), SALib (sensitivity index uncertainty: S1_conf, ST_conf), UQpy (Distribution and inference result shapes). Document the common pattern. Note which candidate design (A/B/C as defined in Step 4) each external schema most resembles. Survey from domain expertise and training knowledge; cite canonical references (papers, documentation URLs) where known, and flag any claims that would benefit from live verification.

2. Consumer Trace (6 consumers x 3 candidates): For each of 6 consumers, determine exactly what UncertaintySummary fields it reads. Evaluate each candidate against each consumer: provides needed info (Y/N), requires branching (Y/N), information lost.

   Consumers:
   (a) LFI Stage 3 — comparison quality for R17 threshold decisions (std_error or CI + sample_size)
   (b) BSE post-experiment — distributional metadata for KL divergence (distribution params or quantiles)
   (c) BSE pre-experiment — does not consume post-experiment UncertaintySummary directly; instead assess whether UncertaintySummary's type structure should be compatible with BSE's predicted posterior output format so that predicted-vs-actual comparison (ARCH §5.4) is type-compatible
   (d) Adversarial Experiment Designer — calibration metadata for predicted vs actual gain (ARCH §5.4)
   (e) Mode Controller — point-level convergence/confidence metrics (ARCH §6.1-6.2)
   (f) ConvergenceSummary — residual-level uncertainty, pattern confidence (Step 13)

3. Cross-Adapter Feasibility: Determine what uncertainty metadata each adapter can actually produce — VASP (SCF residuals, force convergence — likely point estimates + error magnitude only), GROMACS (energy drift, block-averaged statistics — possible bootstrap CI, standard deviation), OpenMM (reporter-dependent — could be statistical if reporter computes block averages). Flag any candidate that requires data an adapter fundamentally cannot produce.

4. Steel-Man / Stress-Test (3 Candidates):
   - Candidate A (Flat Optional Struct): All fields optional, consumers read what they need. Steel-man: simplest, G5-compliant via None, near-precedent in ConfidenceMeta. Suggested attack vectors (non-exhaustive): combinatorial optionality, no type-level validity guarantees, BSE underserved.
   - Candidate B (Tagged Union by Uncertainty Source): Enum variants per uncertainty regime (PointEstimate / ParametricDistribution / EmpiricalDistribution / BoundedInterval / NoUncertainty). Steel-man: type-safe, G5 machine-checkable via NoUncertainty variant, each consumer matches on relevant variant. Suggested attack vectors (non-exhaustive): single-variant forces choice when multiple representations coexist, match-arm overhead, adapter must choose variant.
   - Candidate C (Layered: Point Summary + Optional Distribution Payload): Always-present PointSummary layer + optional DistributionalPayload. Steel-man: mirrors Step 12's structure, V&V consumers read point layer only, BSE checks distributional first with point fallback. Suggested attack vectors (non-exhaustive): inner optionality still combinatorial, evaluate whether distributional payload duplicates Candidate B's variant structure.
   - Evaluate on 7 criteria with priority ordering — Primary: consumer coverage, G5 compliance, cross-adapter feasibility. Secondary: minimality, type safety, extensibility. Tertiary: information preservation.

5. Schema Specification: Write the recommended UncertaintySummary type definition with:
   - Field-by-field justification (which consumer reads it, which adapter populates it)
   - Explicit interaction with DivergenceKind (avoid redundant encoding — e.g., BayesFactor already encodes posterior info)
   - Explicitly address whether the UncertaintySummary type is shared between MetricComponent.uncertainty and ConvergenceSummary.uncertainty, or whether these require different types
   - Connection to WDK#41-44 without resolving them
   - Assessment of whether prototype code changes are warranted

6. Write FINDINGS.md Step 14 Entry: Append to research/trace-semantics/FINDINGS.md in reverse chronological position with sections: Scope (WDK#40), Method (external survey, consumer trace, adapter feasibility, steel-man/stress-test, specification), Findings (numbered with evidence citations and [PROVEN]/[CONJECTURE] tags), Implications (recommended schema, ComparisonProfileV1 integration, adversarial-reward compatibility), Open Threads (remaining schema questions deferred). Update living synthesis: WDK#40 status, new What We Know items. When adding new What We Know items, use the next available sequential number after the current highest.

END GOAL:
A Step 14 FINDINGS.md entry that resolves or narrows WDK#40 with:
- A recommended UncertaintySummary schema that serves all 6 consumers without adapter-specific branching
- Evidence-backed findings distinguishing [PROVEN] from [CONJECTURE]
- Cross-adapter feasibility confirmed for VASP, GROMACS, OpenMM
- Clear connection to downstream dependencies (WDK#41-44, adversarial-reward track)
- Updated living synthesis reflecting WDK#40 resolution status

NARROWING:
- Do NOT resolve WDK#41 (aggregate reward scalar), WDK#42-44 (convergence details) — note connections only
- Do NOT make prototype code changes unless type-level ambiguity between candidates cannot be resolved analytically
- Do NOT write production code — prototypes are research artifacts
- Do NOT work on adversarial-reward FINDINGS.md — that is a downstream consumer, not in scope
- Stay within ATHENA research methodology: no unsupported assertions, no grant-proposal rhetoric
- Schema must respect G1-G5 guarantees from Step 12's ComparisonProfileV1 contract
- Out of scope: implementing adapters, runtime performance benchmarking

Key files to read before starting (all prototype paths are relative to research/trace-semantics/):
- research/trace-semantics/FINDINGS.md — read the full file, especially Steps 12 and 13
- research/trace-semantics/prototypes/lel-ir-prototype/src/common.rs — lines 134-150 (ComparisonOutcome, DivergenceMeasure) and 266-280 (ConfidenceMeta)
- research/trace-semantics/prototypes/lel-ir-prototype/src/overlay.rs — lines 230-277 (compare_predictions)
- ARCHITECTURE.md — sections §4.5, §5.4, §6.1-6.2
- research/adversarial-reward/FINDINGS.md — skim for downstream consumer expectations

Use beads workflow: bd create --title="Step 14: WDK#40 UncertaintySummary schema" before starting, bd update <issue-id> --status=in_progress when working, bd close <issue-id> when done, bd sync --from-main at session end.
```

---

## Review Findings

### Issues Addressed
1. **[WARNING] BSE pre-experiment consumer reframed** — Changed from treating it as a direct consumer to assessing type-compatibility with BSE's predicted posterior output format
2. **[WARNING] WDK#40 numbering disambiguation** — Added explicit note distinguishing WDK#40 from What We Know #40
3. **[WARNING] ConvergenceSummary type-sharing** — Added explicit design decision requirement in Step 5
4. **[WARNING] Beads workflow syntax** — Corrected bd commands with proper arguments and issue-id placeholders
5. **[WARNING] Living synthesis numbering** — Added instruction to use next sequential number

### Remaining Suggestions
- Survey scope could note reliance on domain expertise vs live docs (addressed in Step 1)
- 7 evaluation criteria could be weighted/prioritized (addressed with Primary/Secondary/Tertiary ordering)
- Pre-loaded attacks could bias analysis (reframed as "suggested attack vectors, non-exhaustive")
- File paths should clarify base directory (added note about relative paths)
- "Without adapter-specific branching" definition (added explicit definition in INSTRUCTIONS)

## Usage Notes

- **Best used with:** Claude Opus or equivalent model with strong scientific reasoning, in a session with access to the ATHENA codebase
- **Adjust for:** If web access is available, the external survey (Step 1) can be enriched with live documentation lookups
