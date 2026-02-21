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

IN PROGRESS — DSL trace format survey underway (OpenMM complete, GROMACS and VASP pending).

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### 2026-02-20: OpenMM Trace Format Characterization

**Scope:** Complete characterization of OpenMM's trace output system, mapping every output element to theory, implementation, or boundary layer. Covered: reporter system inventory (7 reporter types), theory-implementation API boundary analysis (ForceField/Topology/System vs. Platform/Context), exception and error exposure, execution metadata accessibility, custom reporter extensibility, NaN energy failure walkthrough, and failure mode taxonomy (17 modes across 4 categories).

**Method:** Documentation review (OpenMM Python API docs at docs.openmm.org, OpenMM User Guide chapters 3, 4, and 8), source code analysis (openmm/app/ Python wrappers: simulation.py, statedatareporter.py, dcdreporter.py, pdbreporter.py, pdbxreporter.py, checkpointreporter.py, forcefield.py, topology.py), and failure pattern analysis from OpenMM GitHub issue tracker (NaN energy, precision, constraint failure threads).

**Findings:**

1. **OpenMM enforces a clean theory-implementation API boundary.** The ForceField/Topology/System chain defines the theory specification; Platform/Context define the implementation. The `ForceField.createSystem()` method is the explicit compilation boundary. The System object's contents (forces, parameters, constraints) are fully queryable via the API, making post-compilation auditing tractable. However, the atom type assignment trail is lost at the `createSystem()` boundary — the System does not record which force field atom types were matched to which topology atoms. (Source: `openmm/app/forcefield.py`, `createSystem()` method; detailed in `dsl-evaluation/openmm-trace-analysis.md` Section 2.3.)

2. **Default trace output is insufficient for three-way fault classification.** Of 17 cataloged failure modes, only 4 are definitively detectable and classifiable from default reporters (GPU memory exhaustion, driver incompatibility, and partially force field template matching errors). The remaining 13 either go undetected or are detected without category-distinguishing information. The most critical gap: NaN energy failures are ambiguous between implementation (precision overflow), methodology (time step too large), and theory (bad force field parameters), and default reporters provide no data to distinguish them. (Source: failure mode taxonomy in `dsl-evaluation/openmm-trace-analysis.md` Section 7.5.)

3. **The reporter API is extensible enough for custom ATHENA instrumentation.** Custom reporters receive the full Simulation, Context, and State objects, enabling per-force-group energy decomposition, per-atom force monitoring, and adaptive reporting intervals. The main gap is sub-step instrumentation — reporters fire between steps, not within them, so crash-time state from mid-step failures is unrecoverable via the reporter API. (Source: `openmm/app/simulation.py` reporter invocation logic; detailed in `dsl-evaluation/openmm-trace-analysis.md` Section 5.)

4. **Methodology-layer failures are invisible to OpenMM.** The framework has no concept of scientific adequacy — insufficient equilibration, inadequate sampling, wrong ensemble choice, and correlation between samples are never detected or reported. An ATHENA IR for OpenMM must incorporate methodology assessment as external domain rules, not as parsed trace data. (Source: analysis of OpenMM exception types in `dsl-evaluation/openmm-trace-analysis.md` Section 3.1.)

5. **Temporal resolution of reporters creates diagnostic blind spots.** Reporters fire at configured intervals (typically every 1000-10000 steps). Events between intervals are invisible. In the NaN walkthrough, up to 2347 steps of energy divergence occurred between the last normal report and the crash, with no recorded state for that interval. (Source: NaN walkthrough in `dsl-evaluation/openmm-trace-analysis.md` Section 6.)

**Implications:**

- The IR cannot operate on default OpenMM trace output alone. A custom ATHENA reporter is a prerequisite for effective fault isolation. This reporter should capture per-force-group energy decomposition, monitor force magnitudes, and implement adaptive reporting frequency.
- The theory-implementation boundary in OpenMM is clean enough for Stage 1 (implementation audit) of the LFI decision tree. The Platform/Context separation allows deterministic checking of hardware state, precision mode, and platform compatibility. Stage 2 (methodology audit) and Stage 3 (theory evaluation) require external criteria that OpenMM does not provide.
- The IR must explicitly represent the ForceField -> createSystem() -> System -> Context compilation chain as a layered structure, preserving the theory-implementation separation at each level.
- The atom type assignment gap at `createSystem()` is a specific weak point: if a wrong atom type is assigned (due to ambiguous topology), the error is silent after compilation. The IR should flag this as a potential ambiguity zone requiring supplementary auditing.
- OpenMM's failure mode taxonomy provides a concrete test suite for IR validation: planted faults from each of the 17 failure modes can serve as ground-truth test cases for fault classification accuracy.

**Open Threads:**

