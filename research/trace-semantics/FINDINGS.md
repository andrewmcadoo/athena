# Trace Semantics Engine: IR Design

## Research Question

What intermediate representation (IR) can translate raw DSL trace logs from structured simulation frameworks (OpenMM, GROMACS, VASP) into semantic failure representations suitable for three-way causal fault classification? The IR must preserve enough structure for the Lakatosian Fault Isolator to deterministically distinguish implementation-layer failures from methodology-layer failures from theory-layer contradictions. Success criteria: an IR specification that, given a trace log containing a known planted fault, enables correct fault classification at a rate significantly exceeding the 21% Top@1 baseline reported for general unstructured traces. This investigation blocks LFI effectiveness and is therefore the highest-priority research dependency.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.5 (Trace Semantics Engine) | Component definition, inputs/outputs, role in analysis pipeline |
| ARCHITECTURE.md | 5.3 (Fault Isolation Decision Tree) | Three-stage audit the IR must support: implementation, methodology, theory |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: High. IR design is unsolved. DSL constraint improves tractability. |
| VISION.md | Open Question #1 | "Semantic Language of Failure" — building the IR is a critical research problem |
| VISION.md | Section 4.1 (LFI) | LFI requires trace logs parseable into causal narratives |
| Constraint | DSL-Only Environments | IR design is bounded to structured DSL output, not arbitrary Python |

## Status

IN PROGRESS — Step 3 (IR requirements derivation) complete. Steps 1, 2, 4 not yet started.

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### 2026-02-20: LFI Audit → IR Requirements Mapping

**Scope:** Backward derivation of minimum IR semantic distinctions from ARCHITECTURE.md three-stage audit (§5.3). For each audit stage, enumerate every deterministic question the LFI must answer, then derive what IR content enables that answer. Also derive cross-cutting requirements, ambiguity handling requirements, and hidden confounder litmus test requirements.

**Method:** Requirements analysis. Source material: ARCHITECTURE.md §4.5 (Trace Semantics Engine), §5.1-5.4 (Information Flow, including Fault Isolation Decision Tree), §8.1 (Per-Component Risks), §8.4 (Incomplete Observability), §8.5 (Classification Staleness); VISION.md §4.1 (LFI), §6 (Honest Limitations), Open Question #1; evaluation/hidden-confounder/README.md (litmus test specification). For each of the three audit stages, I extracted every question the LFI must deterministically answer from the ARCHITECTURE.md text, then worked backwards to the minimum IR element that enables answering that question. Requirements are numbered R1-R25 for cross-referencing in Step 3b (coverage matrix) and Step 5 (IR schema evaluation).

**Findings:**

#### Stage 1: Implementation Audit — IR Must Support

The LFI's Stage 1 asks four explicit questions (ARCHITECTURE.md §5.3, Stage 1 paragraph). Each maps to one or more IR requirements.

**Q1.1: "Did execution complete without framework-level exceptions?"**
The IR must represent whether the DSL framework's execution reached normal termination or terminated abnormally, and if abnormally, what exception or error the framework raised.

- **R1. Execution completion status.** A per-execution record indicating: (a) whether the simulation run completed normally, (b) if not, the framework-reported termination reason. Data: enum {completed, exception, timeout, killed} plus framework error identifier. Source: DSL framework exit status and error logs. Example (OpenMM): a `NaNException` raised by `VerletIntegrator.step()` indicating numerical divergence; example (GROMACS): `Fatal error: step N` indicating constraint failure.

- **R2. Exception/error event.** When execution terminates abnormally, a structured record of the exception: type/code, the framework component that raised it, and the call location within the DSL API (not arbitrary Python stack, but the DSL-layer call path). Data: exception type identifier, DSL component identifier, DSL-layer call location. Example (VASP): `ZBRENT: fatal error in bracketing` from the electronic minimizer; example (OpenMM): `OpenMMException` from `Context.setPositions()` indicating invalid atom coordinates.

**Q1.2: "Do input data pipelines match the specification?"**
The IR must represent the experiment specification's expected inputs and the actual inputs observed during execution, in enough detail for the LFI to compare them.

