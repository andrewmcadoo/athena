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

IN PROGRESS — DSL trace format survey (GROMACS complete, OpenMM and VASP pending)

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### 2026-02-20: GROMACS Trace Format Characterization

**Scope:** Complete catalog and classification of GROMACS MD simulation output files (.log, .edr, .trr, .xtc, .xvg, .cpt, .tpr, .gro), mapping each to theory-layer or implementation-layer. Assessment of the .mdp specification interface as a theory-implementation API boundary. Error and warning taxonomy. LINCS constraint failure walkthrough. grompp preprocessing validation coverage analysis.

**Method:** Systematic analysis of GROMACS output architecture based on the GROMACS reference manual (manual.gromacs.org), source code documentation (github.com/gromacs/gromacs), panedr library documentation, MDAnalysis documentation, GROMACS user forum failure cases, and published GROMACS architecture descriptions (Abraham et al. 2015). Each output file was cataloged by format, content, programmatic access method, and layer classification. The .mdp parameter space was partitioned into theory-layer, implementation-layer, and boundary parameters. A concrete LINCS failure was traced through all output files to assess reconstructibility. grompp validation checks were enumerated and classified by what they catch vs. what slips through.

**Findings:**

1. **Output file inventory.** GROMACS produces 8 primary output file types. The .edr (energy time series, binary XDR, accessible via panedr Python library) is the richest structured data source for anomaly detection. The .log (semi-structured text) is the primary source of error messages but lacks machine-readable structure (no error codes, no schema). The .tpr (binary compiled run input) is the complete experiment specification but merges theory and implementation layers into a single opaque object. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 1.

2. **Theory-implementation boundary.** The .mdp parameter file provides a relatively clean theory-implementation boundary. Theory-layer parameters (integrator, tcoupl, pcoupl, coulombtype, force field) are declarative and have no implementation side effects. Implementation-layer parameters (nstlog, nstenergy, nstxout) control execution mechanics only. However, 10+ parameters are "boundary" — they serve dual roles affecting both physics and execution. The most consequential boundary parameter is `dt` (timestep), which is a physical/methodological decision that manifests as implementation-like symptoms when wrong. The mdrun command-line parameters (-ntomp, -gpu_id, -dd) are purely implementation-layer, providing the cleanest separation in the system. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 2.

3. **Error taxonomy.** GROMACS errors are free-text messages with no structured classification. The most common simulation failures (LINCS/SETTLE/SHAKE constraint violations, domain decomposition errors) are inherently ambiguous between theory, methodology, and implementation causes. These ambiguous errors are also the most frequent errors the LFI would need to classify. Purely implementation-layer errors (memory, GPU, MPI, file mismatch) are cleanly identifiable by message pattern but represent a small fraction of real-world failures. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 5.

4. **Failure walkthrough (LINCS).** Tracing a LINCS constraint failure through the output system shows that correct fault classification requires cross-file correlation: .log (error messages and warnings), .edr (energy escalation pattern), .tpr (parameter context), and initial structure (clash detection). No single output file contains sufficient information. A critical gap: the exact crash-state coordinates/velocities/forces are not preserved; only the last periodic checkpoint (potentially thousands of steps before the crash) is available. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 6.

5. **grompp validation.** grompp catches syntactic/structural errors comprehensively (atom count mismatches, missing force field parameters, box size violations) but does not validate physical/scientific correctness. Timestep adequacy, force field correctness for the chemistry, equilibration quality, and sampling sufficiency all slip through to runtime. This creates a clean audit hierarchy: grompp validates implementation syntax, runtime monitoring validates numerical stability, and post-run analysis validates physical correctness. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 7.

6. **Six concrete IR requirements derived.** The analysis produced six specific requirements for the IR design: (a) GROMACS parameter classification table, (b) cross-file correlation engine, (c) temporal event linking, (d) error pattern library, (e) data absence tracking, (f) user-specified vs. runtime-adjusted parameter distinction. Full details in `dsl-evaluation/gromacs-trace-analysis.md`, Section 8.3.

**Implications:**

- GROMACS provides sufficient structured data for the Trace Semantics Engine to operate, but the IR must perform substantial work to bridge the gap between raw output and semantic failure representations. The .edr time series (via panedr) is the most IR-friendly data source. The .log error messages are the least IR-friendly.
- The theory-implementation boundary is cleaner than expected for most parameters, but the 10+ boundary parameters require explicit dual-annotation in the IR. The `dt` parameter is the most consequential boundary case: wrong dt produces LINCS failures that look like implementation errors but are actually methodology errors.
- The most common GROMACS failures are inherently ambiguous in the LFI's three-way classification. The IR cannot resolve this ambiguity from the error message alone — it must cross-reference parameters, energy trajectories, topology, and structural context. This means the IR must be a multi-source correlation engine, not just a log parser.
- grompp's validation gap (catches syntax, misses physics) maps cleanly to the LFI's Stage 1 vs. Stage 3 distinction. If grompp accepted the simulation, Stage 1 (implementation audit) can assume the specification is syntactically valid and focus on runtime execution errors. Stage 3 (theory evaluation) must handle everything grompp cannot check.

**Open Threads:**