1. How do GROMACS and VASP compare on theory-implementation boundary cleanliness? Do they provide richer default trace output?
2. Can the sub-step instrumentation gap be closed by using OpenMM's `CustomIntegrator` to insert monitoring operations within the integration step?
3. What is the computational overhead of per-force-group energy decomposition at every reporting interval? Is it feasible for production simulations?
4. How should the IR represent the "unknown state" in temporal gaps between reporter intervals?
5. Can the atom type assignment trail be reconstructed by re-running `createSystem()` with instrumentation, or must it be captured at compilation time?

## Accumulated Findings

### What We Know

1. **OpenMM enforces a structurally clean theory-implementation API boundary.** The ForceField/Topology/System chain (theory) is separated from Platform/Context (implementation) by the framework's API design. The `ForceField.createSystem()` method serves as an explicit compilation step between these layers, and the resulting System object's parameters are fully queryable. This confirms that the DSL-only constraint (ARCHITECTURE.md 3.1) is well-satisfied by OpenMM. (Evidence: OpenMM API analysis, log entry 2026-02-20, `dsl-evaluation/openmm-trace-analysis.md` Section 2.)

2. **Default OpenMM trace output is insufficient for LFI three-way fault classification.** Only 4 of 17 cataloged failure modes are definitively detectable and classifiable from default reporter output. The most critical diagnostic failure: NaN energy events are ambiguous across all three fault categories (implementation, methodology, theory) without supplementary per-force-group energy decomposition data. (Evidence: failure mode taxonomy, log entry 2026-02-20, `dsl-evaluation/openmm-trace-analysis.md` Section 7.5.)

3. **OpenMM's reporter API supports custom instrumentation sufficient for enhanced trace capture.** The two-method reporter protocol (`describeNextReport`, `report`) grants access to the full Simulation, Context, and State objects, enabling per-force-group energy decomposition, per-atom force/velocity monitoring, and adaptive reporting intervals without modifying OpenMM's C++ core. (Evidence: reporter API analysis, log entry 2026-02-20, `dsl-evaluation/openmm-trace-analysis.md` Section 5.)

4. **Methodology-layer failures are invisible to OpenMM's error reporting.** The framework does not detect or report insufficient equilibration, inadequate sampling, wrong ensemble choice, or sample correlation. These failures require external domain-specific assessment criteria. (Evidence: exception catalog, log entry 2026-02-20, `dsl-evaluation/openmm-trace-analysis.md` Section 3.)

### What We Suspect

1. **A custom ATHENA reporter capturing per-force-group energy decomposition would resolve most NaN ambiguity.** If the reporter decomposes total energy into contributions from each Force object (bonds, angles, nonbonded, etc.) at each reporting interval, the divergent force term can be identified, narrowing the fault classification. The remaining ambiguity (precision vs. parameters within a single force term) may require additional cross-platform comparison. (Basis: the OpenMM API supports `getState(groups={i})` for energy decomposition; untested whether the overhead is acceptable for production simulations.)

2. **The atom type assignment gap at `createSystem()` is a tractable problem.** The lost mapping between force field atom types and topology atoms could be reconstructed by instrumenting `createSystem()` itself (e.g., wrapping the method to log type assignments) or by post-hoc comparison of System parameters against ForceField XML. This is likely an engineering task, not a research problem. (Basis: ForceField XML and System parameters are both accessible; the matching algorithm is deterministic.)

3. **GROMACS may provide richer default trace output than OpenMM.** GROMACS's `.edr` energy files include per-term energy decomposition by default, and its `.log` files contain detailed performance and diagnostic information. If confirmed, this would mean the IR design should not assume OpenMM-level trace sparsity as the baseline. (Basis: general knowledge of GROMACS output; requires formal survey to confirm.)

### What We Don't Know

1. **Whether sub-step instrumentation is achievable via `CustomIntegrator`.** OpenMM's `CustomIntegrator` allows defining integration steps as a sequence of computations. It may be possible to insert energy evaluation or state logging operations between sub-steps. If feasible, this would close the temporal gap between reporter intervals and enable crash-proximate state capture. (Identified: log entry 2026-02-20, open thread #2.)

2. **The computational overhead of per-force-group energy decomposition at every reporting interval.** Each `getState(groups={i})` call requires a separate reduction on the GPU. For systems with many force groups (5-10), this could significantly slow the simulation. The overhead has not been measured. (Identified: log entry 2026-02-20, open thread #3.)

3. **How the IR should represent temporal gaps.** Between reporter intervals, the system state is unknown. The IR needs a formal representation for "state unknown between timestep X and Y" that the LFI can reason about. No existing IR design we have surveyed addresses this. (Identified: log entry 2026-02-20, open thread #4.)

4. **How GROMACS and VASP compare to OpenMM on theory-implementation boundary cleanliness and trace richness.** The OpenMM analysis establishes one data point. Whether the findings generalize across DSL frameworks, or whether each framework requires a distinct IR adapter, is unknown. (Identified: log entry 2026-02-20, open thread #1.)

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
