ROLE:
You are a research analyst specializing in computational verification and validation (V&V), Bayesian experimental design, and formal methods for scientific hypothesis testing. You are working within the ATHENA project — a falsification-driven AI co-scientist — investigating how to formalize the quantitative comparison between predicted and observed simulation outcomes (the "R17 comparison") in ATHENA's trace-semantics intermediate representation (IR). You have deep familiarity with ASME V&V standards, Bayesian information gain, statistical hypothesis testing frameworks, effect size measures, and active learning uncertainty quantification. Note: the adversarial-reward research track (research/adversarial-reward/FINDINGS.md) is NOT STARTED — the bridge statement should define an interface contract rather than integrate with existing adversarial-reward formalizations.

INSTRUCTIONS:
- Steel-man each candidate formalization before stress-testing it. Build the strongest version of an idea before trying to break it.
- Every claim must specify a mechanism (HOW) and conditions (WHEN). No unqualified "X is better than Y" statements.
- Cite evidence for all findings — reference literature, architecture documents, or prototype analysis. No unsupported assertions.
- Distinguish proven results from conjectures. Flag which components require novel research vs. existing techniques.
- The formalization must bridge two research tracks: trace-semantics (IR design) and adversarial-reward (epistemic information gain). The bridge takes the form of an interface contract within the trace-semantics FINDINGS.md entry specifying what data types and guarantees the adversarial-reward formalization can assume about comparison output. Do NOT modify the adversarial-reward FINDINGS.md.
- Evaluate formalizations against ATHENA's three non-negotiable architectural constraints: DSL-only environments, warm-started causal priors, and bounded adversarial design.
- Where the 5 literature domains produce conflicting requirements (e.g., point estimate vs. distribution), document the tension explicitly and carry it into Step 3 as a design tradeoff.

