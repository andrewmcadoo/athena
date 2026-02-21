# Requirements Coverage Matrix: R1-R29 × {OpenMM, GROMACS, VASP}

**Investigation:** Trace Semantics Engine — Requirements Refinement (Step 3b)
**Date:** 2026-02-20
**Input Documents:**
- FINDINGS.md: R1-R29 definitions (LFI audit investigation log, lines ~228-382)
- cross-framework-synthesis.md: Trace capability matrix (§1), boundary assessment (§2), failure modes (§3), completeness (§4), adapter contract (§5.3), coverage implications (§7.1)
- ir-pattern-catalog.md: Pattern coverage annotations (§1-6), candidate designs (§6)
- evaluation/hidden-confounder/README.md: R27-R29 specification context

---

## Classification Codes

| Code | Meaning | Implication for IR |
|:---|:---|:---|
| **DA** | Directly Available — framework natively outputs the data | Adapter extracts directly; low engineering cost |
| **DI** | Derivable with Instrumentation — obtainable via custom reporters/wrappers/post-processing | Adapter requires custom instrumentation code; moderate engineering cost |
| **ER** | External Rules Required — needs classification tables or domain heuristics | Adapter requires maintained domain-knowledge artifacts; ongoing maintenance cost |
| **FU** | Fundamentally Unavailable — closed source, API limitations, or architectural ceiling | Cannot be satisfied; must be represented as observability gap (R26) |
| **NT** | Not from Trace — comes from experiment spec, hypothesis, or DAG | Data source is external to the Trace Semantics Engine; IR must accept as input |
| **DE** | Derived Element — computed from other IR elements | No extraction needed; IR computes from other populated fields |

---

## 1. Stage 1: Implementation Audit (R1-R7)

Stage 1 requirements are the most tractable. DSL frameworks structurally emit execution status, exceptions, numerical health, and resource state. All three frameworks provide sufficient data for deterministic Stage 1 classification.

| Req | Description | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|:---|
| **R1** | Execution completion status | **DA** — Python exception on crash; process exit code. HIGH. [OpenMM §3.1] | **DA** — Fatal error in .log/stderr; exit code. HIGH. [GROMACS §5.1] | **DA** — Exit code for crashes (segfault, MPI). **Caveat:** exit code often 0 for SCF non-convergence; convergence failure detection deferred to R6. MEDIUM. [VASP §5.1-5.3; cross-framework §3.3] |
| **R2** | Exception/error event | **DA** — `OpenMMException` with type, message, Python traceback giving DSL call path. HIGH. [OpenMM §3.1] | **DA** — .log WARNING/NOTE/Fatal error with severity markers. Free-text, no structured error codes. HIGH for detection, MEDIUM for classification. [GROMACS §5.1; cross-framework §1.4] | **DA** — stdout/stderr crash messages, OUTCAR warnings. Free-text, no structured error codes. Exit code 0 for non-convergence means some "errors" don't produce error events. MEDIUM. [VASP §5.1-5.5; cross-framework §1.4] |
| **R3** | Input specification record | **DI** — No built-in parameter echo. Requires runtime API queries on System, Integrator, Platform objects. Force field XML source not echoed. MEDIUM. [OpenMM §2.1-2.2, §4.1; cross-framework §1.5] | **DA** — Complete .mdp parameter echo in .log including resolved defaults. Also recoverable from .tpr via `gmx dump`. HIGH. [GROMACS §3.2, §2.1; cross-framework §1.5] | **DA** — Complete INCAR echo in vasprun.xml `<incar>` and `<parameters>` including defaults. POSCAR in `<structure name="initialpos">`, POTCAR partial in `<atominfo>`, KPOINTS in `<kpoints>`. HIGH. [VASP §3.1, §3.2; cross-framework §1.5] |
| **R4** | Actual input observation | **DI** — Requires API queries: `context.getState()`, `system.getForce()`, `platform.getPropertyValue()`. Not in default output files. MEDIUM. [OpenMM §5.2; cross-framework §1.5] | **DA** — .log echoes actual values. Runtime auto-tuning changes (nstlist, rlist, PME) logged in .log. HIGH for initial; MEDIUM for runtime-adjusted values. [GROMACS §3.2, §2.3; cross-framework §1.5] | **DA** — vasprun.xml `<parameters>` shows resolved defaults and actual values used. HIGH. [VASP §3.1; cross-framework §1.5] |
| **R5** | Input validation result | **DE** — Computed from R3+R4 comparison. Confidence depends on R3/R4 quality. MEDIUM (limited by R3=DI). | **DE** — Computed from R3+R4 comparison. HIGH. Note: grompp also performs pre-execution validation (syntax/structure) that contributes. [GROMACS §7.1] | **DE** — Computed from R3+R4 comparison. HIGH. Note: VASP validates some INCAR params against POTCAR at startup. [VASP §5.4] |
| **R6** | Numerical status record | **DI** — NaN appears in StateDataReporter output but requires monitoring logic. No per-step numerical health metrics by default. Energy conservation must be derived. Per-force-group decomposition needs custom reporter with `getState(groups={i})`. MEDIUM. [OpenMM §1.1, §3.1, §5.2; cross-framework §1.2, §1.3] | **DA** — LINCS warnings with rms/max deviation in .log (native). Energy values in .edr detectable for NaN/divergence (native). Energy drift in .log final statistics (native). HIGH. [GROMACS §5.2, §3.2, §4.1; cross-framework §1.3] | **DA** — SCF convergence trajectory (dE, d_eps, ncg) in OSZICAR/vasprun.xml per SCF step (native). Ionic convergence (forces) in vasprun.xml (native). **FU** for sub-SCF numerical issues (FFT aliasing, PAW reconstruction errors — hidden by closed source). MEDIUM-HIGH for observable metrics; LOW for internal numerical state. [VASP §1.1, §3.1, §5.1; cross-framework §1.3, §4.3] |
| **R7** | Resource/environment status | **DI** — Platform name, precision, device index queryable via `context.getPlatform()` API at runtime. GPU memory via pynvml (external). Not in default output files. MEDIUM. [OpenMM §4.1; cross-framework §1.6] | **DA** — .log header: GROMACS version, build config, CPU/GPU detection, SIMD, MPI/thread counts. Performance summary with per-component timing. HIGH. [GROMACS §3.2; cross-framework §1.6] | **DA** — vasprun.xml `<generator>` (version, platform). OUTCAR timing/memory/parallelization. stdout MPI decomposition. HIGH. [VASP §1.1, §3.2; cross-framework §1.6] |

