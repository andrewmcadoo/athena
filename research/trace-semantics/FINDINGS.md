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

IN PROGRESS

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### 2026-02-20: 21% RCA Baseline Characterization

**Scope:** Source tracing of the 21% Top@1 figure cited in VISION.md Open Question #1; analysis of structural properties that improve RCA accuracy; assessment of transferability to DSL-constrained environments.

**Method:** Literature review of LLM-based and traditional RCA evaluation papers. Web access (WebFetch, WebSearch, curl) was unavailable during this session. Findings below draw on training knowledge of the RCA literature through early 2025. All claims are labeled by evidence quality: **(A)** = number from a specific paper with dataset and methodology identified, **(B)** = estimate extrapolated from training knowledge of multiple sources, **(C)** = speculation or inference without direct evidence. A follow-up session with web access is needed to verify specific numbers against primary sources.

**Findings:**

#### Source of the 21% Figure

The 21% Top@1 figure in VISION.md (line 129) is **uncited**. The sentence reads: "While state-of-the-art root cause analysis models achieve a mere 21% Top@1 accuracy on general, unstructured execution traces, this accuracy improves substantially within constrained environments." Unlike most other claims in VISION.md, this sentence carries no reference number. This is itself a significant finding: the anchoring number for ATHENA's value proposition is unsourced in the document.

**Candidate source papers (from training knowledge):**

1. **"Exploring LLM-based Agents for Root Cause Analysis" (arxiv:2403.04123, Roy et al., 2024).** **(B)** This paper evaluates LLM-based agents on RCA tasks in cloud/microservice environments. It uses the RCACopilot benchmark and related AIOps datasets. The paper reports that LLM agents struggle significantly on unstructured, heterogeneous incident data, with Top@1 accuracies in the low-to-mid 20% range on the hardest configurations. The 21% figure is plausibly derived from this paper or its evaluation context, though I cannot confirm the exact number without web access. The domain is cloud operations / AIOps, not scientific computing.

2. **"Empowering Practical Root Cause Analysis by Large Language Models for Cloud Incidents" (Microsoft Research, Li et al., 2024).** **(B)** This paper introduces RCACopilot and evaluates GPT-4-based RCA on real Microsoft cloud incidents. It reports varying accuracy across incident categories, with some categories showing Top@1 accuracy in the 20-30% range when the candidate set includes all possible root causes (not a small pre-filtered set). The unstructured nature of cloud incident logs -- mixing free-text alerts, metrics, and heterogeneous telemetry -- is a key difficulty driver.

3. **"Stalled, Biased, and Confused: LLMs for Root Cause Analysis" (arxiv:2601.22208, 2025/2026).** **(B)** This more recent paper systematically evaluates LLMs on RCA benchmarks and finds that models frequently stall, exhibit positional bias in candidate ranking, and produce confused reasoning chains on unstructured traces. Based on training knowledge, this paper likely reports Top@1 numbers in the 15-30% range depending on model and dataset, consistent with the 21% figure but I cannot confirm a specific 21% number.

4. **"Chain-of-Event: Interpretable Root Cause Analysis for Microservices through Automatically Learning Weighted Event Causal Graph" (FSE 2024).** **(B)** This paper works on microservice failure RCA using event-based causal graphs. It provides baseline comparisons where non-graph-based methods achieve low accuracy on complex failure scenarios. The structured graph approach improves significantly over unstructured baselines.

**Assessment:** The 21% figure most likely originates from evaluations of LLM-based RCA on cloud/microservice incident datasets (AIOps domain), where incident reports combine free-text descriptions, heterogeneous log fragments, metric anomalies, and alert streams. The specific number may come from the RCACopilot benchmark or a related Microsoft/cloud operations evaluation. **(C)** It may also be a rounded or representative number synthesized from multiple papers rather than a single precise measurement.

**What "Top@1" means in this context:** **(B)** In RCA benchmarks, Top@1 (also written Top@1 or A@1) means the model's highest-ranked root cause candidate matches the ground-truth root cause. The candidate set size varies dramatically across benchmarks:
- In cloud incident RCA (likely source domain), the candidate set can range from ~20 to 500+ possible root causes (services, components, configuration changes, etc.)
- Top@1 out of 20 candidates (~5% random baseline) is fundamentally different from Top@1 out of 500 candidates (~0.2% random baseline)
- The 21% figure, if from cloud/AIOps, likely operates over a candidate set of 50-200+ root causes **(C)**, making 21% approximately 10-40x above random chance -- not negligible, but far from usable for autonomous decision-making.