- **R3. Input specification record.** The experiment specification's declared inputs: parameter names, expected values or ranges, data sources, and formats. Data: list of (parameter_name, expected_value_or_range, source_identifier). This is derived from the experiment specification, not the trace log, but must be represented in the IR for comparison. Example: an OpenMM experiment specifying `temperature=300*kelvin`, `topology=1ubq.pdb`, `forcefield=amber14-all.xml`.

- **R4. Actual input observation.** For each declared input, the value actually used during execution, as recorded in the trace log. Data: list of (parameter_name, actual_value, source_event_reference). Example: GROMACS `.mdp` file values as logged at simulation startup; VASP `INCAR` parameter echo at job start.

- **R5. Input validation result.** A derived comparison: for each input, whether the actual value matches the specification, and if not, the nature of the mismatch. Data: list of (parameter_name, match_status: {exact, within_range, mismatch, missing}, deviation_detail). This is a computed IR element, not directly extracted from the trace.

**Q1.3: "Are numerical operations within precision bounds?"**
The IR must represent the numerical health of the simulation during execution.

- **R6. Numerical status record.** Records of numerical conditions encountered during execution: NaN values, infinities, overflow/underflow events, precision mode (single/double), convergence failures in iterative solvers, and energy conservation violations. Data: list of (event_type: {nan, infinity, overflow, underflow, convergence_failure, conservation_violation}, location_in_DSL_pipeline, timestamp_or_step, severity, affected_quantity). Example (OpenMM): energy values becoming NaN at step 5000; example (VASP): electronic self-consistency loop failing to converge after maximum iterations; example (GROMACS): LINCS warning about constraint deviations.

**Q1.4: "Does the hardware/resource state match expectations?"**
The IR must represent the execution environment's state.

- **R7. Resource/environment status.** Records of the computational platform and resource state: device type (CPU/GPU), memory allocation and usage, parallelization configuration, and any resource-related warnings or failures. Data: (platform_type, device_identifiers, memory_allocated, memory_peak, parallelization_config, resource_warnings[]). Example (OpenMM): CUDA device selection, GPU memory exhaustion; example (GROMACS): MPI rank failure, thread count mismatch.

**Stage 1 summary.** Requirements R1-R7 are necessary and sufficient for the LFI to answer all four Stage 1 questions. All seven are implementation-layer concerns and must be tagged as such (see R19). All are directly extractable from DSL trace logs because DSL frameworks structurally separate these operations from theory-layer specifications (ARCHITECTURE.md §3.1).

#### Stage 2: Methodological Audit — IR Must Support

The LFI's Stage 2 asks four explicit questions (ARCHITECTURE.md §5.3, Stage 2 paragraph). Stage 2 is reached only if Stage 1 finds no faults. Stage 2 requires comparing the experiment specification against the hypothesis's causal claims, using the current DAG as context.

**Q2.1: "Does the experiment measure the variables the hypothesis links causally?"**
The IR must represent what was actually measured/observed during the experiment, with enough specificity to compare against the hypothesis's causal claims.

- **R8. Observable measurement record.** For each quantity measured during the experiment: the variable name (as defined in the DAG), the measurement method or observable type, the raw values or summary statistics, and the measurement conditions (e.g., at what simulation time, under what state). Data: list of (variable_name, measurement_method, values_or_summary, measurement_conditions, units). Example (OpenMM): radial distribution function g(r) computed from trajectory frames 500-1000; example (VASP): total energy per atom after ionic relaxation.

- **R9. Observable-to-DAG linkage.** For each measured observable, a mapping to the DAG node(s) it corresponds to, enabling the LFI to verify that the experiment measured the variables the hypothesis claims are causally linked. Data: list of (observable_id, DAG_node_id, relationship_type: {direct_measurement, proxy, derived}). This is a cross-referencing requirement: the IR must support joining observables to the causal graph. Source: ARCHITECTURE.md §5.3 ("comparing the experiment specification against the hypothesis's causal claims").

**Q2.2: "Is the intervention on the hypothesized cause or a correlated proxy?"**
The IR must represent what was intervened upon (the independent variable manipulation) and how.

- **R10. Intervention specification.** A record of the experimental intervention: which parameter(s) were varied, over what range, what control conditions were maintained, and whether the intervention targets the hypothesized causal variable directly or through an intermediary. Data: (intervened_parameter_name, intervention_range, control_conditions[], DAG_node_id_of_target, directness: {direct, proxy}). Example: varying `temperature` from 280K to 350K in OpenMM while holding `pressure` constant, targeting the DAG node for thermal kinetic energy.

