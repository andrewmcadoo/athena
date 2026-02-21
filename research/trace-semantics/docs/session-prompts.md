# Session Prompts for P1: Trace Semantics IR

7 unblocked beads, each designed for an independent Claude session.

---

## athena-jv4 — Step 1a: OpenMM Trace Format Characterization

```
TASK:
Research and document OpenMM's trace output system. Produce a structured analysis of what OpenMM generates during simulation execution, specifically mapping each output element to either the theory layer (what the scientist specifies) or the implementation layer (how the framework executes it).

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Contains the research question, key definitions, and architecture references. You will append a log entry here.
- research/trace-semantics/docs/research-plan.md — Full investigation plan for context on how this step fits.
- ARCHITECTURE.md §4.5 (Trace Semantics Engine), §5.3 (Fault Isolation Decision Tree) — What the IR must ultimately support.
- VISION.md §4.1 (LFI differentiator), Open Question #1 — The "Semantic Language of Failure" problem.

DELIVERABLES:
1. Create `research/trace-semantics/dsl-evaluation/openmm-trace-analysis.md` with:

   ### Reporter System Inventory
   Complete catalog of OpenMM reporter types (StateDataReporter, DCDReporter, PDBReporter, PDBxReporter, CheckpointReporter, XTCReporter, etc.). For each: what data it emits, format, configurability, what layer it belongs to.

   ### Theory-Implementation API Boundary
   Map OpenMM's architecture to the theory-implementation distinction:
   - Theory layer: ForceField, Topology, force parameters, equations of motion choices
   - Implementation layer: Integrator, Context, Platform, precision mode, parallelization
   - The boundary: `ForceField.createSystem()` — document exactly what this compiles and what's lost/transformed
   - Assessment: Is this boundary clean enough for deterministic auditing?

   ### Exception and Error Exposure
   How does OpenMM surface failures? Python exceptions? Platform-level errors? Does the framework distinguish theory errors (bad force field parameters) from implementation errors (NaN from precision)? Catalog specific exception types.

   ### Execution Metadata
   What metadata about the execution environment is accessible? Platform type, precision mode, device properties, parallelization strategy, memory usage.

   ### Custom Reporter Extensibility
   Can OpenMM's reporter system be extended to capture additional trace data? What is the reporter API contract? What hooks exist for custom instrumentation?

   ### Failure Walkthrough: NaN Energy
   Walk through a concrete NaN energy failure scenario. Document every piece of information available to an external observer: what reporters emit, what exceptions are thrown, what state is recoverable, what is lost.

   ### Failure Mode Taxonomy
   Classify known OpenMM failure modes as: implementation (NaN/precision, memory, platform), methodology (wrong sampling, insufficient equilibration), theory (wrong force field, bad parameters), ambiguous.

2. Append an investigation log entry to `research/trace-semantics/FINDINGS.md` under "## Investigation Log" (replacing "*No entries yet.*"). Format:

   ### 2026-02-20: OpenMM Trace Format Characterization
   **Scope:** [What was investigated]
   **Method:** [How — documentation review, source code analysis, GitHub issues, web research]
   **Findings:** [Key discoveries, with citations]
   **Implications:** [What this means for IR design]
   **Open Threads:** [Unresolved questions]

3. Begin populating FINDINGS.md "Accumulated Findings" sections with evidence-backed claims from this investigation.

RESEARCH SOURCES:
- OpenMM documentation: docs.openmm.org (Reporter classes, Simulation, ForceField, Platform, Context APIs)
- OpenMM GitHub: github.com/openmm/openmm (source for statedatareporter.py, simulation.py, integrator implementations, platform backends)
- OpenMM GitHub Issues: Search for NaN, energy explosion, precision, constraint failure — these reveal real failure modes and what trace data is available
- OpenMM User Guide: sections on output, reporters, platforms, precision
- mdtraj, OpenMMTools — third-party tools that extend OpenMM's trace capabilities

CONSTRAINTS:
- Do not write production code. Analysis documents only.
- Do not modify any files outside research/trace-semantics/.
- Do not edit prior FINDINGS.md entries — append only (reverse chronological).
- Cite evidence for every claim. No unsupported assertions.
- Do not use grant-proposal rhetoric ("groundbreaking", "revolutionary").

CONTEXT:
- ATHENA is a falsification-driven AI co-scientist. The Trace Semantics Engine translates raw DSL traces into an IR for the Lakatosian Fault Isolator's three-stage audit: (1) implementation audit, (2) methodological audit, (3) theoretical evaluation. The outside-in ordering is deliberate — check outermost layer first to avoid penalizing valid theories for broken scripts.
- "Theory-implementation separation" means the API-enforced distinction in DSL frameworks between what the user specifies (theory: force field, equations, parameters) and how the framework executes it (implementation: numerical integration, memory management, hardware utilization).
- The 21% Top@1 baseline refers to general RCA accuracy on unstructured traces. DSL structure should improve this. Understanding WHAT structure OpenMM provides is the goal of this investigation.
- This is Priority 1 research for the entire project. The IR design blocks everything downstream.

VERIFICATION:
- openmm-trace-analysis.md exists and has all 7 sections filled with concrete, cited information
- FINDINGS.md has a new log entry at the top of the Investigation Log section
- At least 3 Accumulated Findings claims are populated with evidence references
- The theory-implementation boundary is documented with specific API references, not generalities
- At least 5 distinct failure modes are classified in the taxonomy
```