### Stage 1 Summary

| Framework | DA | DI | DE | FU | Coverage |
|:---|:---|:---|:---|:---|:---|
| OpenMM | 2 (R1, R2) | 4 (R3, R4, R6, R7) | 1 (R5) | 0 | Full — all satisfiable with custom instrumentation |
| GROMACS | 5 (R1, R2, R3, R4, R6) | 0 | 1 (R5) | 0 | Full — mostly native |
| VASP | 4 (R1, R2, R3, R4) | 0 | 1 (R5) | Partial (R6 sub-SCF) | Full for observable metrics; partial FU for internal numerical state |

**Note:** R7 counts as DA for GROMACS and VASP, DI for OpenMM. R6 for VASP has a split classification: DA for SCF convergence metrics, FU for sub-SCF numerical internals.

---

## 2. Cross-Cutting Requirements (R19-R29)

These span all three audit stages. R19 (layer tag) is the load-bearing structural requirement.

| Req | Description | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|:---|
| **R19** | Layer tag (impl vs. theory) | **DA** — API-enforced separation at `ForceField.createSystem()`. Theory: ForceField/Topology/System. Implementation: Platform/Context. ~4 boundary params (nonbondedMethod, constraints, ewaldErrorTolerance, hydrogenMass) are well-defined and few. HIGH. [OpenMM §2; cross-framework §2.1] | **DI+ER** — .mdp parameters need classification table. ~10 boundary params (dt, nsteps, rlist, fourierspacing, etc.) require dual-annotation. mdrun CLI is purely implementation. Classification is stable but not API-declared. MEDIUM. [GROMACS §2; cross-framework §2.2, §2.4] | **ER** — Flat INCAR namespace requires external classification table for ~200-300 tags (~50-80 commonly used). Ambiguous params (PREC, LREAL, ALGO) have context-dependent classification for pathological systems. Classification table is finite and static but requires domain expert construction. LOW-MEDIUM. [VASP §2; cross-framework §2.3, §2.4, §6] |
| **R20** | Provenance chain | **DI** — Default reporters produce simple CSV/binary with step-based structure. Source traceability requires custom reporter design that records which API call/reporter produced each element. MEDIUM. [OpenMM §1] | **DA** — Multi-file output (.log/.edr/.trr/.tpr) has clear per-file structure with identifiable sections and timestamps. Elements traceable to file + section + line range. HIGH. [GROMACS §1; cross-framework §1] | **DA** — vasprun.xml has structured XML with clear XPath-addressable nodes. OUTCAR sections identifiable by markers. HIGH. [VASP §1.1, §3.1; cross-framework §1] |
| **R21** | Temporal ordering | **DA** — StateDataReporter includes step number and simulation time. DCD/PDB frames carry step indices. HIGH. [OpenMM §1.1; cross-framework §1] | **DA** — .edr frames timestamped. .log events carry step numbers. .trr/.xtc frames timestamped. HIGH. [GROMACS §1.3, §3.2; cross-framework §1] | **DA** — vasprun.xml ionic/SCF steps are sequentially nested. OSZICAR has iteration indices. Total ordering clear. HIGH. [VASP §1.1, §3.1; cross-framework §1] |
| **R22** | Experiment specification linkage | **NT** — The experiment specification (Python script defining System + Integrator + Platform) is an external input. The IR must reference it but cannot extract it from trace data. The trace echoes some parameter values (R3/R4) but not the script structure/intent. HIGH. [LFI req log] | **NT** — The experiment specification (.mdp + topology + structure files) is an external input. Parameter echo in .log captures values but not the specification document identity. HIGH. [LFI req log] | **NT** — The experiment specification (INCAR + POSCAR + POTCAR + KPOINTS) is an external input. Parameter echo in vasprun.xml captures values but not file provenance. HIGH. [LFI req log] |
| **R23** | Hypothesis linkage | **NT** — Hypothesis comes from Causal Graph Manager, not trace. IR must be joinable to hypothesis via shared variable naming. HIGH. [LFI req log; ARCHITECTURE §5.3] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |
| **R24** | Queryability (multi-index) | **DE** — Structural requirement on IR organization. Satisfied by IR schema design (indexes on layer, event type, time range, variable name, DAG node, stage). Not a data extraction concern. HIGH. [LFI req log] | **DE** — Same. HIGH. | **DE** — Same. HIGH. |
| **R25** | Classification confidence metadata | **DI** — Requires custom reporter logic to track which data categories were captured and at what resolution. Completeness = f(active reporters, reporting frequency). MEDIUM. [OpenMM §5.2; ARCHITECTURE §8.4] | **DI** — Derivable from output configuration (nstenergy, nstxout, etc.) and file existence checks. Crash-state availability flaggable from checkpoint timing. MEDIUM. [GROMACS §1.4, §6.5] | **DI** — Derivable from file existence + closed-source ceiling assessment. Well-characterized ceiling means gaps are predictable. MEDIUM-HIGH (gaps are well-known). [VASP §7.1; cross-framework §4.3] |
| **R26** | Observability gap record | **DI** — Adapter's `declare_data_completeness()` enumerates: which reporters active, what data absent (velocities? forces? per-group energy?), temporal gap between last report and crash. MEDIUM. [OpenMM §5.3, §6.5; cross-framework §5.3] | **DI** — Adapter declares: crash-state gap (checkpoint vs. crash timing), output frequency gaps, methodology metrics not computed. MEDIUM. [GROMACS §6.5; cross-framework §5.3] | **DI** — Adapter declares: closed-source ceiling (internal SCF state, FFT aliasing, PAW errors), warnings not in vasprun.xml, crash-induced file truncation. Gaps are well-characterized and largely static per VASP version. HIGH. [VASP §7.1; cross-framework §4.3, §5.3] |
| **R27** | Confounder-as-methodological classification | **NT+DE** — Confounder detection requires DAG context (R14) + controlled variable set (R13) + observable (R8). The classification of confounder as "methodological" (Stage 2, not Stage 3) is an LFI design decision supported by IR structure, not extracted from trace. HIGH (if R8, R13, R14 satisfied). [hidden-confounder §2; LFI req log] | **NT+DE** — Same. HIGH. | **NT+DE** — Same. HIGH. |
| **R28** | Interventional vs. observational distinction | **NT** — Whether an experiment was interventional or observational is determined by experiment design, not trace data. The trace looks the same either way; the tag must come from the experiment specification or workflow controller. HIGH. [hidden-confounder §2; LFI req log] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |
| **R29** | Cross-experiment queryability | **NT+DE** — experiment_cycle_id is an external label (NT). Cross-experiment querying is a structural IR design requirement (DE). Neither comes from individual trace data. HIGH. [hidden-confounder §2; LFI req log] | **NT+DE** — Same. HIGH. | **NT+DE** — Same. HIGH. |