- **R11. Intervention-to-DAG linkage.** A mapping from the intervention to the DAG edge(s) the hypothesis claims are causal. The LFI must verify the intervention targets the upstream node of the hypothesized causal edge, not a correlated but causally distinct variable. Data: (intervention_id, hypothesized_edge: {cause_node, effect_node}, intervention_targets: {cause_directly, proxy_via_node_X}). Source: ARCHITECTURE.md §5.3 ("Is the intervention on the hypothesized cause or a correlated proxy?").

**Q2.3: "Is the sampling sufficient to distinguish the effect from noise?"**
The IR must represent sampling adequacy.

- **R12. Sampling metadata.** Records of the experiment's sampling characteristics: sample count (e.g., number of trajectory frames, number of independent runs), sampling distribution, equilibration period, autocorrelation time, and any power analysis or uncertainty quantification performed. Data: (sample_count, sampling_method, equilibration_steps, autocorrelation_time_if_computed, statistical_power_if_computed, confidence_level). Example (GROMACS): 10 ns production run with 2 ns equilibration, 1000 frames sampled every 10 ps; example (VASP): 5 independent relaxations from perturbed starting geometries.

**Q2.4: "Are there known confounders (from the current DAG) that the experiment did not control for?"**
The IR must represent which variables were held constant (controlled) during the experiment and enable comparison against the DAG's confounder structure.

- **R13. Controlled variable set.** An explicit list of variables that the experiment held constant or controlled for, and the mechanism of control. Data: list of (variable_name, control_value_or_method, DAG_node_id). Example: pressure held at 1 atm via barostat in OpenMM; exchange-correlation functional held as PBE in VASP.

- **R14. DAG confounder query support.** The IR must be structured so the LFI can query: "Given the intervention in R10 and the observable in R8, which DAG nodes are potential confounders (common causes of both), and are they in the controlled set R13?" This is not a stored IR element but a queryability requirement: the IR must support efficient joins between intervention nodes, observable nodes, controlled variable sets, and DAG structure. Source: ARCHITECTURE.md §5.3 ("known confounders from the current DAG that the experiment did not control for") and §8.5 ("the confounder check depends on the DAG's accuracy").

**Stage 2 caveat.** ARCHITECTURE.md §5.3 explicitly warns: "the confounder check depends on the DAG's accuracy. If the DAG is missing a real confounder or contains a spurious one, this audit will either miss real confounders or flag phantom ones." The IR cannot fix this. But the IR must make the DAG dependency transparent -- every confounder judgment must be traceable to the specific DAG edges consulted (see R14). This traceability enables reclassification when the DAG changes (ARCHITECTURE.md §8.5).

#### Stage 3: Theoretical Evaluation — IR Must Support

Stage 3 is reached only if Stages 1 and 2 pass. The LFI compares results against the hypothesis's predictions (ARCHITECTURE.md §5.3, Stage 3 paragraph).

**Q3.1: "Does the evidence contradict the hypothesis's predictions?"**
This requires three sub-elements: what the hypothesis predicted, what was observed, and a formal comparison.

- **R15. Prediction record.** The hypothesis's quantitative predictions, stated before the experiment was run: which observable, what predicted value or distribution, what predicted direction of effect, and what tolerance or confidence interval constitutes "agreement." Data: (hypothesis_id, predicted_observable: variable_name, predicted_value_or_distribution, predicted_direction: {increase, decrease, no_change, specific_relationship}, tolerance_or_CI, DAG_edges_supporting_prediction[]). Source: ARCHITECTURE.md §5.1 ("candidate hypotheses with explicit causal claims and testable predictions") and §5.3 ("compares results against the hypothesis's predictions").

- **R16. Observation record.** The actual experimental result for the predicted observable, as extracted from the trace and processed by the IR. Data: (observable_id matching R8, actual_value_or_distribution, uncertainty_estimate, measurement_conditions). This overlaps with R8 but is specifically the subset of observables relevant to the hypothesis's predictions.