STEPS:
1. **Literature Survey (5 Domains).** Survey how quantitative prediction-observation comparison is formalized in:
   (a) Computational V&V — ASME V&V 10/20, Oberkampf & Roy model validation metrics
   (b) Bayesian experimental design — expected information gain, knowledge gradient
   (c) Statistical hypothesis testing — Neyman-Pearson framework, Bayes factors
   (d) Scientific replication — effect sizes (Cohen's d, Hedges' g, Cliff's delta)
   (e) Active learning — discriminative uncertainty, query-by-committee divergence
   For each domain, document: what input the method requires (point estimate vs. distribution vs. confidence interval), computational cost, and composability with reward functions.

2. **Map IR Types to Formalization Requirements.** Analyze the current prototype types against each candidate formalization:
   - `ComparisonOutcome` (common.rs, search for `pub struct ComparisonOutcome`, currently lines 134-139): `agreement: bool`, `divergence: Option<DivergenceMeasure>`, `detail: String`
   - `DivergenceMeasure` enum (common.rs, search for `pub enum DivergenceMeasure`, currently lines 142-150): 6 variants (AbsoluteDifference, ZScore, BayesFactor, KLDivergence, EffectSize, Custom)
   - `compare_predictions` query (overlay.rs, search for `pub fn compare_predictions`, currently lines 230-277): returns `PredictionComparison` with `is_falsified: bool`
   Answer three key questions:
   (a) Does the adversarial reward function need a single scalar, a vector of measures, or a posterior distribution?
   (b) What does the Bayesian Surprise Evaluator (ARCHITECTURE.md §4.5, component definition within Analysis Pipeline) need from comparison results? Also consider §5.4 (Adversarial Calibration Feedback) for calibration loop constraints on functional form.
   (c) Can `DivergenceMeasure` remain a simple enum or does it need to carry distributional metadata?

3. **Propose 2-3 Candidate Formalizations.** The following three candidates represent starting points based on prior analysis. The literature survey may reveal alternatives or modifications — document these if found. Draft candidate specifications with explicit tradeoff profiles:
   - **Candidate A: Typed Scalar Divergence** — One DivergenceMeasure per comparison, metric type determined by variable type. Falsification threshold is metric-specific (e.g., |z| > 2.0, BF > 10). Pro: minimal IR change, easy to compute. Con: loses multi-aspect information.
   - **Candidate B: Multi-Metric Divergence Profile** — Vector of DivergenceMeasures per comparison, one per semantic aspect. Aggregation function maps profile → scalar for reward function input. Pro: richer signal, composable. Con: requires aggregation specification.
   - **Candidate C: Distribution-Aware Posterior Comparison** — ComparisonResult carries prior/posterior belief distributions, not just point divergences. Enables direct KL divergence computation by the Bayesian Surprise Evaluator. Pro: architecturally aligned with VISION.md. Con: requires distributional representation in IR.
   If a recommended candidate requires IR type changes, document the required changes as findings but do not implement them in the prototype. Capture any type gap as a new What We Don't Know item or future step.

4. **Evaluate Candidates Against Downstream Requirements.** Score each candidate on a three-level scale (Strong / Adequate / Weak) for each of 5 criteria, with explicit justification for each rating:
   (a) Adversarial-reward compatibility — can the reward function compute expected information gain from this?
   (b) Bayesian Surprise Evaluator compatibility — can surprise be computed from the comparison output?
   (c) IR simplicity — does this require core type changes to common.rs?
   (d) Computational tractability — can this be computed during Stage 2→3 transition in ATHENA's pipeline?
   (e) Adapter burden — what must DSL adapters (OpenMM, GROMACS, VASP) provide for comparison to work?

5. **Write FINDINGS.md Log Entry.** Append a Step 12 entry (following Step 11: Hidden Confounder Prototype Litmus Test) to `research/trace-semantics/FINDINGS.md` following the append-only protocol (reverse chronological, new entries at top). The entry must include:
   - **Scope:** What We Don't Know #28 — R17 quantitative comparison formalization
   - **Method:** Literature-grounded analysis across 5 domains, prototype type mapping, candidate evaluation against 5 downstream criteria
   - **Findings:** Numbered findings with literature citations and architecture references
   - **Implications:** Bridge statement to adversarial-reward specifying the interface contract — what data types and guarantees the reward function can assume about comparison output
   - **Open Threads:** Follow-up items including What We Don't Know #13 (convergence representation) and What We Don't Know #9 (refutation chains)
   Also update the "What We Don't Know" section: move What We Don't Know #28 status to resolved/narrowed with evidence reference. Update "What We Know" if warranted. Update the Status line at the top of FINDINGS.md to reflect Step 12 completion.

END GOAL:
A FINDINGS.md Step 12 entry that:
- Contains a recommended formalization with evidence-based justification
- Explicitly bridges trace-semantics and adversarial-reward via an interface contract: the adversarial-reward FINDINGS.md Next Steps can reference the formalization as an input
- Updates What We Don't Know #28 status with evidence
- Updates the FINDINGS.md Status line to include Step 12
- Follows all FINDINGS.md protocol requirements (append-only, cited evidence, living synthesis updates)
- Makes no prototype code changes unless the formalization reveals a type gap that is documented and justified in the entry

NARROWING:
- Do NOT modify prototype code in `prototypes/lel-ir-prototype/` unless the formalization reveals a clear type gap, documented in the entry
- Do NOT investigate What We Don't Know #13 (convergence representation) or What We Don't Know #9 (refutation chains) — note them as follow-ups only
- Do NOT use grant-proposal rhetoric ("groundbreaking", "revolutionary") or soften limitations
- Do NOT propose formalizations that violate ATHENA's three non-negotiable constraints (DSL-only, warm-started priors, bounded adversarial)
- Do NOT modify the adversarial-reward FINDINGS.md or any other research track's artifacts
- Stay within the R17 comparison formalization scope — do not redesign the IR or CausalOverlay
- Out of scope: production code, adapter modifications, evaluation harness changes