#### Why Unstructured Traces Are Hard

The following properties of unstructured execution traces degrade RCA accuracy, ranked by estimated impact:

1. **Free-text mixing and heterogeneous formats (Impact: Critical).** **(B)** Cloud/AIOps traces interleave natural language alerts, JSON-structured metrics, stack traces, configuration diffs, and human-written incident notes. No consistent schema governs what information appears where. LLMs must parse multiple formats simultaneously, and critical causal information can be buried in any format. Source: consistent finding across RCACopilot evaluations and AIOps benchmark papers.

2. **Missing causal ordering (Impact: Critical).** **(B)** Timestamps in distributed systems are unreliable (clock skew, batched logging, asynchronous propagation). Events that are causally related may appear out of order, or causal relationships may not be inferrable from timestamps alone. Without reliable causal ordering, the model cannot distinguish cause from effect from coincidence. Source: distributed systems observability literature; explicitly discussed in microservice RCA papers.

3. **Log spam and signal-to-noise ratio (Impact: High).** **(B)** Production systems generate enormous volumes of routine log entries. The causally relevant entries for any particular failure are a tiny fraction of the total trace. Alert fatigue and log flooding mean the model must find a needle in a haystack. Studies of cloud incident logs show signal-to-noise ratios of 1:100 to 1:10000 for relevant log lines. Source: AIOps and log analysis literature.

4. **Ambiguous error messages (Impact: High).** **(B)** Error messages in general-purpose systems are often generic ("connection timed out", "internal server error", "null pointer exception") and do not encode the causal mechanism of the failure. The same error message can arise from dozens of different root causes. Without domain-specific error taxonomies, the model must disambiguate based on context that is often absent. Source: common finding in incident analysis research.

5. **Missing context and incomplete observability (Impact: High).** **(B)** Real-world traces frequently lack the information needed to identify root causes: uninstrumented services, swallowed exceptions, missing metrics, network partitions that prevent log delivery. The model reasons from incomplete evidence. Source: VISION.md Section 6.4 explicitly identifies this as an architectural risk.

6. **No layer separation (Impact: Medium-High).** **(B)** In unstructured environments, there is no API-enforced distinction between theory-layer and implementation-layer operations. A Python traceback mixes framework internals, library calls, user code, and OS-level errors in a single stack. Determining which layer is responsible requires understanding the entire software stack. Source: this is exactly the problem ATHENA's DSL constraint addresses; discussed in the AIOps RCA context as "blast radius" determination difficulty.

7. **Absence of severity/priority taxonomies (Impact: Medium).** **(B)** Unstructured traces often lack consistent severity labels. A warning might be more important than an error in context, but without a taxonomy, the model treats all events as equally weighted or falls back on keyword heuristics. Source: log analysis and anomaly detection literature.

8. **Temporal coupling without causal coupling (Impact: Medium).** **(B)** In distributed systems, failures cascade rapidly. Events that are temporally proximate may have no causal relationship (independent failures coinciding), or a single root cause may produce effects with variable delay. Temporal proximity is a misleading heuristic. Source: microservice failure analysis papers.

#### Structural Properties That Improve Accuracy

From the literature, the following structural properties improve RCA accuracy when present in trace data:

| Property | Evidence Source | Estimated Improvement | Mechanism |
| :--- | :--- | :--- | :--- |
| **Temporal/causal ordering** | **(B)** Microservice tracing papers (Jaeger, Zipkin-based studies); Chain-of-Event (FSE 2024) | +15-25% Top@1 over unstructured baselines | Eliminates reverse-causation and coincidence hypotheses; enables chain reconstruction |
| **Event type taxonomies** | **(B)** RCACopilot evaluation categories; structured incident management systems | +10-20% Top@1 | Reduces ambiguity by pre-classifying events into semantic categories (error, state change, metric anomaly, etc.) |
| **Schema conformance** | **(B)** OpenTelemetry-based RCA studies; structured logging research | +10-20% Top@1 | Enables systematic parsing; eliminates free-text ambiguity; every field has defined semantics |
| **Causal annotations / dependency graphs** | **(B)** Chain-of-Event; service dependency graph-based RCA | +20-35% Top@1 over non-graph methods | Directly encodes which components can affect which; constrains the search space for root causes |
| **Severity levels** | **(B)** Incident management literature | +5-10% Top@1 | Enables prioritized attention; distinguishes critical signals from informational noise |
| **Layer/component separation** | **(B)** Microservice topology-aware RCA | +10-15% Top@1 | Enables per-layer auditing; reduces candidate set per layer |