---

## athena-7pi — Step 1b: GROMACS Trace Format Characterization

```
TASK:
Research and document GROMACS's trace output system. Produce a structured analysis of what GROMACS generates during MD simulation execution, mapping each output element to either the theory layer or implementation layer.

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Contains the research question and key definitions.
- research/trace-semantics/docs/research-plan.md — Full investigation plan.
- ARCHITECTURE.md §4.5, §5.3 — Trace Semantics Engine and Fault Isolation Decision Tree.
- VISION.md §4.1, Open Question #1 — LFI and "Semantic Language of Failure".

DELIVERABLES:
1. Create `research/trace-semantics/dsl-evaluation/gromacs-trace-analysis.md` with:

   ### Output File Inventory
   Complete catalog of GROMACS output files: `.log`, `.edr` (energy), `.trr` (full trajectory), `.xtc` (compressed trajectory), `.xvg` (analysis), `.cpt` (checkpoint), `.tpr` (run input), `.gro` (structure). For each: content structure, format (text/binary/structured), programmatic access method, theory vs. implementation layer.

   ### Theory-Implementation API Boundary
   Map GROMACS's architecture to the theory-implementation distinction:
   - Theory layer: `.mdp` parameters that specify physics (integrator type, tcoupl, pcoupl, coulombtype, vdwtype, force field choice)
   - Implementation layer: `.mdp` parameters that specify execution (nstlog, nstenergy, nstxout, ncore, ntomp, gpu_id)
   - The boundary: `grompp` — the preprocessing step that compiles `.mdp` + `.top` + `.gro` into binary `.tpr`. Document what validation grompp performs and what it catches vs. misses.
   - Assessment: Is this boundary clean enough for deterministic auditing? Are there `.mdp` parameters that blur the line?

   ### Log File Structure
   Analyze the `.log` file format: Is it structured or free-text? What sections does it contain? What is machine-parseable vs. human-readable-only? How does verbosity affect content?

   ### Energy File (.edr) Access
   Document the `.edr` binary format: what data it contains, how to access programmatically (via `gmx energy`, `panedr` Python library, or `xdrlib`). What time series are available? Resolution? Completeness?

   ### Error and Warning Taxonomy
   How does GROMACS report errors vs. warnings vs. notes? Catalog specific error types:
   - Constraint algorithm failures (LINCS, SETTLE, SHAKE)
   - "Atoms too far" / domain decomposition errors
   - NaN generation
   - Software inconsistency errors
   - Memory / resource errors
   Classify each as: implementation, methodology, theory, ambiguous.

   ### Failure Walkthrough: LINCS Constraint Failure
   Walk through a LINCS constraint failure scenario. Document: what appears in the .log file, what's in .edr, what exit code is returned, what state files are produced, what an external observer can reconstruct about the cause.

   ### Preprocessing Validation (grompp)
   What does `grompp` check before simulation starts? What errors can it catch? What slips through? This is a pre-execution verification step — document its coverage and gaps.

2. Append an investigation log entry to `research/trace-semantics/FINDINGS.md`:

   ### 2026-02-20: GROMACS Trace Format Characterization
   **Scope:** **Method:** **Findings:** **Implications:** **Open Threads:**

3. Update Accumulated Findings sections with evidence-backed claims.

RESEARCH SOURCES:
- GROMACS manual: manual.gromacs.org (file formats reference, mdrun docs, mdp options, run-time errors)
- GROMACS GitHub: github.com/gromacs/gromacs (source for mdrun output, log formatting, error handling)
- panedr library: github.com/panedr/panedr (Python access to .edr files)
- GROMACS user guide: common errors section, getting started tutorials
- GROMACS forums/mailing lists: real failure case discussions

CONSTRAINTS:
- Do not write production code. Analysis documents only.
- Do not modify files outside research/trace-semantics/.
- Append-only to FINDINGS.md Investigation Log (reverse chronological, new entries at top).
- Cite evidence for every claim.
- Do not use grant-proposal rhetoric.

CONTEXT:
- ATHENA is a falsification-driven AI co-scientist. The Trace Semantics Engine translates raw DSL traces into an IR for the Lakatosian Fault Isolator's three-stage audit: (1) implementation, (2) methodology, (3) theory.
- "Theory-implementation separation" = the API-enforced distinction between what the user specifies (physics/equations/parameters) and how the framework executes it (numerics/memory/hardware).
- GROMACS is the most widely used MD engine. Its `.mdp` file system provides a relatively clean theory-implementation boundary, but some parameters blur the line (e.g., timestep `dt` is both a theory choice and numerical stability concern). Document these ambiguities carefully.
- The `.tpr` binary compiled by grompp is analogous to a compiled executable — it bundles theory and implementation into a single opaque object. Understanding what's recoverable from `.tpr` at runtime matters for trace completeness.
```

