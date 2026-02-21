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

IN PROGRESS — DSL trace format survey (Step 1) underway. VASP survey complete. OpenMM and GROMACS surveys pending.

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

### Entry 001 — 2026-02-20: VASP Trace Output System Survey

**Scope:** Complete survey of VASP's output file system, theory-implementation boundary analysis, vasprun.xml structure, failure signaling, and closed-source constraints. Part of Next Step 1 (Survey DSL trace formats).

**Method:** Systematic analysis of VASP's documented output system based on VASP Wiki documentation, pymatgen/ASE API documentation, and domain knowledge of DFT workflows. Produced a structured analysis document (`dsl-evaluation/vasp-trace-analysis.md`) covering seven sections: output file inventory, theory-implementation boundary, vasprun.xml structure, output file comparison, failure signaling, DFT-specific theory-implementation mapping, and closed-source constraints. Each claim tagged with evidence basis ([documented], [observed], [inferred]).

**Findings:**

1. **VASP's output system is well-structured for theory-layer reconstruction.** vasprun.xml provides a comprehensive XML record of all input parameters (with resolved defaults), SCF convergence trajectories, ionic step results (energy, forces, stress), eigenvalues, and DOS. Combined with OUTCAR (implementation diagnostics, warnings, timing) and stdout/stderr (crash information), this forms a sufficient trace for most fault isolation tasks.

2. **The theory-implementation boundary exists but is not API-declared.** VASP's INCAR file mixes theory parameters (GGA, ENCUT, ISMEAR) and implementation parameters (NCORE, KPAR, ALGO) in a single flat namespace. Unlike MD codes where force fields are external data files, VASP's "force field" (the exchange-correlation functional) is selected by an INCAR tag. ATHENA must maintain an external classification table for INCAR tags — a finite engineering task (~200-300 tags total, ~50-80 commonly used).

3. **Theory is distributed across four input files.** INCAR specifies the functional and convergence criteria. POSCAR defines the crystal structure. POTCAR provides pseudopotentials (electron-ion interaction approximation). KPOINTS specifies Brillouin zone sampling. All four carry theory content. The IR must capture and fuse all four into a unified specification representation.

4. **Ambiguous parameters create genuine cross-layer coupling.** PREC simultaneously configures physical accuracy and resource allocation. LREAL trades accuracy for speed. ALGO can affect which SCF minimum is found in pathological cases. These parameters cannot be cleanly assigned to theory or implementation and require special handling in the IR.

5. **The most dangerous VASP failures are silent.** Insufficient ENCUT, inadequate k-points, inappropriate functional choice, and wrong pseudopotential selection all produce results without any error, warning, or non-zero exit code. VASP does not signal SCF non-convergence via exit code. The Trace Semantics Engine must implement domain-aware validation rules beyond what VASP reports.

6. **Closed-source constraints are manageable but impose a ceiling.** ATHENA cannot instrument VASP internals. Observable output (vasprun.xml + OUTCAR + stdout) is sufficient for standard calculations. The ceiling is hit for subtle numerical issues (FFT aliasing, PAW reconstruction errors, non-deterministic MPI reductions) that are invisible in output.

7. **Community tooling (pymatgen, custodian, ASE) provides mature parsing infrastructure.** pymatgen's custodian package is particularly relevant — it implements automated error detection and correction for VASP, functioning as a community-built implementation audit tool.

8. **VASP's input is purely declarative.** Unlike OpenMM (which requires Python scripting), VASP's four input files contain no imperative code. This makes VASP's input more amenable to static analysis and specification reconstruction than scripting-based DSLs.

**Implications:**

- The IR must support multi-file trace composition (fusing vasprun.xml + OUTCAR + stdout into one semantic representation). This is a structural requirement not present in single-log systems.
- The IR must support convergence trajectory representation (SCF and ionic convergence as sequences, not just final values). Trajectory shape carries diagnostic information.
- Silent failure detection requires a rule-based validation layer within the Trace Semantics Engine, implementing domain-aware checks that VASP itself does not perform. This layer needs access to the Causal Graph Manager for system-type-dependent rules (e.g., metals need denser k-meshes than insulators).
- The IR needs DSL-specific adapters rather than a universal schema — VASP's multi-file declarative input differs structurally from OpenMM's Python scripting interface and GROMACS's MDP/topology format.
- VASP should remain in ATHENA's target set, but ATHENA should also support at least one open-source DFT code for cross-validation and deeper instrumentation.