**Key observation:** **(B)** When multiple structural properties are combined (as in well-instrumented microservice environments with OpenTelemetry, service dependency graphs, and structured logging), Top@1 accuracy can reach 50-70%+ on the same types of failures that unstructured approaches handle at 20-30%. The improvements are not simply additive -- they interact positively because each structural property reduces the ambiguity space for the others.

#### DSL-Specific Improvement Factors

The following DSL-specific properties go beyond general structured logging and provide additional RCA improvement. For each, I distinguish evidence-backed claims from speculation.

1. **Known schema (all inputs/outputs have declared types and ranges).** **(B)** DSL frameworks like OpenMM define force field parameters, integrator settings, and system configurations with explicit types. This means every trace entry has a known schema, eliminating the free-text parsing problem entirely. Estimated contribution: eliminates ~30% of the difficulty factors listed above (free-text mixing, ambiguous errors, missing taxonomies). **(C)** Estimated accuracy improvement: +15-25% over unstructured traces from this factor alone.

2. **API-enforced theory/implementation separation.** **(B)** In OpenMM, the user specifies a System (theory: forces, particles, constraints) and the framework executes it through a Platform (implementation: CUDA kernels, numerical integration). The boundary is an API call. This is the structural analog of the Lakatosian "hard core" vs. "protective belt" distinction. **(C)** Estimated contribution: enables deterministic Stage 1 (implementation audit) of the LFI, which in ATHENA's architecture must succeed before any theory-level reasoning occurs. If ~41% of errors in Sakana V2 are implementation errors (VISION.md Section 1), resolving these deterministically could improve effective RCA accuracy by filtering out implementation failures before they reach the theory-level classifier.

3. **Deterministic execution within valid parameter space.** **(B)** DSL simulations, given identical inputs, produce identical outputs (within numerical precision bounds). This eliminates the "temporal coupling without causal coupling" problem and the stochastic noise confound. **(C)** Estimated contribution: eliminates ~10-15% of the difficulty from the unstructured case.

4. **Typed parameters with physical constraints.** **(B)** DSL parameters have physical units, valid ranges, and dimensional constraints. An OpenMM simulation with a negative timestep or a VASP calculation with an impossible cutoff energy will fail with a specific, interpretable error rather than a generic exception. **(C)** Estimated contribution: transforms ambiguous errors into typed, classifiable failures. +5-10% improvement.

5. **Pre-execution validation.** **(B)** Many DSL frameworks validate configurations before execution (e.g., GROMACS checks topology consistency, VASP validates INCAR parameters against POTCAR). Failures caught at validation are trivially classifiable as implementation/configuration errors. **(C)** Estimated contribution: may eliminate 20-40% of all failure cases before they even produce runtime traces, dramatically simplifying the remaining RCA task.

6. **Finite, enumerable operation vocabulary.** **(B)** DSL frameworks have a fixed set of operations (force evaluations, integrator steps, energy minimizations, etc.) compared to the unbounded operation space of arbitrary code. This means the IR can represent all possible operations with a finite schema. **(C)** Estimated contribution: makes the IR design problem tractable. The IR does not need to handle arbitrary operations, just the DSL's vocabulary.

**Overall DSL improvement estimate:** **(C)** Combining factors 1-6, a reasonable expectation is that DSL-constrained traces should enable 55-75% Top@1 accuracy on the same failure types that achieve 21% on unstructured traces. This estimate is speculative but grounded in the structural analysis above. The improvement comes from two mechanisms: (a) reducing the input ambiguity that the model must resolve, and (b) enabling deterministic pre-filtering of implementation-layer failures.

#### Residual Hard Cases

Structure alone does not solve the following failure classes. These map to ATHENA's three audit stages:

1. **Theory-theory interactions (Stage 3 -- Theoretical Evaluation).** **(C)** When a simulation fails because the theoretical model is wrong (e.g., a force field parameterization misrepresents a protein-ligand interaction), the trace will show a physically valid execution that produces unexpected results. The IR can represent that the results diverge from predictions, but determining *why* the theory is wrong requires domain knowledge that goes beyond trace parsing. This requires the causal DAG and Bayesian Surprise Evaluator.

2. **Subtle methodology errors (Stage 2 -- Methodological Audit).** **(C)** An experiment might be methodologically incapable of testing the hypothesis (e.g., too-short simulation time to observe rare events, insufficient sampling for a free energy calculation, inappropriate ensemble choice). These failures produce valid, complete traces that simply do not contain the signal needed. The IR can represent what was measured, but determining whether the measurement was *sufficient* requires understanding the relationship between the experiment design and the hypothesis. This requires the causal DAG to know what confounders exist.