---

## athena-xir — Step 1c: VASP Trace Format Characterization

```
TASK:
Research and document VASP's trace output system. Produce a structured analysis of what VASP generates during DFT calculations, mapping each output element to either the theory layer or implementation layer. Note: VASP is a DFT code, not molecular dynamics — the "theory" here is quantum mechanical (exchange-correlation functional, basis set, k-point sampling).

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Research question and key definitions.
- research/trace-semantics/docs/research-plan.md — Full investigation plan.
- ARCHITECTURE.md §4.5, §5.3 — Trace Semantics Engine and Fault Isolation Decision Tree.
- VISION.md §4.1, Open Question #1 — LFI and "Semantic Language of Failure".

DELIVERABLES:
1. Create `research/trace-semantics/dsl-evaluation/vasp-trace-analysis.md` with:

   ### Output File Inventory
   Complete catalog of VASP output files: OUTCAR, OSZICAR, vasprun.xml, CONTCAR, CHGCAR, DOSCAR, EIGENVAL, PROCAR, WAVECAR, IBZKPT, etc. For each: content, format (text/XML/binary), size characteristics, programmatic access (pymatgen, ASE, vasprun-xml), theory vs. implementation layer.

   ### Theory-Implementation API Boundary
   Map VASP's architecture to the theory-implementation distinction:
   - Theory layer: INCAR tags specifying physics (ENCUT, ISMEAR, SIGMA, EDIFF, EDIFFG, GGA/METAGGA, LHFCALC, IBRION, ISIF, NSW, LDAU parameters)
   - Implementation layer: INCAR tags specifying execution (NCORE, NPAR, KPAR, LPLANE, LWAVE, LCHARG, ALGO, NELM)
   - Ambiguous parameters: Some INCAR tags affect both physics and performance (PREC, LREAL, ADDGRID). Document these carefully.
   - Assessment: Is VASP's theory-implementation boundary clean enough? How does it compare to MD codes?

   ### vasprun.xml Structure
   Analyze the XML hierarchy: what nodes exist, what data each contains, how theory-layer and implementation-layer elements are intermixed or separated. This is the primary structured output — assess its completeness for IR purposes.

   ### OUTCAR vs. OSZICAR vs. vasprun.xml
   Compare these three primary outputs: what each contains, overlap, unique information, parsability. Which combination constitutes a "complete trace"?

   ### Failure Signaling
   How does VASP signal failure?
   - SCF non-convergence (electronic): what appears in OSZICAR, OUTCAR, vasprun.xml, exit code?
   - Ionic relaxation failure: unconverged forces, exceeded NSW
   - Memory errors, segfaults
   - Basis set issues (ENCUT too low)
   - k-point sampling inadequacy
   Classify each as: implementation, methodology, theory, ambiguous.

   ### DFT-Specific Theory-Implementation Distinction
   VASP differs from MD codes. The "theory" is quantum mechanical:
   - Exchange-correlation functional (LDA, GGA, hybrid, meta-GGA) = theory
   - SCF convergence algorithm (ALGO=Normal/Fast/VeryFast) = implementation
   - Plane-wave cutoff (ENCUT) = theory choice with implementation consequences
   - Document how DFT-specific concepts map to ATHENA's three-stage audit

   ### Closed-Source Constraints
   VASP is proprietary (unlike OpenMM/GROMACS). How does this affect:
   - Ability to instrument trace capture
   - Completeness of trace data (internal state not accessible)
   - Reliance on output files only vs. runtime hooks
   - Comparison with open-source frameworks for IR design

2. Append investigation log entry to `research/trace-semantics/FINDINGS.md`.
3. Update Accumulated Findings sections.

RESEARCH SOURCES:
- VASP Wiki: vasp.at/wiki/ (output files, INCAR tags, terminal output, tutorials)
- pymatgen VASP I/O: pymatgen.org/pymatgen.io.vasp.html (Vasprun parser, Outcar parser)
- ASE VASP calculator: wiki.fysik.dtu.dk/ase/ase/calculators/vasp.html
- VASP forum / discussion boards: common errors and failure patterns
- Materials Project documentation: materialsproject.org (VASP workflow patterns)

CONSTRAINTS:
- Analysis documents only, no production code.
- Do not modify files outside research/trace-semantics/.
- Append-only to FINDINGS.md (reverse chronological).
- Cite evidence for every claim.
- Be explicit about information limitations due to VASP's closed-source nature — distinguish "documented behavior" from "observed behavior" from "inferred behavior."

CONTEXT:
- ATHENA targets DSL environments with API-separated theory and implementation layers. VASP is the materials science representative alongside OpenMM (molecular dynamics) and GROMACS (molecular dynamics).
- VASP uses DFT (Density Functional Theory), which is fundamentally different from classical MD. The "theory" is the choice of exchange-correlation functional and associated approximations, not force field parameters. This affects how theory-layer failures manifest.
- VASP's INCAR file is the primary "specification" input, analogous to GROMACS's .mdp file. But VASP also requires POSCAR (structure), POTCAR (pseudopotentials), and KPOINTS — the theory-implementation boundary is distributed across multiple input files.
- Closed-source nature is a significant constraint. If VASP's trace output is insufficient for the IR, ATHENA may need to narrow its target DSL set or rely more heavily on wrapper-based instrumentation.
```