**Open Threads:**

- How does VASP's output compare quantitatively to OpenMM and GROMACS in terms of trace completeness? Need to complete those surveys for comparison.
- What fraction of real-world VASP failures fall into the "silent" category vs. self-announcing crashes? Materials Project workflow data (custodian error logs) might provide statistics.
- Can custodian's error handler catalog serve as a starting point for the rule-based validation layer?
- VASP 6 introduced the REPORT file with more detailed logging. How much does this close the gap in implementation-layer observability?

## Accumulated Findings

### What We Know

- VASP's vasprun.xml provides a comprehensive, structured XML record of all input parameters (with resolved defaults), SCF convergence trajectories, ionic relaxation trajectories (energy, forces, stress per step), eigenvalues, and DOS. It is approximately 80-90% complete for theory-layer trace reconstruction. (Entry 001; VASP Wiki documentation)
- VASP's theory-implementation boundary is not API-declared. Theory parameters (GGA, ENCUT, ISMEAR) and implementation parameters (NCORE, KPAR) coexist in a flat INCAR namespace without structural separation. (Entry 001; VASP Wiki documentation)
- Theory specification in VASP is distributed across four input files: INCAR, POSCAR, POTCAR, KPOINTS. All four carry theory content. (Entry 001; VASP Wiki documentation)
- VASP does not signal SCF non-convergence, ionic non-convergence, insufficient ENCUT, or inadequate k-point sampling via non-zero exit codes. These must be inferred from output file content. (Entry 001; observed behavior documented in community sources and custodian error handlers)
- Ambiguous parameters (PREC, LREAL, ALGO) create genuine cross-layer coupling that cannot be cleanly resolved to theory or implementation. (Entry 001; VASP Wiki documentation)
- VASP's input is purely declarative (no scripting), making it more amenable to static analysis than scripting-based DSLs like OpenMM. (Entry 001; VASP Wiki documentation)
- pymatgen, ASE, and custodian provide mature parsing and error-handling infrastructure for VASP output. Custodian's error handler catalog is effectively a community-built implementation audit tool. (Entry 001; pymatgen/custodian documentation)
- The minimum "complete trace" for ATHENA's three-stage audit is vasprun.xml + OUTCAR + stdout/stderr. OSZICAR is redundant given the first two. (Entry 001; inferred from content analysis)

### What We Suspect

- The IR will need DSL-specific adapters rather than a universal schema, because VASP's multi-file declarative input differs structurally from OpenMM's Python scripting and GROMACS's MDP/topology format. (Entry 001; inferred from VASP analysis, pending OpenMM/GROMACS surveys for confirmation)
- Silent theory failures (insufficient ENCUT, inadequate k-points, inappropriate functional) may constitute a significant fraction of real VASP failures in practice, making domain-aware validation rules essential for the Trace Semantics Engine. (Entry 001; inferred from VASP's lack of explicit signaling)
- VASP's closed-source constraints are manageable for standard DFT calculations but may impose a meaningful ceiling for edge cases (heavy-element SOC, strongly correlated systems, metastable magnetic states). (Entry 001; inferred from analysis of failure modes vs. VASP reporting)
- Custodian's error handler catalog could serve as a foundation for the rule-based validation layer in the Trace Semantics Engine. (Entry 001; inferred from custodian's design purpose)

### What We Don't Know

- How VASP's trace completeness compares quantitatively to OpenMM and GROMACS. (Pending: OpenMM and GROMACS surveys)
- What fraction of real-world VASP failures fall into the "silent" category vs. self-announcing crashes. Materials Project workflow logs might provide statistics. (No data yet)
- Whether the VASP 6 REPORT file significantly closes the implementation-layer observability gap. (Not investigated)
- How the IR should represent convergence trajectories (SCF energy sequences, force sequences) — as raw time series, as classified patterns (oscillating, monotone, plateaued), or as derived features. (Pending: IR schema design in Step 5)
- Whether a single IR schema can accommodate both DFT codes (VASP) and MD codes (OpenMM, GROMACS) or whether the structural differences require fundamentally different IR designs with a common interface. (Pending: completion of all three DSL surveys)

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