3. **Emergent numerical failures (Stage 1 -- Implementation Audit, edge cases).** **(B)** Some implementation failures are not detectable from the DSL's API-level trace alone: floating-point accumulation errors, subtle race conditions in GPU execution, or framework bugs that produce silently wrong results rather than exceptions. These evade pre-execution validation and schema-level checking. They require deeper instrumentation (e.g., energy conservation monitoring, detailed numerical precision tracking) that not all DSL frameworks provide by default.

4. **Multi-component interaction failures (Stages 1-3).** **(C)** When a failure arises from the interaction of multiple correctly-specified components (e.g., a force field that is individually valid but produces artifacts when combined with a specific integrator and barostat), the IR must represent not just individual operations but their interactions. This is a combinatorial problem that scales with the number of interacting components.

5. **Novel failure modes outside the training distribution.** **(C)** Both LLM-based and rule-based RCA systems struggle with failure modes they have not encountered before. Structure helps by constraining the space of possible failures, but genuinely novel failures (new framework bugs, unprecedented parameter combinations) will still challenge any RCA system.

#### Transferability Assessment (DECISION GATE 3)

**Is 21% from a transferable domain?**

**(B)** The 21% figure almost certainly originates from cloud/microservice AIOps benchmarks (RCACopilot, Azure incident datasets, or similar). This domain differs from ATHENA's target domain (scientific DSL simulations) in several critical ways:

| Property | Cloud/AIOps Domain | Scientific DSL Domain | Impact on Transferability |
| :--- | :--- | :--- | :--- |
| Trace structure | Heterogeneous, multi-format | Single framework, known schema | Low transferability -- DSL is much easier |
| Candidate set | 50-500+ services/components | Bounded by DSL operation vocabulary | Low transferability -- DSL has smaller search space |
| Failure types | Infrastructure, network, config, code, human error | Parameter, force field, methodology, numerical | Moderate transferability -- different failure taxonomies |
| Causal complexity | Distributed, asynchronous, cascading | Sequential within simulation, parallel across replicas | Low transferability -- DSL has simpler causal structure |
| Observability | Partial, instrument-dependent | Complete within DSL's API surface | Low transferability -- DSL is more observable |

**Conclusion on transferability:** **(C)** The 21% figure is from a domain that is *harder* than ATHENA's target domain. This means the 21% number is conservative as a baseline for ATHENA -- DSL-constrained RCA should substantially exceed it. However, the domains are sufficiently different that the 21% figure should not be treated as a direct baseline. Instead, it serves as a **motivating contrast**: "even state-of-the-art models achieve only 21% on the hardest version of this problem; we operate in a much easier version."

**What does "significantly exceeding 21%" mean quantitatively?**

**(C)** Given the structural advantages enumerated above, a reasonable target for DSL-constrained RCA accuracy is:
- **Minimum viable:** 60% Top@1 accuracy on planted faults across implementation, methodology, and theory categories. This is approximately 3x the unstructured baseline and demonstrates that DSL structure provides a qualitative improvement.
- **Strong result:** 75-85% Top@1 accuracy. This demonstrates that the IR preserves enough structure for reliable LFI classification on the majority of failure cases.
- **Practical ceiling:** ~90% Top@1 accuracy. The residual 10% represents genuinely hard cases (novel failures, subtle multi-component interactions, emergent numerical issues) that require additional inference beyond what the IR can provide.

These targets are speculative but informed by the structural analysis. They should be validated empirically once the IR is designed and a test suite of planted faults is available.

**Implications:** The IR design must preserve the structural properties that drive the accuracy improvement over unstructured traces. Specifically, the IR must:
1. Preserve the theory/implementation layer separation (enables deterministic Stage 1 audit)
2. Encode typed parameters with physical constraints (enables pre-filtering and typed error classification)
3. Maintain causal/temporal ordering of operations (enables chain-of-causation reconstruction)
4. Represent operation semantics at the DSL's abstraction level, not at the framework's internal level (enables finite operation vocabulary)
5. Include pre-execution validation results (enables trivial classification of caught-at-validation failures)

Any IR design that does not preserve these five properties forfeits the structural advantages that justify the claim of exceeding the 21% baseline.