- **R17. Comparison result.** A formal quantitative comparison between prediction (R15) and observation (R16): effect size, statistical divergence measure (e.g., KL divergence, z-score, Bayes factor), confidence interval overlap, and a determination of whether the observation falls within or outside the prediction's tolerance. Data: (prediction_id, observation_id, effect_size, divergence_measure, divergence_value, within_tolerance: bool, comparison_method). Source: ARCHITECTURE.md §5.3 ("If the evidence contradicts the hypothesis") -- "contradicts" must be formalized as a quantitative comparison.

**Q3.2: "Which causal edges does the contradiction implicate?"**
When Stage 3 determines theoretical falsification, the LFI must produce "a graph update directive specifying which edges to prune or reweight" (ARCHITECTURE.md §5.3).

- **R18. Causal implication mapping.** For a theoretical falsification, a mapping from the contradicted prediction to the specific DAG edges that supported that prediction, enabling the LFI to produce a targeted graph update directive rather than a blanket penalty. Data: (falsified_prediction_id, implicated_DAG_edges[], proposed_update_per_edge: {prune, reweight, annotate}). Source: ARCHITECTURE.md §5.3 ("the graph update directive specifies which edges to prune or reweight") and §5.1 ("a directed update specifying which edges to prune, reweight, or annotate as falsified").

#### Cross-Cutting Requirements

These requirements apply across all three stages and are necessary for the LFI to function as specified.

- **R19. Layer tag.** Every IR element must be tagged as either `implementation-layer` or `theory-layer`. This is the fundamental structural distinction that makes the three-stage audit possible. The DSL's API separation provides this distinction (ARCHITECTURE.md §3.1: "the theoretical specification and the computational implementation are separated by the framework's API"), but the IR must preserve it. Without layer tags, the LFI cannot determine which stage an element belongs to. Source: ARCHITECTURE.md §3.1 and §4.5 ("maps theory-layer operations to implementation-layer events").

- **R20. Provenance chain.** Every IR element must be traceable to its source in the raw trace log: which log line(s), which file, which timestamp in the raw output produced this IR element. This is required for (a) the LFI to verify its reasoning against raw evidence, (b) enhanced logging re-runs when classification is ambiguous (ARCHITECTURE.md §5.3, Ambiguity handling), and (c) human escalation, where the raw evidence must be presentable (ARCHITECTURE.md §6.3). Data: each IR element carries (source_file, source_line_range, raw_text_hash). Source: ARCHITECTURE.md §4.5 ("receives raw trace logs... produces structured semantic failure/success representation"), §5.3 (ambiguity handling: "re-run with enhanced logging"), §6.3 (escalation: "provides the raw evidence").

- **R21. Temporal ordering.** IR elements must preserve causal sequence: the order in which events occurred during execution. The outside-in audit structure (Stage 1 before Stage 2 before Stage 3) requires knowing what happened in what order -- an exception at step 5000 preceded by a NaN at step 4999 tells a different causal story than the reverse. Data: every IR event carries a temporal coordinate (simulation step, wall-clock timestamp, or logical sequence number) enabling total ordering. Source: ARCHITECTURE.md §5.3 (sequential audit requires temporal reasoning about execution events).

- **R22. Experiment specification linkage.** The IR must include or reference the full experiment specification that produced the trace, so the LFI can compare intended vs. actual execution. The LFI receives "the experiment specification" as a separate input (ARCHITECTURE.md §4.5), but the IR must be joinable to it -- every IR element about inputs (R3, R4, R5), interventions (R10), and controls (R13) must reference the corresponding specification element. Source: ARCHITECTURE.md §4.5 (LFI "receives the structured IR from the Trace Semantics Engine... the experiment specification, and the hypothesis under test").

- **R23. Hypothesis linkage.** The IR must be joinable to the hypothesis under test, so Stage 2 can compare methodological adequacy against causal claims and Stage 3 can compare observations against predictions. The hypothesis itself is a separate LFI input, but the IR's observable records (R8), intervention records (R10), and prediction records (R15) must reference hypothesis elements. Source: ARCHITECTURE.md §4.5 (LFI receives "the hypothesis under test") and §5.3 (Stage 2: "comparing the experiment specification against the hypothesis's causal claims").