### Cross-Cutting Summary

| Framework | DA | DI | ER | NT | DE | NT+DE |
|:---|:---|:---|:---|:---|:---|:---|
| OpenMM | 2 (R20 partial, R21) | 3 (R19 partial, R25, R26) | 0 | 3 (R22, R23, R28) | 1 (R24) | 2 (R27, R29) |
| GROMACS | 2 (R20, R21) | 2 (R25, R26) | 1 (R19 partial) | 3 (R22, R23, R28) | 1 (R24) | 2 (R27, R29) |
| VASP | 2 (R20, R21) | 2 (R25, R26) | 1 (R19) | 3 (R22, R23, R28) | 1 (R24) | 2 (R27, R29) |

**Critical finding:** R19 (layer tag) is the only cross-cutting requirement with a framework-dependent classification difficulty. OpenMM is DA (clean API boundary). GROMACS requires a moderate classification table (DI+ER). VASP requires an extensive classification table (ER) with context-dependent ambiguous parameters. This confirms the plan prediction and validates Decision Gate 1's acceptance of VASP with conditions.

---

## 3. Stage 2: Methodological Audit (R8-R14)

Stage 2 is the hardest stage for all three frameworks. Methodology-layer data (sampling adequacy, confounder control, intervention design) is largely invisible to DSL frameworks. Most R8-R14 cells are NT (from experiment spec/hypothesis/DAG) or DI (requiring post-processing analysis). The IR's role in Stage 2 is limited: it provides R8 (observable values), R12 (sampling metadata), and R13 (controlled variable values). The rest is external context.