---

## athena-psc — Step 2a: RCA and Formal Verification IR Survey

```
TASK:
Survey intermediate representations used in root cause analysis (RCA) tools and formal verification systems. Identify IR design patterns that could transfer to ATHENA's trace semantics problem: translating raw DSL trace logs into a structured representation suitable for deterministic three-stage fault classification.

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Research question and key definitions.
- research/trace-semantics/docs/research-plan.md — Full investigation plan, especially the IR requirements.
- ARCHITECTURE.md §4.5 (Trace Semantics Engine), §5.3 (three-stage LFI audit) — What the IR must support.

DELIVERABLES:
1. Create `research/trace-semantics/dsl-evaluation/rca-formal-verification-ir-survey.md` with:

   ### RCA Intermediate Representations
   Survey how RCA systems represent causal chains from root cause to observable symptom:

   **LLM-based RCA:**
   - "Exploring LLM-based Agents for Root Cause Analysis" (arxiv:2403.04123) — How do LLM agents structure their analysis? What implicit IR do they use (chain-of-thought, event logs, dependency graphs)?
   - "Stalled, Biased, and Confused: Uncovering Reasoning Failures in LLMs for Cloud RCA" (arxiv:2601.22208) — What reasoning failures occur? What IR deficiencies cause them? What structural improvements are suggested?
   - "Empowering Practical Root Cause Analysis" (if findable) — May be the source of the 21% Top@1 figure. How does it structure trace analysis?

   **Structured RCA:**
   - "Chain-of-Event: Interpretable Root Cause Analysis for Microservices" (FSE 2024) — Event chain IR. How are events typed, linked, and queried? What accuracy improvement over unstructured approaches?
   - Microservice distributed tracing (Jaeger, Zipkin) — Span-based trace models. How are causal dependencies encoded?

   For each: data structures used, how causal ordering is represented, how root cause candidates are ranked, queryability.

   ### Formal Verification IRs
   Survey how verification systems encode specification-vs-implementation:

   **Compiler/Program Analysis IRs:**
   - LLVM IR (SSA form) — How does static single assignment represent data flow? How are error paths represented? Relevance to failure trace analysis.
   - MLIR (Multi-Level IR) — Dialect system allowing multiple levels of abstraction in one IR. How could this map to ATHENA's theory-implementation layering?

   **Verification-Specific IRs:**
   - Boogie (from Dafny/VCC) — Intermediate verification language. How does it separate specification from implementation? How are assertions structured?
   - Why3 (from Frama-C/SPARK) — Logical specification language. How does it encode pre/postconditions and invariants?

   **Proof/Counter-Example Traces:**
   - DRAT proofs (SAT solver resolution traces) — Machine-checkable proof of unsatisfiability. How is a refutation structured?
   - AIGER (hardware model checking) — Counter-example traces witnessing specification violations. How is the trace that leads to violation represented?

   For each: how the specification-implementation distinction is encoded, what query patterns are supported, what verification guarantees are provided.

   ### Program Analysis Error Path Representations
   - Clang Static Analyzer — How are error paths (from source to bug) represented?
   - Soot/WALA (Java) — How are data flow and control flow represented for error analysis?
   - Infer (Facebook) — How does it represent error traces in its bi-abduction framework?

   ### Transferable Patterns Catalog
   For each pattern identified, assess:
   - **Pattern name and source**
   - **Core mechanism** (1-2 sentences)
   - **Transferability to ATHENA** (high/medium/low with reasoning)
   - **Which LFI audit stage it supports** (Stage 1: implementation, Stage 2: methodology, Stage 3: theory, or cross-cutting)
   - **Limitations** for ATHENA's use case

   ### Anti-Patterns
   IR design choices from surveyed systems that would be harmful for ATHENA:
   - Post-mortem-only designs (can't support real-time querying)
   - Specification-implementation conflation (defeats the three-stage audit)
   - Others identified during survey

2. Append investigation log entry to `research/trace-semantics/FINDINGS.md`.
3. Update Accumulated Findings sections.

RESEARCH SOURCES:
- ArXiv papers: 2403.04123, 2601.22208, and related RCA papers
- Chain-of-Event FSE 2024 paper
- LLVM IR reference: llvm.org/docs/LangRef.html
- MLIR documentation: mlir.llvm.org
- Boogie language reference: github.com/boogie-org/boogie
- Why3 documentation: why3.lri.fr
- DRAT proof format: satcompetition.github.io/2024/
- Clang Static Analyzer: clang-analyzer.llvm.org
- Infer documentation: fbinfer.com

CONSTRAINTS:
- Analysis documents only, no production code.
- Do not modify files outside research/trace-semantics/.
- Append-only to FINDINGS.md (reverse chronological).
- Cite evidence for every claim. Include paper titles, URLs, and specific section references.
- Focus on transferability to ATHENA's specific problem (three-stage fault classification in DSL traces), not general IR theory.

CONTEXT:
- ATHENA's IR must support deterministic querying for three-stage fault classification: (1) implementation audit — framework exceptions, data pipeline correctness, numerical precision, resource state; (2) methodological audit — experiment measures right variables, intervention on cause not proxy, sufficient sampling, confounders controlled; (3) theoretical evaluation — results contradict predictions under clean conditions.
- The IR will be implemented in Rust for parsing throughput (ADR 001). Design patterns must be compatible with zero-copy/streaming parsing of megabyte-scale traces.
- The key question for each surveyed IR: does it cleanly separate "what was specified" from "what executed"? ATHENA's entire LFI depends on this distinction being deterministic.
```