- **R24. Queryability.** The IR must support efficient lookup by: (a) layer tag (implementation vs. theory), (b) event type (execution, exception, numerical, resource, observable, intervention, etc.), (c) temporal range (events within step N to M), (d) variable name (all records pertaining to a specific variable), (e) DAG node (all records linked to a specific DAG node), (f) stage relevance (which IR elements are relevant to Stage 1 vs. 2 vs. 3). This is a structural requirement on the IR's organization, not on its content. The three-stage audit is sequential; each stage must be able to efficiently extract the subset of IR elements it needs without scanning the entire representation. Source: ARCHITECTURE.md §5.3 (sequential audit structure implies stage-specific queries) and §4.5 ("structured semantic intermediate representation suitable for causal fault analysis").

#### Ambiguity Handling Requirements

ARCHITECTURE.md §5.3 (Ambiguity handling) specifies: "When the LFI cannot confidently assign a failure to a single category, this is an escalation condition." The IR must support this.

- **R25. Classification confidence metadata.** For each IR element that contributes to a stage's determination, the IR must carry information about the element's completeness and reliability. Specifically: (a) whether the element was fully observed or partially inferred, (b) whether the raw trace contained sufficient information to populate all fields, (c) any gaps or uncertainties in the extraction. This enables the LFI to compute a classification confidence and trigger escalation when confidence is low. Source: ARCHITECTURE.md §5.3 (ambiguity handling), §8.4 ("unrecorded state changes introduce invisible failures"), §6.3 (A1: "irresolvable fault classification" -- the LFI needs to know when its evidence is insufficient).

- **R26. Observability gap record.** When the trace log lacks data that the IR schema expects (e.g., a numerical health metric that the framework did not log, a controlled variable whose value was not recorded), the IR must explicitly represent the gap rather than silently omitting the element. This is critical for incomplete observability (ARCHITECTURE.md §8.4): the LFI must distinguish "this was checked and is fine" from "this was not checkable." Data: list of (expected_element, gap_reason: {not_logged, framework_limitation, configuration_omission}, severity). Source: ARCHITECTURE.md §8.4 ("when the trace log does not contain the data of the actual failing component, the LFI will misattribute the failure").

#### Hidden Confounder Litmus Test Requirements

The litmus test (evaluation/hidden-confounder/README.md) demands specific IR capabilities beyond the general three-stage audit.

- **R27. Confounder-as-methodological classification support.** The litmus test expects ATHENA to "perform Lakatosian Fault Isolation to explicitly tag the dataset as confounded (a failure of the protective belt, not the core theory)" (VISION.md §7). This means the IR must represent the confounder as a Stage 2 (methodological) issue, not Stage 3 (theoretical). Specifically, the IR must be able to represent: a variable that correlates with the observable (R8) and the intervention (R10) but is not in the controlled set (R13), and that the DAG identifies as a potential confounder (R14). The confounder detection in the litmus test is the canonical test of R14's sufficiency.

- **R28. Interventional vs. observational distinction.** The litmus test's confounder is "discoverable only through interventional experiments that probe confounding structure" (hidden-confounder/README.md §2). The IR must distinguish between results obtained under intervention (the adversarial experiment designer actively varied a parameter) and results obtained under passive observation (the parameter varied naturally). This distinction is critical because confounders that are invisible in observational data become visible under intervention. Data: each observation record (R8, R16) must carry an (observation_mode: {interventional, observational}) tag. Source: hidden-confounder/README.md ("discoverable only through interventional experiments").

- **R29. Cross-experiment queryability.** The litmus test operates over 50 cycles. The LFI must be able to query IR elements across multiple experiments to detect patterns (e.g., a variable that consistently co-varies with the outcome across experiments but was never intervened upon). This extends R24 to multi-experiment scope. Data: every IR element carries an experiment_cycle_id enabling cross-experiment joins. Source: hidden-confounder/README.md ("maximum of 50 experiment execution cycles") and ARCHITECTURE.md §5.1 ("accumulated failure history").

**Implications:**

1. *Straightforward to extract from DSL traces (R1, R2, R6, R7):* Execution completion status, exceptions, numerical health, and resource state are directly emitted by DSL frameworks as log messages, error codes, and status reports. These are the most tractable requirements. The DSL trace format surveys (Step 1) should confirm this for OpenMM, GROMACS, and VASP specifically.