| Req | Description | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|:---|
| **R8** | Observable measurement record | **DI** — Energy values from StateDataReporter (DA for total energy). Most scientific observables (g(r), RMSD, free energy, etc.) require post-processing of trajectory data via MDAnalysis/mdtraj. Per-force-group energy decomposition requires custom reporter. MEDIUM. [OpenMM §1.1, §1.2-1.7, §5.2; cross-framework §1.1, §1.2] | **DA** — Energy decomposition per force type in .edr (native). Structural observables require post-processing of .trr/.xtc. Raw trajectory data is native; computed observables are DI. MEDIUM-HIGH. [GROMACS §4.1, §1.4-1.5; cross-framework §1.1, §1.2] | **DA** — Total energy, forces, eigenvalues, DOS, band structure in vasprun.xml (native). DFT observables are structurally simpler (energy per atom, forces, electronic structure) and mostly available directly. HIGH. [VASP §1.1, §3.1, §3.2; cross-framework §1.1, §1.2] |
| **R9** | Observable-to-DAG linkage | **NT** — Mapping measured observables to DAG nodes requires external ontology/naming convention shared between Trace Semantics Engine and Causal Graph Manager. Not in trace data. HIGH. [LFI req log] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |
| **R10** | Intervention specification | **NT** — Which parameters were varied (and their ranges) is determined by experiment design. The trace echoes parameter values used in each run, but does not know which were "interventions" vs. "controls." HIGH. [LFI req log] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |
| **R11** | Intervention-to-DAG linkage | **NT** — Mapping interventions to DAG edges requires DAG structure. Not in trace data. HIGH. [LFI req log] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |
| **R12** | Sampling metadata | **DI** — Number of steps from simulation setup (DA). Equilibration period must be determined by post-hoc analysis of energy/RMSD time series. Autocorrelation time not computed by OpenMM. Statistical power not assessed. MEDIUM. [OpenMM §1.1; cross-framework §3.1, §3.4] | **DI** — Number of steps from .mdp echo (DA). Equilibration quality must be assessed post-hoc from .edr time series. Energy drift reported in .log final stats (DA). Autocorrelation requires external tools. MEDIUM. [GROMACS §3.2, §4.1; cross-framework §3.1, §3.4] | **DI** — Number of ionic steps in vasprun.xml (DA). SCF iteration counts (DA). For DFT, "sampling" means k-point mesh density and ENCUT completeness — these are input parameters (R3), not sampling metadata in the MD sense. Convergence testing (multiple ENCUT/k-point runs) is an external multi-experiment analysis. MEDIUM. [VASP §3.1; cross-framework §3.1, §3.4] |
| **R13** | Controlled variable set | **NT+DI** — Which variables are "controls" is experiment design (NT). Their actual values are obtainable from trace: via API queries (DI). MEDIUM. [OpenMM §2.1, §5.2; LFI req log] | **NT+DA** — Which variables are "controls" is experiment design (NT). Their actual values are in .log parameter echo (DA). HIGH for values. [GROMACS §3.2; LFI req log] | **NT+DA** — Which variables are "controls" is experiment design (NT). Their actual values are in vasprun.xml parameter echo (DA). HIGH for values. [VASP §3.1; LFI req log] |
| **R14** | DAG confounder query support | **DE** — Structural/queryability requirement on IR, not a data extraction concern. Requires R8, R10, R13 to be populated and joinable to DAG. Satisfied by IR design. HIGH (if dependencies met). [LFI req log] | **DE** — Same. HIGH. | **DE** — Same. HIGH. |

### Stage 2 Summary

| Framework | DA | DI | NT | NT+DI/DA | DE |
|:---|:---|:---|:---|:---|:---|
| OpenMM | 0 | 2 (R8, R12) | 3 (R9, R10, R11) | 1 (R13) | 1 (R14) |
| GROMACS | 1 (R8 partial) | 1 (R12) | 3 (R9, R10, R11) | 1 (R13) | 1 (R14) |
| VASP | 1 (R8) | 1 (R12) | 3 (R9, R10, R11) | 1 (R13) | 1 (R14) |

**Critical finding:** Stage 2 depends far more on external context (experiment specification, hypothesis, DAG) than on trace data. The IR's contribution to Stage 2 is limited to providing observable values (R8) and sampling metadata (R12). The remaining Stage 2 requirements (R9, R10, R11, R13-identification, R14) are external to the Trace Semantics Engine. This is consistent with the accumulated finding that "methodology-layer failures are invisible to all three frameworks" [FINDINGS.md, What We Know #3].

---

## 4. Stage 3: Theoretical Evaluation (R15-R18)

Stage 3 requirements are straightforward once Stage 2 is assessed. R15 and R18 are NT (from hypothesis/DAG). R16 draws on trace data. R17 is DE (computed from R15+R16).

| Req | Description | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|:---|
| **R15** | Prediction record | **NT** — Hypothesis predictions come from the Causal Graph Manager and hypothesis under test, not from trace data. HIGH. [LFI req log; ARCHITECTURE §5.1] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |
| **R16** | Observation record | **DI** — Subset of R8 relevant to hypothesis predictions. Energy values from StateDataReporter (DA for scalars). Most predicted observables require post-processing of trajectory. Confidence depends on what was predicted. MEDIUM. [OpenMM §1.1, §5.2] | **DA** — Energy decomposition from .edr (native for energy-based predictions). Structural observables from trajectory require post-processing. MEDIUM-HIGH. [GROMACS §4.1] | **DA** — Energy, forces, eigenvalues, DOS from vasprun.xml (native for standard DFT predictions). HIGH for standard DFT observables. [VASP §3.1] |
| **R17** | Comparison result | **DE** — Computed from R15 (prediction) + R16 (observation). Effect size, divergence measure, tolerance check. Requires formalized quantitative comparison method (not yet designed — this is an open research element). HIGH (mechanically), but comparison method is novel research. [LFI req log; ir-pattern-catalog.md §7] | **DE** — Same. HIGH. | **DE** — Same. HIGH. |
| **R18** | Causal implication mapping | **NT** — Mapping contradicted predictions to implicated DAG edges requires DAG structure. This is the LFI's core reasoning step for theoretical falsification, not a data extraction task. HIGH. [LFI req log; ARCHITECTURE §5.3] | **NT** — Same. HIGH. | **NT** — Same. HIGH. |

### Stage 3 Summary

| Framework | DA | DI | NT | DE |
|:---|:---|:---|:---|:---|
| OpenMM | 0 | 1 (R16) | 2 (R15, R18) | 1 (R17) |
| GROMACS | 1 (R16 partial) | 0 | 2 (R15, R18) | 1 (R17) |
| VASP | 1 (R16) | 0 | 2 (R15, R18) | 1 (R17) |