- How do OpenMM and VASP compare? OpenMM's Python API may provide richer programmatic access but weaker theory-implementation separation. VASP's INCAR/POSCAR/POTCAR system may have different boundary parameter characteristics. These comparisons are needed to identify IR elements that generalize vs. those that are GROMACS-specific.
- Can panedr's DataFrame output serve as a direct input to the IR, or does the IR need a more abstract energy representation that works across frameworks?
- The error pattern library approach (regex matching on known GROMACS error messages) is brittle across GROMACS versions. Is there a more robust approach? GROMACS source code analysis could provide a definitive catalog of error messages.
- The crash-state data gap (no state dump at exact crash point) limits forensic analysis. Is this a fundamental limitation or can GROMACS be configured to dump state on crash?
- How does the auto-tuning behavior (nstlist, rlist, PME parameters) interact with reproducibility? If two runs of the same .tpr produce different auto-tuned parameters, the IR must track this divergence.

## Accumulated Findings

### What We Know

- GROMACS produces 8 primary output file types with well-defined content boundaries. The .edr (energy time series) is binary-structured and accessible via the panedr Python library without a GROMACS installation, making it the most IR-friendly data source. The .log is semi-structured free text with no machine-readable error taxonomy. The .tpr bundles theory and implementation layers into a single opaque binary, recoverable via `gmx dump`. [Log: 2026-02-20, GROMACS Trace Format Characterization]

- The GROMACS .mdp parameter interface provides a relatively clean theory-implementation separation. Theory-layer parameters (integrator, tcoupl, pcoupl, coulombtype, force field choice) are declarative and side-effect-free. Implementation-layer parameters (nstlog, nstenergy, nstxout) control execution mechanics only. mdrun command-line parameters (-ntomp, -gpu_id, -dd) are purely implementation-layer. However, 10+ "boundary" parameters (dt, fourierspacing, nstlist, lincs-order, pbc, verlet-buffer-tolerance) serve dual theory-implementation roles and require explicit dual-annotation in the IR. [Log: 2026-02-20, GROMACS Trace Format Characterization]

- GROMACS error messages are free-text strings with no error codes, no severity taxonomy, and no machine-readable classification. The most common simulation failures (constraint violations: LINCS, SETTLE, SHAKE) are inherently ambiguous between theory, methodology, and implementation causes. Correct classification of these failures requires cross-file correlation of .log events, .edr energy trajectories, .tpr parameters, and structural context. No single output file contains sufficient information for fault classification. [Log: 2026-02-20, GROMACS Trace Format Characterization]

- grompp (the GROMACS preprocessor) validates syntactic/structural correctness comprehensively but does not validate physical/scientific correctness. This creates a clean audit hierarchy: grompp handles Stage 1 syntax validation, runtime monitoring detects numerical instability, and post-run analysis evaluates physical correctness. If grompp accepted a simulation, the implementation-layer specification is known to be syntactically valid. [Log: 2026-02-20, GROMACS Trace Format Characterization]

### What We Suspect

- The IR will need to be a multi-source correlation engine rather than a single-file parser. The LINCS failure walkthrough demonstrates that fault classification requires merging temporal event sequences (.log warnings), quantitative state trajectories (.edr energy series), parameter context (.tpr specification), and structural data (atom-level topology). This suggests the IR must represent "failure narratives" as composite objects linking data from multiple sources, not as flat event logs. [Log: 2026-02-20, GROMACS Trace Format Characterization, Section 6.7]

- The `dt` (timestep) parameter may be the single most diagnostic boundary parameter for GROMACS fault classification. Wrong dt is the most common cause of LINCS failures, which are the most common GROMACS crash type, and wrong dt produces symptoms (constraint violation, energy explosion) that appear to be implementation failures but are actually methodology errors. If the IR can correctly classify dt-related failures, it will handle a large fraction of real-world GROMACS crashes. [Log: 2026-02-20, GROMACS Trace Format Characterization, Sections 2.1.3 and 5.2]

- An error pattern library approach (regex matching against known GROMACS error message strings) may be sufficient for initial IR prototyping but is likely too brittle for production use, since GROMACS error message text can change between versions. A more robust approach may involve mapping error messages to semantic categories based on the GROMACS source code's error-reporting call sites. [Log: 2026-02-20, GROMACS Trace Format Characterization, Section 5.1]

### What We Don't Know

- Whether the GROMACS trace format is representative of other DSL frameworks. OpenMM (Python API, different output formats) and VASP (INCAR/POSCAR, different error reporting) may have fundamentally different trace characteristics that require different IR design patterns. The IR's generalizability across frameworks is untested. [Log: 2026-02-20, GROMACS Trace Format Characterization, Open Threads]

- Whether panedr's DataFrame representation of .edr data can serve as a direct IR input or whether a more abstract energy representation is needed for cross-framework compatibility. [Log: 2026-02-20, GROMACS Trace Format Characterization, Open Threads]

- Whether GROMACS can be configured to produce a complete state dump at crash time (coordinates, velocities, forces at the exact crash step). The default behavior preserves only the last periodic checkpoint, which may be thousands of steps before the crash. If crash-state data is unavailable, the IR must work with incomplete temporal data near failure points. [Log: 2026-02-20, GROMACS Trace Format Characterization, Section 6.5]

- What the minimum set of IR features is that enables fault classification at a rate significantly exceeding the 21% baseline. The GROMACS analysis identifies six candidate IR requirements, but it is unknown which are strictly necessary vs. nice-to-have for exceeding the baseline. [Log: 2026-02-20, GROMACS Trace Format Characterization, Section 8.3]

- How GROMACS runtime auto-tuning (auto-adjusted nstlist, rlist, PME parameters) affects trace reproducibility and whether the IR must account for divergent auto-tuning between nominally identical runs. [Log: 2026-02-20, GROMACS Trace Format Characterization, Open Threads]

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