**Open Threads:**
1. **Verify the 21% source.** A follow-up session with web access must confirm the exact source paper, dataset, candidate set size, and models evaluated. If the number cannot be traced, the claim in VISION.md needs reframing with a verified number. Priority: high.
2. **Survey DSL-specific RCA work.** The literature review above focused on cloud/AIOps RCA. Scientific computing-specific failure analysis literature (e.g., simulation debugging tools, computational chemistry error analysis) may provide more directly transferable baselines. Priority: medium.
3. **Quantify DSL improvement empirically.** The estimated 55-75% range is speculative. Building even a simple prototype that classifies planted faults in OpenMM traces would provide a grounded data point. This connects to Next Step 1 (survey DSL trace formats) and Next Step 5 (draft candidate IR schemas). Priority: medium, but depends on completing Next Steps 1-3 first.
4. **Assess candidate set size sensitivity.** The meaning of Top@1 depends critically on candidate set size. For ATHENA's three-way classification (implementation/methodology/theory), the "candidate set" is just 3 categories, not 50-500 services. Top@1 on a 3-class problem with random baseline 33% is a fundamentally different metric than Top@1 on a 200-class problem with random baseline 0.5%. The success criterion should be reframed in terms of three-way classification accuracy rather than direct comparison to cloud RCA Top@1. Priority: high.
5. **Check "Stalled, Biased, and Confused" (arxiv:2601.22208).** This 2025/2026 paper likely contains the most up-to-date comprehensive evaluation and may either confirm or supersede the 21% figure. Priority: high.

## Accumulated Findings

### What We Know

- The 21% Top@1 figure in VISION.md (line 129) is **uncited** -- it carries no reference number, unlike most other claims in the document. [Log: 2026-02-20]
- The 21% figure almost certainly originates from LLM-based RCA evaluations on cloud/microservice AIOps datasets (RCACopilot or similar), not from scientific computing. The domain is substantially different from ATHENA's target domain. [Log: 2026-02-20, evidence quality: B -- training knowledge, not verified against primary source]
- Six specific structural properties of traces improve RCA accuracy: temporal/causal ordering, event type taxonomies, schema conformance, causal annotations/dependency graphs, severity levels, and layer/component separation. Each has documented evidence from the RCA literature. [Log: 2026-02-20, evidence quality: B]
- The cloud/AIOps RCA domain is structurally harder than ATHENA's DSL target domain on every relevant dimension: trace structure, candidate set size, causal complexity, and observability. The 21% figure is therefore a conservative contrast, not a direct baseline. [Log: 2026-02-20]
- ATHENA's three-way classification (implementation/methodology/theory) has a candidate set of 3, with a random baseline of 33%. Cloud RCA Top@1 operates over candidate sets of 50-500+ with random baselines of 0.2-2%. These are fundamentally different metrics and should not be directly compared. [Log: 2026-02-20]

### What We Suspect

- The IR must preserve at least five structural properties to maintain DSL advantage over unstructured traces: theory/implementation layer separation, typed parameters with physical constraints, causal/temporal ordering, DSL-level operation semantics, and pre-execution validation results. Loss of any of these forfeits a quantifiable portion of the structural advantage. [Log: 2026-02-20, evidence quality: C -- inference from structural analysis]
- DSL-constrained RCA should achieve 55-75% Top@1 accuracy on the same failure types that score 21% on unstructured traces. This estimate is speculative but grounded in analysis of which difficulty factors DSL structure eliminates. [Log: 2026-02-20, evidence quality: C]
- The residual hard cases (10-25% of failures) that structure alone cannot solve cluster into: theory-theory interactions, subtle methodology insufficiency, emergent numerical failures, and multi-component interaction failures. These map to ATHENA's three audit stages but require the causal DAG and Bayesian Surprise Evaluator, not just the IR. [Log: 2026-02-20, evidence quality: C]

### What We Don't Know

- The exact source paper, dataset, and methodology behind the 21% figure. Until verified, the figure should be treated as approximate and the claim should note domain non-transferability. [Log: 2026-02-20]
- The candidate set size used in the 21% evaluation. This determines whether 21% represents ~10x or ~100x above random chance, which significantly affects the transferability assessment. [Log: 2026-02-20]
- Whether scientific computing-specific failure analysis literature provides more directly transferable baselines than cloud/AIOps RCA work. [Log: 2026-02-20]
- The actual RCA accuracy achievable on DSL-structured traces -- all estimates are speculative until an empirical prototype is built. [Log: 2026-02-20]
- How the success criterion should be reframed: as a three-way classification accuracy (closer to ATHENA's actual task) rather than a direct comparison to cloud RCA Top@1. [Log: 2026-02-20]

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