**Finding:** Stage 3 is dominated by NT (hypothesis/DAG context) and DE (computed comparison). The trace data contribution is limited to R16 (the observation values), which has the same availability profile as R8.

---

## 5. Gap Analysis

### 5.1 Non-DA Cells by Fill Strategy

For each non-DA classification, the following fill strategy applies:

#### Strategy A: Custom Instrumentation (DI cells)

These gaps are fillable by building adapter-level instrumentation code.

| Req | Framework | Adapter Method | Complexity | Overhead |
|:---|:---|:---|:---|:---|
| R3 | OpenMM | Query System/Integrator/Platform via API at simulation start | Low | Negligible (one-time) |
| R4 | OpenMM | Same as R3 + periodic re-query for mutable state | Low | Negligible |
| R6 | OpenMM | Custom reporter: `getState(getEnergy=True, groups={i})` for per-force-group decomposition + NaN detection | Medium | Unknown — per-force-group energy decomposition cost untested [What We Don't Know #2] |
| R7 | OpenMM | Query `context.getPlatform()` at start + pynvml for GPU memory | Low | Negligible |
| R8 | OpenMM | Custom reporter for energy; MDAnalysis/mdtraj for structural observables | Medium-High | Post-processing; not real-time |
| R12 | OpenMM | Count steps from setup; derive equilibration from energy time series | Medium | Post-processing |
| R12 | GROMACS | Parse .mdp nsteps; derive equilibration from .edr | Medium | Post-processing |
| R12 | VASP | Parse vasprun.xml ionic counts; k-point/ENCUT adequacy is rule-based (→ ER) | Medium | Post-processing |
| R16 | OpenMM | Same mechanism as R8 | Medium-High | Post-processing |
| R19 | GROMACS | Apply parameter classification table to .mdp echo | Low | Negligible |
| R20 | OpenMM | Custom reporter design with source annotation | Medium | Moderate (metadata overhead) |
| R25 | All | Adapter `declare_data_completeness()` logic | Medium | Negligible |
| R26 | All | Adapter `declare_data_completeness()` gap enumeration | Medium | Negligible |
| R13 (values) | OpenMM | API query for parameter values | Low | Negligible |

**Total DI count:** OpenMM: 10 cells requiring instrumentation. GROMACS: 4. VASP: 4.
OpenMM has the highest instrumentation burden, consistent with its 30-40% default observability [cross-framework §4.1].

#### Strategy B: External Rules / Classification Tables (ER cells)

These gaps require maintained domain-knowledge artifacts.

| Req | Framework | Required Artifact | Complexity | Maintenance |
|:---|:---|:---|:---|:---|
| R19 | GROMACS | .mdp parameter classification table (~10 boundary params + ~50 total params) | Low-Medium | Low (parameter semantics rarely change between GROMACS versions) |
| R19 | VASP | INCAR tag classification table (~200-300 tags, ~50-80 commonly used) | Medium-High | Medium (VASP 5→6 default changes; new tags in new versions). Must be version-aware. [cross-framework §6.4] |

**Key risk:** VASP R19 is the only ER cell that is both large (200-300 tags) and has context-dependent ambiguity (~5-10 truly ambiguous tags like PREC, ALGO, LREAL). The GROMACS table is small and mostly unambiguous.

#### Strategy C: Pre-Execution / Experiment Spec Analysis (NT cells)

These elements come from sources external to the Trace Semantics Engine.

| Req | Source | All Frameworks |
|:---|:---|:---|
| R9 | DAG + variable naming ontology | Requires shared naming convention between Trace Semantics Engine and Causal Graph Manager |
| R10 | Experiment specification | Experiment design document must declare interventions |
| R11 | DAG + experiment specification | Requires mapping intervention params to DAG edges |
| R15 | Hypothesis under test | Hypothesis must declare quantitative predictions |
| R18 | DAG + falsification result | LFI reasoning step, not data extraction |
| R22 | Experiment specification | IR must reference spec document |
| R23 | Hypothesis | IR must be joinable to hypothesis |
| R28 | Experiment specification | Experiment must be tagged as interventional/observational |
| R29 (cycle_id) | Workflow controller | Multi-experiment cycle IDs assigned externally |

**Implication:** 9 of 29 requirements (31%) are purely NT. This confirms that the IR is not a pure trace-log derivative — it is a composite structure joining trace data, experiment specifications, hypotheses, and DAG references [FINDINGS.md, What We Know #20].

#### Strategy D: Derived Computation (DE cells)

| Req | Derived From | Computation |
|:---|:---|:---|
| R5 | R3 + R4 | Per-parameter match/mismatch comparison |
| R14 | R8 + R10 + R13 + DAG | Graph query: common causes of intervention and observable not in controlled set |
| R17 | R15 + R16 | Quantitative comparison: effect size, divergence, tolerance check |
| R24 | IR structure | Index construction (by layer, type, time, variable, DAG node, stage) |
| R27 | R8 + R13 + R14 + DAG | Confounder classification as methodological |
| R29 (queryability) | IR structure + cycle_id | Cross-experiment join capability |

#### Strategy E: Fundamentally Unavailable (FU cells)

| Req | Framework | What's Missing | Impact |
|:---|:---|:---|:---|
| R6 (partial) | VASP | Sub-SCF numerical state: FFT aliasing errors, PAW reconstruction errors, internal solver state, non-deterministic MPI reduction effects | Affects classification of ~5-10% of VASP numerical failures where the observable metrics (SCF convergence trajectory) are normal but internal numerical issues exist. Impact: degraded confidence for a small subset of implementation-layer classifications. [cross-framework §4.3, §7.4] |
| R6 (partial) | OpenMM | Sub-step integrator operations, GPU frame buffer state during force evaluation, CUDA kernel execution order | Affects classification of non-deterministic precision failures (I4: non-deterministic force summation). Impact: requires `DeterministicForces=true` workaround or platform comparison diagnostic. [OpenMM §5.3; cross-framework §4.1] |
| R6 (partial) | GROMACS | Internal constraint solver iteration history (only final result reported), crash-step full state (only checkpoint available) | Affects forensic analysis of constraint failures (G-A1). Impact: "last known state" semantics required. [GROMACS §5.2, §6.5; cross-framework §4.2] |

**Finding:** FU cells are narrowly scoped to sub-component numerical internals, not to any full requirement. No R1-R29 requirement is entirely FU for any framework. The FU classifications are partial — the surface-level data is available (DA/DI), but deeper internal state is hidden. This is consistent with Decision Gate 1's acceptance of VASP [cross-framework §6].

### 5.2 Gap Summary by Framework

| Metric | OpenMM | GROMACS | VASP |
|:---|:---|:---|:---|
| DA cells (R1-R29) | 4 | 9 | 9 |
| DI cells | 10 | 4 | 4 |
| ER cells | 0 | 1 | 1 |
| NT cells | 9 | 9 | 9 |
| DE cells | 6 | 6 | 6 |
| FU cells (partial) | 1 | 1 | 1 |
| **Instrumentation burden** | **Highest** | **Low** | **Low** |
| **Classification table burden** | **None** | **Low** | **Medium-High** |

**OpenMM** has the most DI cells because it lacks built-in parameter echo and most data requires API queries via custom reporters. However, OpenMM's reporter extensibility makes these gaps fillable with moderate engineering effort.

**GROMACS** has the best default coverage due to comprehensive .log/.edr output and grompp preprocessing validation.

**VASP** has good default coverage for its structured output (vasprun.xml) but carries the highest ER burden due to the INCAR classification table requirement (R19).

---

## 6. LFI Stage Feasibility Assessment

### 6.1 Stage 1: Implementation Audit

**Question:** Can the IR provide sufficient data for Stage 1 to deterministically answer Q1.1-Q1.4?

| Framework | Feasible? | Evidence | Residual Risks |
|:---|:---|:---|:---|
| **OpenMM** | **YES** with custom instrumentation | R1 (DA), R2 (DA), R3-R4 (DI — fillable via API), R5 (DE), R6 (DI — custom reporter needed), R7 (DI — API + pynvml). All gaps are fillable. [OpenMM survey; cross-framework §4.1] | Sub-step crash state unrecoverable (FU). Reporter temporal gap (DI — adaptive frequency helps). Per-force-group energy overhead unknown. |
| **GROMACS** | **YES** natively for most cases | R1-R4 (DA), R5 (DE), R6 (DA — LINCS warnings + .edr), R7 (DA). Only gap: crash-state coordinates (partial — confout.gro in GROMACS 2020+). [GROMACS survey; cross-framework §4.2] | Crash-step velocities/forces lost (FU). Error classification requires pattern matching on free-text (adapter engineering). |
| **VASP** | **YES** with caveats | R1 (DA — but exit code 0 for non-convergence), R2 (DA), R3-R4 (DA), R5 (DE), R6 (DA for SCF trajectory, FU for internals), R7 (DA). [VASP survey; cross-framework §4.3] | Exit code unreliability for non-convergence (mitigated by R6 SCF convergence check). Sub-SCF numerical internals hidden (FU). Crash truncates output files. |

**Stage 1 verdict: FEASIBLE for all three frameworks.** The primary risk is the universal crash-state gap (all frameworks lose some state at the moment of failure). The IR must adopt "last known state" semantics as specified in the accumulated findings [FINDINGS.md, What We Know #4].

### 6.2 Stage 2: Methodological Audit

**Question:** Can the IR provide sufficient data for Stage 2 to deterministically answer Q2.1-Q2.4?

| Framework | Feasible? | Evidence | Residual Risks |
|:---|:---|:---|:---|
| **OpenMM** | **CONDITIONAL** — depends on experiment spec quality and DAG accuracy | R8 (DI — fillable), R9 (NT), R10 (NT), R11 (NT), R12 (DI — fillable), R13 (NT+DI), R14 (DE). Trace data contributes R8 (observable values) and R12 (step counts). Everything else is external context. [LFI req log] | Methodology failures are invisible to OpenMM [What We Know #3]. Sampling sufficiency assessment requires external rules. DAG accuracy determines confounder check quality [ARCHITECTURE §8.5]. |
| **GROMACS** | **CONDITIONAL** — same dependencies | R8 (DA partial), R12 (DI — fillable). Slightly better than OpenMM due to native energy decomposition. Same external dependencies for R9-R11, R13-R14. [LFI req log] | Same as OpenMM. grompp warnings provide some methodology signals (Berendsen thermostat, box size) but most methodology inadequacies are silent. |
| **VASP** | **CONDITIONAL** — same dependencies, with ER for silent failure detection | R8 (DA), R12 (DI). VASP has additional ER requirement: domain-aware validation rules for silent methodology failures (wrong ISMEAR, SIGMA too large). These rules partially compensate for VASP's higher silent failure rate. [VASP §5.4-5.5, §6] | Same external dependencies. VASP's silent failure detection rules (ER) are the only framework-specific methodology assessment capability. |

**Stage 2 verdict: CONDITIONALLY FEASIBLE for all three frameworks.** Stage 2 feasibility depends primarily on:
1. Experiment specification quality (must declare interventions, controls, and predictions)
2. DAG accuracy (confounder check quality = DAG quality) [ARCHITECTURE §8.5]
3. The IR providing R8 (observables) and R12 (sampling metadata) — both satisfiable

The IR's role in Stage 2 is necessary but not sufficient. Most Stage 2 intelligence resides in the experiment specification and DAG, not in trace data. This is a fundamental architectural reality, not a gap to be filled.

### 6.3 Stage 3: Theoretical Evaluation

**Question:** Can the IR provide sufficient data for Stage 3 to deterministically answer Q3.1-Q3.2?

| Framework | Feasible? | Evidence | Residual Risks |
|:---|:---|:---|:---|
| **OpenMM** | **YES** if prediction-observation comparison formalized | R15 (NT — from hypothesis), R16 (DI — fillable, same as R8), R17 (DE — computed from R15+R16), R18 (NT — from DAG). The IR contributes R16 (observation values). Comparison method (R17) is novel research, not data availability. [LFI req log] | Quantitative prediction-observation comparison method is not yet formalized (open research element) [ir-pattern-catalog.md §7, Open Thread]. Observation values for complex predictions (free energy, kinetic rates) require post-processing. |
| **GROMACS** | **YES** if prediction-observation comparison formalized | Same structure. R16 slightly better (native energy decomposition in .edr). [LFI req log] | Same. |
| **VASP** | **YES** if prediction-observation comparison formalized | Same structure. R16 strongest (standard DFT observables natively in vasprun.xml). [LFI req log] | Same. Ironically, VASP has the best Stage 3 data availability despite having the worst Stage 1 boundary separation (R19). |

**Stage 3 verdict: FEASIBLE for all three frameworks**, pending formalization of the quantitative prediction-observation comparison method (R17). This is a well-scoped research problem: define effect size measures, divergence metrics, and tolerance thresholds for scientific predictions. It is IR-design-adjacent but not a data availability concern.

---

## 7. Decision Gate 4 Assessment

### Gate Criterion

> Any LFI stage where FU requirements block determinate classification for >10% of expected failures in a framework.

### Assessment

**Stage 1:**
- FU cells are partial R6 (sub-component numerical internals) in all three frameworks.
- These affect: OpenMM ~5% (non-deterministic GPU precision — failure mode I4), GROMACS ~5% (constraint solver internals — contributes to G-A1 ambiguity), VASP ~5-10% (FFT/PAW internals — contributes to V-A1/V-A2 ambiguity).
- **Below 10% threshold for all frameworks.** The surface-level numerical metrics (energy trajectory, convergence trajectory, constraint warnings) are available (DA/DI) and sufficient for most Stage 1 classifications.

**Stage 2:**
- No FU cells. The gaps are NT (external context) and DI (fillable with instrumentation).
- Stage 2 feasibility is not limited by data availability but by experiment specification quality and DAG accuracy.
- **Not applicable — no FU blocks.**

**Stage 3:**
- No FU cells. The gap is the prediction-observation comparison formalization (R17 — DE, novel research).
- R16 (observations) is DA or DI for all frameworks.
- **Not applicable — no FU blocks.**

### Decision Gate 4 Verdict: **PASS**

No LFI stage has FU requirements that block determinate classification for >10% of expected failures in any framework.

**Conditions:**
1. OpenMM requires custom ATHENA reporter (DI cells) — without it, Stage 1 degrades to ~40% coverage [cross-framework §4.1].
2. VASP's INCAR classification table (ER) must be constructed and validated — without it, R19 is unsatisfied and the three-stage audit cannot route correctly.
3. VASP's closed-source ceiling limits confidence for ~20-30% of calculations involving ambiguous parameters (PREC, ALGO, LREAL) [cross-framework §6.3]. This is accepted per Decision Gate 1.
4. The prediction-observation comparison method (R17) must be formalized — this is novel research, not a data gap.

### Escalation Options (Not Triggered)

Had the gate tripped, these options were available:
- (a) Accept degraded confidence for specific failure modes
- (b) Drop framework from initial target set
- (c) Redesign requirement to reduce data dependency
- (d) Add mandatory instrumentation requirement to adapter contract
- (e) Defer to causal DAG for disambiguation

Option (a) is already in effect for VASP's ambiguous parameter subset.

---

## 8. Implications for Step 5a: Candidate IR Schema Evaluation

### Coverage Patterns by Candidate

The coverage matrix reveals which data-source patterns favor each candidate IR design [ir-pattern-catalog.md §6]:

#### LEL (Layered Event Log)

**Favored by:**
- High DA density in Stage 1 (R1-R7) — event log naturally captures execution events, errors, parameter echoes, numerical metrics.
- Streaming compatibility: DA cells produce data incrementally, matching LEL's append-only structure.
- R21 (temporal ordering) — native in an event log.

**Challenged by:**
- NT cells (31% of requirements) — event logs are poor at representing external context (specs, hypotheses, DAG references). These would need to be injected as "specification events" or maintained as separate joinable structures.
- Stage 2/3 DE cells (R14, R17, R27) requiring cross-referencing and computed comparisons — flat log does not support structural joins naturally.

**Assessment:** LEL is the strongest candidate for Stage 1 and the weakest for Stages 2-3. The coverage matrix confirms this: Stage 1 has the most DA cells, and LEL excels at capturing directly-available event data.

#### DGR (Dual-Graph IR)

**Favored by:**
- NT cells (experiment specs, hypotheses, DAG references) — graph structure naturally represents these as entities with qualified relationships. The "dual" prospective/retrospective split maps to NT (specification) vs. DA/DI (trace).
- DE cells requiring cross-referencing (R5, R14, R17, R27) — graph traversal supports joins between predictions, observations, interventions, and confounders.
- R24 (queryability) — graph indexes support multi-index lookup.
- R29 (cross-experiment queryability) — graph structure supports cross-cycle entity linking.

**Challenged by:**
- Pure event data (Stage 1 DA cells) — converting event streams into graph entities adds impedance mismatch cost.
- Streaming: graph construction from streaming data requires forward-reference management [ir-pattern-catalog.md §6, DGR weaknesses].

**Assessment:** DGR is the strongest candidate for Stages 2-3 and handles the composite nature of the IR (trace + spec + hypothesis + DAG) most naturally. The coverage matrix confirms this: the high NT density (31%) and cross-referencing DE requirements favor graph structure. **DGR is the recommended primary candidate** for Step 5a evaluation.

#### TAL (Typed Assertion Log)

**Favored by:**
- Sequential audit structure: assertions ordered by stage map directly to Stage 1→2→3 flow.
- NT cells as "ghost state" assertions: methodology and theory assertions are populated from external context, matching TAL's design where assertions can come from any source.
- R25 (confidence) — each assertion carries its own evidence quality, making confidence explicit.

**Challenged by:**
- DA cells producing raw data (R1-R7) — converting raw events into structured assertions requires domain-specific logic. Where do assertions come from? [ir-pattern-catalog.md §6, TAL weaknesses]
- R14, R27 (graph-based reasoning) — assertion chains don't support causal graph traversal.
- Highest novelty risk [ir-pattern-catalog.md §6].

**Assessment:** TAL aligns well with the LFI's assertion-checking workflow but imposes the highest barrier for data extraction (turning DA/DI data into assertions). The coverage matrix suggests TAL is a better LFI query interface than a data representation — it may work as a layer on top of LEL or DGR rather than as the primary IR.

### Recommendation for Step 5a

1. **Evaluate DGR as the primary candidate** — the coverage matrix shows the IR is fundamentally a composite multi-source structure (trace + spec + hypothesis + DAG), and DGR handles this naturally.
2. **Evaluate LEL as the Stage 1 prototype** — for the simplest path to a working Stage 1 implementation audit, LEL's low impedance mismatch with DA cells is advantageous. Incremental evolution from LEL to DGR is plausible [ir-pattern-catalog.md §7, Question 5].
3. **Evaluate TAL as a query interface layer** — rather than as a standalone IR, TAL's assertion structure may work as the LFI's query language over a DGR or LEL substrate.

### Key Data Flow for IR Design

The coverage matrix reveals the IR's data flow architecture:

```
External Inputs (NT)              Trace Data (DA/DI)           Domain Knowledge (ER)
├── Experiment spec (R3,R10,R13)  ├── Execution events (R1,R2)  ├── GROMACS param table (R19)
├── Hypothesis (R15,R23)          ├── Numerical metrics (R6)    └── VASP INCAR table (R19)
├── DAG (R9,R11,R18)              ├── Resource state (R7)
├── Cycle ID (R29)                ├── Parameter echo (R3,R4)
└── Observation mode (R28)        ├── Energy/observables (R8,R16)
                                  └── Temporal markers (R21)
              │                              │                          │
              └──────────────┬───────────────┘                          │
                             ▼                                          │
                    ┌─────────────────┐                                 │
                    │  IR Composition  │◄────────────────────────────────┘
                    │  (Adapter Layer) │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │Derived(DE)│  │Derived(DE)│  │Derived(DE)│
        │ R5       │  │ R14, R17 │  │R24,R27,R29│
        │(validation)│ │(reasoning)│ │(structure)│
        └──────────┘  └──────────┘  └──────────┘
```

This three-input architecture (trace + external + domain rules) should be the organizing principle for Step 5a candidate evaluation.

---

**Sources:** All citations reference the following documents:
- [OpenMM §N] = openmm-trace-analysis.md, Section N
- [GROMACS §N] = gromacs-trace-analysis.md, Section N
- [VASP §N] = vasp-trace-analysis.md, Section N
- [cross-framework §N] = cross-framework-synthesis.md, Section N
- [ir-pattern-catalog.md §N] = ir-pattern-catalog.md, Section N
- [LFI req log] = FINDINGS.md, LFI Audit → IR Requirements Mapping investigation log
- [ARCHITECTURE §N] = ARCHITECTURE.md, Section N
- [hidden-confounder §N] = evaluation/hidden-confounder/README.md, Section N
- [What We Know #N] = FINDINGS.md, Accumulated Findings → What We Know, item N
- [What We Don't Know #N] = FINDINGS.md, Accumulated Findings → What We Don't Know, item N