---

## athena-k52 — Step 2b: Provenance and Scientific Workflow IR Survey

```
TASK:
Survey provenance data models and scientific workflow IRs. Assess whether W3C PROV-DM's Entity-Activity-Agent model (or extensions like ProvONE) can represent the theory-implementation distinction central to ATHENA's trace semantics problem.

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Research question and key definitions.
- research/trace-semantics/docs/research-plan.md — Full investigation plan.
- ARCHITECTURE.md §4.5, §5.3 — Trace Semantics Engine and three-stage LFI audit.

DELIVERABLES:
1. Create `research/trace-semantics/dsl-evaluation/provenance-workflow-ir-survey.md` with:

   ### W3C PROV-DM Analysis
   Deep analysis of the PROV Data Model:
   - Core types: Entity, Activity, Agent
   - Core relations: wasGeneratedBy, used, wasAttributedTo, wasDerivedFrom, wasAssociatedWith, actedOnBehalfOf
   - **Mapping to ATHENA**: Can Entity = simulation state/result? Activity = simulation step/computation? Agent = DSL framework / user specification? Does this mapping preserve the theory-implementation distinction?
   - Qualified relations (PROV-DM has these for adding metadata to relations) — do they provide enough resolution for fault classification?
   - PROV-CONSTRAINTS (validity rules) — can they encode LFI audit preconditions?
   - Assessment: strengths and limitations of PROV-DM as an IR foundation for ATHENA.

   ### ProvONE (Scientific Workflow Extension)
   - Additional concepts: Program, Port, Channel, Controller, Workflow
   - How do these map to DSL simulation concepts? Program = simulation code? Workflow = experiment specification?
   - Does ProvONE's extension address gaps in base PROV-DM for scientific computing?
   - Assessment: does ProvONE get closer to ATHENA's needs?

   ### Provenance Query Languages
   - SPARQL over PROV (PROV is RDF-compatible) — query expressiveness for causal reasoning
   - ProvQL or other specialized provenance query languages
   - Key question: Can queries like "was this output causally dependent on implementation parameter X or theory parameter Y?" be expressed? With what complexity?
   - Performance characteristics for megabyte-scale provenance graphs

   ### Scientific Workflow Provenance Systems
   Survey existing systems that capture provenance from scientific workflows:
   - Kepler, Taverna, VisTrails — how they represent workflow execution provenance
   - Galaxy (bioinformatics) — provenance tracking in data analysis pipelines
   - Common Workflow Language (CWL) — provenance specification
   - What do these systems capture that's relevant to fault analysis? What do they miss?

   ### Process Mining on Workflow Logs
   - Techniques for extracting process models from execution logs
   - Event log standards (XES - eXtensible Event Stream)
   - Conformance checking: comparing expected process model to actual execution — directly relevant to ATHENA's divergence detection
   - Discovery algorithms: can they infer causal structure from traces?

   ### Scalability Assessment
   - PROV graphs for megabyte-scale traces: how many entities/activities/relations?
   - Storage and query performance characteristics
   - Streaming/incremental construction feasibility (important for Rust implementation)
   - Comparison with flat event log approaches

   ### Expected vs. Actual Outcome Representation
   Critical for LFI Stage 3 (Theoretical Evaluation):
   - How do provenance models represent predictions vs. observations?
   - PROV-DM has "Plans" (intended behavior) — can these encode hypothesis predictions?
   - Is there a standard pattern for representing expected-vs-actual divergence?

   ### Transferable Patterns Catalog
   For each pattern: name, source, core mechanism, transferability assessment, which LFI stage it supports, limitations.

   ### Anti-Patterns
   Provenance design choices that would be harmful for ATHENA.

2. Append investigation log entry to `research/trace-semantics/FINDINGS.md`.
3. Update Accumulated Findings sections.

RESEARCH SOURCES:
- W3C PROV-DM: w3.org/TR/prov-dm/ (the specification)
- W3C PROV-O: w3.org/TR/prov-o/ (OWL ontology)
- W3C PROV-CONSTRAINTS: w3.org/TR/prov-constraints/
- ProvONE: purl.dataone.org/provone-v1-dev (or search for "ProvONE dataone")
- Scientific workflow provenance literature (search: "scientific workflow provenance", "computational reproducibility provenance")
- XES event log standard: xes-standard.org
- Process mining: processmining.org, PM4Py documentation
- FAIR provenance: go-fair.org

CONSTRAINTS:
- Analysis documents only.
- Do not modify files outside research/trace-semantics/.
- Append-only to FINDINGS.md (reverse chronological).
- Cite evidence for every claim with specific W3C section references or paper citations.
- The key assessment question is: "Can this represent the theory-implementation distinction deterministically?" Every surveyed model must be evaluated against this question.

CONTEXT:
- ATHENA's LFI performs a three-stage audit. The IR must enable Stage 1 (implementation check), Stage 2 (methodology check), Stage 3 (theory evaluation). The theory-implementation distinction must be structurally encoded, not inferred.
- PROV-DM is a strong candidate foundation because it already separates "what was done" (Activities) from "what resulted" (Entities) from "who did it" (Agents). The question is whether this maps cleanly to ATHENA's theory-implementation-methodology layering.
- The IR will eventually be implemented in Rust for parsing throughput (ADR 001). Graph-based representations (like PROV's RDF triples) may have performance implications compared to flat event logs.
- DECISION GATE 2 depends on this investigation: if PROV-DM fits well, it becomes the IR foundation. If not, a novel design is needed (higher risk). Be honest in the assessment.
```