2. *Require matching trace data against experiment specifications (R3, R4, R5, R10, R13):* Input validation, intervention specification, and controlled variable identification require comparing what the experiment specification declared against what the trace log records as actually executed. The IR must bridge two data sources (specification + trace), not just parse one. This is tractable but requires a well-defined experiment specification format.

3. *Require DAG context to populate (R9, R11, R14, R18, R27):* Several requirements involve linking IR elements to DAG nodes and edges. The IR does not store the DAG, but it must be joinable to it. This means the IR's variable naming and identification scheme must be compatible with the DAG's node identification scheme. This is a coordination requirement between the Trace Semantics Engine and the Causal Graph Manager.

4. *Require hypothesis context to populate (R15, R23):* Prediction records come from the hypothesis, not from the trace. The IR must incorporate hypothesis-derived data or be joinable to the hypothesis structure. This means the IR is not purely a trace-derived artifact -- it is a composite of trace data, experiment specification, and hypothesis predictions.

5. *May be partially unobservable (R25, R26):* The IR must represent its own gaps. This is the honest response to ARCHITECTURE.md §8.4. The IR will inevitably be incomplete for some experiments; the question is whether the incompleteness is visible or silent.

6. *Require inference or derivation, not direct extraction (R5, R17):* Input validation results and prediction-observation comparisons are computed from other IR elements, not read from trace logs. The IR must support derived elements, not just raw extractions.

7. *R19 (layer tagging) is the load-bearing requirement.* Without the implementation/theory layer distinction, the entire three-stage structure collapses. The DSL's API separation is what makes this possible (ARCHITECTURE.md §3.1), but the IR must faithfully preserve it. If the layer tag is wrong for any element, the LFI may skip Stage 1 checks that should have caught an implementation error, or apply Stage 1 checks to theory-layer elements.

8. *R28 (interventional vs. observational) is critical for the litmus test but not explicitly required by the three-stage audit text.* This requirement is derived from the litmus test specification, not from §5.3 directly. It represents a gap: the ARCHITECTURE.md audit description does not explicitly distinguish interventional from observational evidence, but the litmus test cannot be passed without this distinction.

**Open Threads:**

1. **Dependency on Step 1 (DSL trace survey).** Requirements R1, R2, R6, R7 assert that certain data is "directly extractable from DSL traces." The Step 1 survey must confirm this for each target framework. If a framework does not log numerical health metrics (R6) or resource state (R7) by default, the requirement is valid but the extraction is harder -- it may require custom logging configurations.

2. **Variable naming coordination.** Requirements R9, R11, R14 require the IR's variable names to be joinable to DAG node identifiers. This implies a shared ontology or naming convention between the Trace Semantics Engine and the Causal Graph Manager. This coordination is not addressed by any current research investigation and may need its own decision.

3. **Composite IR vs. trace-only IR.** The findings show the IR is not a pure trace-log derivative. It incorporates experiment specification data (R3, R10, R13), hypothesis data (R15), and DAG references (R9, R11, R14, R18). The Step 5 schema evaluation should explicitly address whether the IR is a single composite structure or a set of joinable structures with defined interfaces.

4. **Cross-experiment scope.** R29 extends the IR's scope from single-experiment to multi-experiment. This has implications for IR storage and lifecycle that Step 5 must address.

5. **Derived elements.** R5 and R17 are computed from other IR elements. The IR schema must define whether these are stored or computed on demand. This affects queryability (R24) performance.

6. **Ambiguity threshold.** R25 requires "classification confidence metadata" but does not specify what threshold constitutes "insufficient confidence" for escalation. This is an LFI design decision, not an IR design decision, but the IR must provide the raw material for confidence computation.

## Accumulated Findings

### What We Know

1. The IR must represent a minimum of 29 distinct semantic elements (R1-R29) to support the LFI's three-stage audit as specified in ARCHITECTURE.md §5.3. This count is derived by backward analysis from each deterministic question the LFI must answer. [Log: 2026-02-20]

2. The IR is not a pure trace-log derivative. It is a composite of trace-extracted data (R1, R2, R6, R7, R8, R12, R16), experiment specification data (R3, R4, R10, R13), hypothesis-derived data (R15), computed/derived elements (R5, R17), and DAG cross-references (R9, R11, R14, R18). Any IR schema must account for these multiple data sources. [Log: 2026-02-20]