---

## athena-661 — Step 3a: Map LFI Audit to Minimum IR Requirements

```
TASK:
Derive the minimum set of semantic distinctions the IR must represent by working backwards from the Lakatosian Fault Isolator's three-stage audit (ARCHITECTURE.md §5.3). For each audit stage, specify exactly what the IR must be queryable for, producing a numbered requirements specification.

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Research question and key definitions.
- research/trace-semantics/docs/research-plan.md — Full plan, especially Session 4 section.
- ARCHITECTURE.md — Read thoroughly. Focus on:
  - §4.5 (Trace Semantics Engine) — component definition
  - §5.1-5.4 (Information Flow) — how traces flow through the system
  - §5.3 (Fault Isolation Decision Tree) — the three-stage audit in detail
  - §8.1 (Per-Component Risks) — what happens when IR is insufficient
  - §8.4 (Incomplete Observability) — trace gaps
  - §8.5 (Classification Staleness) — reclassification needs
- VISION.md — §4.1 (LFI differentiator), §6 (Honest Limitations), Open Question #1
- evaluation/hidden-confounder/README.md — the litmus test the IR must ultimately support

DELIVERABLES:
1. Append an investigation log entry to `research/trace-semantics/FINDINGS.md` containing:

   ### 2026-02-20: LFI Audit → IR Requirements Mapping
   **Scope:** Backward derivation of minimum IR semantic distinctions from ARCHITECTURE.md three-stage audit.

   **Method:** Requirements analysis — for each audit stage, enumerate every deterministic question the LFI must answer, then derive what IR content enables that answer.

   **Findings:**

   #### Stage 1: Implementation Audit — IR Must Support
   [For each question the LFI asks at Stage 1, specify: the question, what IR element answers it, what data type/structure is needed, and an example from a real DSL framework]

   Minimum semantic distinctions:
   1. Execution event (timestamped, completion status)
   2. Exception event (type, location, stack equivalent)
   3. Data validation event (input name, expected vs. actual)
   4. Numerical status event (precision mode, NaN/overflow, location)
   5. Resource status (platform, memory, device state)

   #### Stage 2: Methodological Audit — IR Must Support
   [Same treatment for Stage 2]

   Minimum semantic distinctions:
   6. Observable measurement (variable name, measurement method, values)
   7. Intervention specification (parameter name, varied range, control values)
   8. Sampling metadata (sample count, distribution, statistical power)
   9. Controlled variable set (which variables held constant, how)
   10. DAG linkage (which IR elements correspond to which DAG nodes/edges)

   #### Stage 3: Theoretical Evaluation — IR Must Support
   [Same treatment for Stage 3]

   Minimum semantic distinctions:
   11. Prediction record (hypothesis ID, predicted observable, predicted value/distribution)
   12. Observation record (corresponding actual value/distribution)
   13. Comparison result (effect size, divergence metric, confidence interval)

   #### Cross-Cutting Requirements
   14. Provenance chain (every IR element traceable to raw trace source)
   15. Layer tag (every element tagged as theory-layer or implementation-layer)
   16. Temporal ordering (causal sequence preservation)
   17. Queryability (efficient lookup by layer, event type, time range, variable, DAG node)

   #### Ambiguity Handling Requirements
   [From ARCHITECTURE.md 5.3 ambiguity handling: what the IR must represent when classification is indeterminate]

   #### Hidden Confounder Litmus Test Requirements
   [What additional IR properties the litmus test demands: confounder must be discoverable as methodological failure, not theoretical]

   **Implications:** [Which requirements are straightforward to extract from DSL traces (informed by DSL knowledge), which require inference, which may be fundamentally unobservable]

   **Open Threads:** [Questions that depend on Step 1 DSL survey results; requirements that may need revision]

2. Update Accumulated Findings sections with requirements-derived claims.

CONSTRAINTS:
- This is a requirements derivation exercise, not a design exercise. Derive WHAT the IR must represent, not HOW.
- Do not propose IR schemas — that's Step 5.
- Do not modify files outside research/trace-semantics/.
- Append-only to FINDINGS.md (reverse chronological).
- Every requirement must be traceable to a specific line/section in ARCHITECTURE.md or the litmus test spec.
- Number all requirements for cross-referencing in later steps.

CONTEXT:
- The LFI's three stages are sequential and outside-in: Stage 1 (implementation) must complete before Stage 2 (methodology), which must complete before Stage 3 (theory). If Stage 1 finds a fault, processing stops — the hypothesis is not penalized.
- Stage 2 quality is bounded by DAG accuracy. If the DAG misses real confounders, Stage 2 will too. The IR cannot fix this, but it must make the limitation transparent.
- The hidden confounder litmus test (evaluation/hidden-confounder/README.md) is the end-to-end validation. The IR must enable the LFI to classify the confounder as methodological, not theoretical. Read the litmus test spec carefully.
- This requirements document will be the reference for Step 3b (coverage matrix mapping requirements against actual DSL trace data) and Step 5 (candidate IR schema evaluation). Precision matters.
```

---

## athena-rl5 — Step 4: Characterize 21% RCA Baseline

```
TASK:
Trace the "21% Top@1 accuracy" figure cited in VISION.md to its source. Understand what drives low general RCA accuracy on unstructured traces, and identify what structural properties of DSL environments improve it. Produce a characterized baseline that informs IR design decisions.

REFERENCES:
- research/trace-semantics/FINDINGS.md — Read first. Research question and key definitions.
- research/trace-semantics/docs/research-plan.md — Full plan, especially Session 5 section.
- VISION.md — Open Question #1 (line ~129): "root cause analysis models achieve a mere 21% Top@1 accuracy on general, unstructured execution traces, this accuracy improves substantially within constrained environments."
- ARCHITECTURE.md §8.1 — Risk: if IR is insufficiently resolved, LFI cannot distinguish failure categories.

DELIVERABLES:
1. Append an investigation log entry to `research/trace-semantics/FINDINGS.md` containing:

   ### 2026-02-20: 21% RCA Baseline Characterization
   **Scope:** Source tracing of the 21% Top@1 figure; analysis of what structural properties improve RCA accuracy.

   **Method:** Literature review of LLM-based and traditional RCA evaluation papers.

   **Findings:**

   #### Source of the 21% Figure
   - Which paper(s) report this number?
   - What dataset was used? What domain (cloud systems, scientific computing, general software)?
   - What does "Top@1" mean in this context? (correct root cause is highest-ranked candidate out of how many?)
   - What kind of traces were used? (log files, metrics, distributed traces, structured events?)
   - What models were evaluated?

   #### Why Unstructured Traces Are Hard
   Identify specific properties that make unstructured traces difficult for RCA:
   - Free-text mixing with structured data
   - Lack of causal ordering
   - Missing context (what was the system supposed to do?)
   - Ambiguous error messages
   - Irrelevant log spam (low signal-to-noise ratio)
   - No semantic layer tags (theory vs. implementation conflated)
   [Rank by estimated impact on accuracy]

   #### Structural Properties That Improve Accuracy
   From literature comparing structured vs. unstructured trace analysis:
   - Temporal ordering (events in causal sequence)
   - Event type taxonomies (typed events vs. free text)
   - Causal dependency annotations (explicit cause-effect links)
   - Severity levels (error vs. warning vs. info)
   - Schema conformance (known structure, typed fields)
   - Layer separation (specification vs. execution)
   [For each: evidence source, estimated accuracy improvement, mechanism]

   #### DSL-Specific Improvement Factors
   How DSL structure specifically helps beyond general structuring:
   - Known schema (framework defines output format)
   - API-enforced theory-implementation separation
   - Deterministic execution within valid subspace
   - Typed parameters with known valid ranges
   - Pre-execution validation (grompp, createSystem)
   [Estimate how much each contributes. Flag which are speculative vs. evidence-backed.]

   #### Residual Hard Cases
   Even with perfect structure, some failures remain hard to classify:
   - [Identify failure types that structure alone doesn't solve]
   - [What additional information or inference is needed?]
   - [How do these map to ATHENA's three audit stages?]

   #### Transferability Assessment
   DECISION GATE 3 evaluation:
   - Is the 21% figure from a domain transferable to scientific DSL traces?
   - If not, what is a reasonable accuracy expectation for structured DSL traces?
   - What does "success criteria: significantly exceeding 21%" mean quantitatively?

   **Implications:** [What this means for IR design — which structural properties the IR must preserve to capture the DSL improvement, which can be deferred]

   **Open Threads:** [If baseline is from cloud/microservice domain, what additional literature on scientific computing RCA exists?]

2. Update Accumulated Findings sections.

RESEARCH SOURCES:
- Start from VISION.md Open Question #1 citation context
- "Exploring LLM-based Agents for Root Cause Analysis" (arxiv:2403.04123)
- "Stalled, Biased, and Confused" (arxiv:2601.22208)
- "Empowering Practical Root Cause Analysis by Large Language Models" — likely source paper
- "Chain-of-Event: Interpretable Root Cause Analysis for Microservices" (FSE 2024)
- RCA benchmark papers and surveys (search: "root cause analysis benchmark", "RCA accuracy evaluation", "LLM root cause analysis")
- Scientific computing debugging/failure analysis literature (if 21% is from a different domain)

CONSTRAINTS:
- Analysis and literature review only, no code.
- Do not modify files outside research/trace-semantics/.
- Append-only to FINDINGS.md (reverse chronological).
- Cite every claimed number with its source paper, dataset, and methodology.
- Distinguish clearly between: (a) numbers from papers, (b) estimates extrapolated to DSL domain, (c) speculation. Label each.
- If the 21% figure cannot be traced to a specific paper, document what was found and what the closest available number is. Do not fabricate a source.

CONTEXT:
- The 21% figure anchors ATHENA's entire value proposition: general RCA is bad, but DSL-structured traces should be much better. If this number is wrong or from an incomparable domain, the claim needs reframing.
- "Top@1 accuracy" typically means: given a set of candidate root causes, the correct one is ranked first. The candidate set size matters — Top@1 out of 5 is very different from Top@1 out of 500.
- ATHENA's IR doesn't need to solve general RCA. It operates within DSL-constrained environments with theory-implementation separation. The question is: how much of the gap between 21% and some higher number comes from structure that the IR can provide?
- This investigation informs the feasibility assessment in Step 3b and the evaluation criteria in Step 5b. If the baseline is shaky, the success criteria need recalibration.
```

---

## Notes for Running These Sessions

1. Each prompt is self-contained — no session needs context from another.
2. All sessions write to `research/trace-semantics/FINDINGS.md`. If running truly in parallel, there will be merge conflicts on this file. Options:
   - Run each session's FINDINGS.md updates as separate sections and merge manually afterward
   - Have each session write to a temporary file (e.g., `findings-entry-jv4.md`) and consolidate later
   - Run the three Step 1 sessions sequentially (they're quick) and Steps 2a/2b/3a/4 in parallel
3. The `dsl-evaluation/` deliverables are separate files with no conflicts.
4. Steps 3a and 4 can run in parallel with Steps 1 and 2 since they don't depend on each other. But Step 3a's output will be refined in Step 3b once Step 1 results are available.