3. The layer tag (R19: implementation vs. theory) is the load-bearing structural distinction. Without it, the three-stage sequential audit cannot function. The DSL's API separation is what makes this tagging possible. [Log: 2026-02-20, derived from ARCHITECTURE.md §3.1 and §5.3]

4. The IR must explicitly represent its own observability gaps (R26). Silent omission of unobservable elements will cause the LFI to misattribute failures (ARCHITECTURE.md §8.4). [Log: 2026-02-20]

5. Stage 2 requirements (R8-R14) are bounded by DAG accuracy. The IR cannot fix a bad DAG, but it must make DAG dependency transparent by linking every confounder judgment to specific DAG edges consulted. [Log: 2026-02-20, derived from ARCHITECTURE.md §5.3 Stage 2 caveat and §8.5]

### What We Suspect

1. Stage 1 requirements (R1, R2, R6, R7) are the most tractable because DSL frameworks structurally emit execution status, exceptions, numerical health, and resource state as part of their normal logging. Pending confirmation by Step 1 DSL trace survey. [Log: 2026-02-20]

2. The interventional vs. observational distinction (R28) may be a gap in ARCHITECTURE.md §5.3. The audit description does not explicitly require it, but the hidden confounder litmus test cannot be passed without it. This suggests either §5.3 is underspecified or the litmus test imposes requirements beyond the three-stage audit. [Log: 2026-02-20]

3. A shared variable naming ontology between the Trace Semantics Engine and the Causal Graph Manager is an implicit requirement (from R9, R11, R14) not addressed by any current research investigation. This may need its own coordination decision. [Log: 2026-02-20]

### What We Don't Know

1. Whether OpenMM, GROMACS, and VASP trace logs actually contain sufficient data to populate R1-R7 without custom logging configurations. The Step 1 DSL trace survey must answer this. [Log: 2026-02-20]

2. Whether the IR should be a single composite structure or a set of joinable structures with defined interfaces. The composite nature (trace + specification + hypothesis + DAG references) creates a design tension between cohesion and modularity. Step 5 must address this. [Log: 2026-02-20]

3. What classification confidence threshold (R25) separates determinate from ambiguous classifications. This is an LFI design question, but the IR must provide the input data for confidence computation. [Log: 2026-02-20]

4. How cross-experiment queryability (R29) interacts with IR storage and lifecycle. Single-experiment IR is conceptually simpler; multi-experiment IR requires aggregation and indexing decisions. [Log: 2026-02-20]

5. Whether derived IR elements (R5: input validation results, R17: prediction-observation comparisons) should be stored or computed on demand. This affects queryability (R24) and has performance implications. [Log: 2026-02-20]

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| *None yet* | | | |

## Next Steps

1. **Survey DSL trace formats** — Collect and document the actual trace output structure from OpenMM, GROMACS, and VASP. Identify common elements (timestamped events, state dumps, error codes) and framework-specific elements. Scope: 2-3 sessions.

2. **Survey existing IR designs in RCA and formal verification** — Review intermediate representations used in root cause analysis tools, formal verification (SAT solvers, model checkers), and program analysis. Identify which design patterns transfer to DSL trace parsing. Scope: 2-3 sessions.

3. **Map LFI three-stage audit backwards to minimum IR requirements** — Starting from the three audit stages in ARCHITECTURE.md 5.3, derive the minimum set of semantic distinctions the IR must represent. What must Stage 1 (implementation audit) be able to query? Stage 2 (methodology audit)? Stage 3 (theory evaluation)? Scope: 1-2 sessions.

4. **Characterize the 21% baseline and DSL improvement** — Understand what drives the low accuracy of general RCA on unstructured traces, and what structural properties of DSL environments improve it. Identify the specific features of DSL traces that make them more amenable to semantic parsing. Scope: 1-2 sessions.

5. **Draft candidate IR schemas** — Based on the above, propose 2-3 candidate IR designs with explicit tradeoffs (resolution vs. generality vs. implementation cost). Each should be evaluated against the minimum requirements from step 3. Scope: 2-3 sessions.
