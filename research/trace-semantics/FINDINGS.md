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

IN PROGRESS — Steps 1-7 and all synthesis steps (1d, 2c, 3b) complete. Step 5a (candidate IR schemas) complete: Hybrid LEL+DGR recommended (94/100). Step 5b (LEL prototype) complete. Step 5c (open thread resolution) complete: 5/5 threads resolved/narrowed/deferred with evidence. Step 6 (Hybrid LEL+DGR Phase 2 prototype) complete: `by_id` index implemented, `CausalOverlay` + R14 confounder query implemented. Step 7 (R17+R18 query implementation) complete: `compare_predictions` + `implicate_causal_nodes` implemented with depth-aware BFS helper. Step 9 complete: GROMACS adapter implemented on existing LEL types (`src/gromacs_adapter.rs`). Step 10 complete: VASP adapter implemented on existing LEL types (`src/vasp_adapter.rs`) with first adapter-level use of `ConvergencePoint` and `StateSnapshot`. Step 11 complete: hidden confounder prototype litmus validated end-to-end on VASP-derived traces. Step 12 complete: R17 quantitative comparison formalization narrowed with a trace-semantics-to-adversarial-reward interface contract. Step 13 complete (NARROWED): convergence trajectory representation recommends a hybrid raw-plus-summary design (Option D) with ComparisonProfileV1-compatible outputs and explicit WDK#40 hook. Step 14 complete (NARROWED): minimal `UncertaintySummary` schema direction selected (layered point summary + optional tagged distribution payload) with six-consumer trace and cross-adapter feasibility evidence. Session 19 added WDK#41 bookkeeping closure and WDK#43 prototype derivation rules for GROMACS/OpenMM convergence summaries. Session 20 resolves WDK#42 and WDK#44 in prototype scope via shared convergence derivation extraction, canonical taxonomy projection, OpenMM CSV support, and cross-framework equivalence tests. Crate now passes 119/119 tests with strict clippy clean.

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).


## Table of Contents

- [Research Question](#research-question)
- [Investigation Log](#investigation-log)
  1. [2026-02-21: WDK#26 — INCAR Classification Table Completeness](#2026-02-21-wdk26--incar-classification-table-completeness)
  2. [2026-02-21: WDK#25 — VASP Closed-Source Observability Ceiling](#2026-02-21-wdk25--vasp-closed-source-observability-ceiling)
  3. [2026-02-21: WDK#39 — prediction_id Type Harmonization](#2026-02-21-wdk39--prediction_id-type-harmonization)
  4. [2026-02-21: WDK#35 + WDK#36 — ContractTerm Value Extensions for VASP](#2026-02-21-wdk35--wdk36--contractterm-value-extensions-for-vasp)
  5. [2026-02-22: Step 14 — UncertaintySummary Schema for Divergence Metrics (WDK#40)](#2026-02-22-step-14--uncertaintysummary-schema-for-divergence-metrics-wdk40)
  6. [2026-02-22: Step 13 — Convergence Trajectory Representation (WDK#13)](#2026-02-22-step-13--convergence-trajectory-representation-wdk13)
  7. [2026-02-22: Step 12 — R17 Quantitative Comparison Formalization and Interface Contract](#2026-02-22-step-12--r17-quantitative-comparison-formalization-and-interface-contract)
  8. [2026-02-22: Step 11 — Hidden Confounder Prototype Litmus Test](#2026-02-22-step-11--hidden-confounder-prototype-litmus-test)
  9. [2026-02-22: Step 10 — VASP Adapter Implementation](#2026-02-22-step-10--vasp-adapter-implementation)
  10. [2026-02-21: Step 9: GROMACS Adapter for Cross-Framework Validation](#2026-02-21-step-9-gromacs-adapter-for-cross-framework-validation)
  11. [2026-02-21: Step 7: R17+R18 Query Implementation](#2026-02-21-step-7-r17r18-query-implementation)
  12. [2026-02-21: Hybrid LEL+DGR Phase 2 Prototype — CausalOverlay + R14 Query (Step 6)](#2026-02-21-hybrid-leldgr-phase-2-prototype--causaloverlay--r14-query-step-6)
  13. [2026-02-21: Open Thread Resolution (Step 5c)](#2026-02-21-open-thread-resolution-step-5c)
  14. [2026-02-20: Candidate IR Schemas with Hybrid LEL-DGR Recommendation (Step 5a)](#2026-02-20-candidate-ir-schemas-with-hybrid-lel-dgr-recommendation-step-5a)
  15. [2026-02-20: Requirements Coverage Matrix and Gap Analysis (Step 3b)](#2026-02-20-requirements-coverage-matrix-and-gap-analysis-step-3b)
  16. [2026-02-20: Comparative IR Synthesis (Step 2c)](#2026-02-20-comparative-ir-synthesis-step-2c)
  17. [2026-02-20: Cross-Framework Trace Synthesis (Step 1d)](#2026-02-20-cross-framework-trace-synthesis-step-1d)
  18. [2026-02-20: 21% RCA Baseline Characterization](#2026-02-20-21-rca-baseline-characterization)
  19. [2026-02-20: LFI Audit → IR Requirements Mapping](#2026-02-20-lfi-audit--ir-requirements-mapping)
  20. [2026-02-20 — Provenance Data Models and Scientific Workflow IR Survey](#2026-02-20--provenance-data-models-and-scientific-workflow-ir-survey)
  21. [Entry 1 — 2026-02-20: RCA and Formal Verification IR Survey](#entry-1--2026-02-20-rca-and-formal-verification-ir-survey)
  22. [Entry 001 — 2026-02-20: VASP Trace Output System Survey](#entry-001--2026-02-20-vasp-trace-output-system-survey)
  23. [2026-02-20: GROMACS Trace Format Characterization](#2026-02-20-gromacs-trace-format-characterization)
  24. [2026-02-20: OpenMM Trace Format Characterization](#2026-02-20-openmm-trace-format-characterization)
- [Accumulated Findings](#accumulated-findings)
  - [What We Know](#what-we-know)
  - [What We Suspect](#what-we-suspect)
  - [What We Don't Know](#what-we-dont-know)
- [Prototype Index](#prototype-index)
- [Next Steps](#next-steps)

## Investigation Log

> **Investigation Log Index** — 26 entries, reverse chronological.
>
> | # | Date | Identifier | Scope |
> | :--- | :--- | :--- | :--- |
> | 1 | 2026-02-23 | Session 20 | WDK#42 canonical taxonomy + WDK#44 placement decision + OpenMM CSV validation |
> | 2 | 2026-02-24 | Session 19 | WDK#41 closure + WDK#43 convergence-summary derivation |
> | 3 | 2026-02-21 | WDK#26 | INCAR classification table completeness |
> | 4 | 2026-02-21 | WDK#25 | VASP closed-source observability ceiling |
> | 5 | 2026-02-21 | WDK#39 | prediction_id type harmonization |
> | 6 | 2026-02-21 | WDK#35 + WDK#36 | ContractTerm value extensions for VASP |
> | 7 | 2026-02-22 | Step 14 | UncertaintySummary schema for divergence metrics |
> | 8 | 2026-02-22 | Step 13 | Convergence trajectory representation |
> | 9 | 2026-02-22 | Step 12 | R17 comparison formalization and interface contract |
> | 10 | 2026-02-22 | Step 11 | Hidden confounder prototype litmus test |
> | 11 | 2026-02-22 | Step 10 | VASP adapter implementation |
> | 12 | 2026-02-21 | Step 9 | GROMACS adapter for cross-framework validation |
> | 13 | 2026-02-21 | Step 7 | R17+R18 query implementation |
> | 14 | 2026-02-21 | Step 6 | Hybrid LEL+DGR Phase 2 prototype |
> | 15 | 2026-02-21 | Step 5c | Open thread resolution |
> | 16 | 2026-02-20 | Step 5a | Candidate IR schemas with Hybrid recommendation |
> | 17 | 2026-02-20 | Step 3b | Requirements coverage matrix and gap analysis |
> | 18 | 2026-02-20 | Step 2c | Comparative IR synthesis |
> | 19 | 2026-02-20 | Step 1d | Cross-framework trace synthesis |
> | 20 | 2026-02-20 | — | 21% RCA baseline characterization |
> | 21 | 2026-02-20 | — | LFI audit to IR requirements mapping |
> | 22 | 2026-02-20 | — | Provenance and scientific workflow IR survey |
> | 23 | 2026-02-20 | Entry 1 | RCA and formal verification IR survey |
> | 24 | 2026-02-20 | Entry 001 | VASP trace output system survey |
> | 25 | 2026-02-20 | — | GROMACS trace format characterization |
> | 26 | 2026-02-20 | — | OpenMM trace format characterization |

### 2026-02-23: Session 20 — WDK#42 Canonical Taxonomy + WDK#44 Placement Decision + OpenMM CSV Validation
**Date:** 2026-02-23
**Scope:** Resolve WDK#42 (canonical convergence taxonomy) and WDK#44 (placement decision) in prototype scope, while validating OpenMM CSV parsing against StateDataReporter-style output.
**Method:** (1) Extracted duplicated GROMACS/OpenMM convergence derivation into shared `convergence::derive_energy_convergence_summary` utility and kept adapter-inline invocation, (2) added canonical taxonomy projection (`CanonicalConvergence`, `classify_convergence`, `classify_all_convergence`) with divergence-priority override and framework-aware mapping, (3) extended OpenMM energy parser to support CSV headers containing `"Potential Energy (kJ/mol)"` with whitespace fallback retained, (4) added additive taxonomy/CSV/equivalence tests (A-F scenarios with VASP B/C documented as N/A), (5) diagnosed and fixed OpenMM parser gap where NaN energies were parsed but no `NumericalStatus` emitted (required for Divergent override), (6) executed full quality gates.

**Findings:**

1. **Shared derivation extraction is complete with no Stage 1->2 post-pass architecture added.** Both GROMACS and OpenMM now call adapter-inline `convergence::derive_energy_convergence_summary(..., "simulation.log")`, eliminating duplicated helper logic while preserving natural provenance at parse time.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/convergence.rs::derive_energy_convergence_summary`; `.../src/gromacs_adapter.rs`; `.../src/adapter.rs`.

2. **Provenance and uncertainty invariants remain intact for derived convergence events.** Derived summaries retain causal links to source energy events plus terminal execution/numerical context, use `Completeness::Derived { from_elements }`, and emit no convergence event when energy window `< 4`.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/convergence.rs::derive_energy_convergence_summary`; tests `test_mock_adapter_no_convergence_summary_below_min_window`, `test_gromacs_adapter_no_convergence_summary_below_min_window`, `test_*_convergence_summary_provenance_refs`.

3. **Canonical taxonomy projection is implemented without schema changes.** `CanonicalConvergence` and `classify_convergence` are read-only views over existing `TraceEvent` data; `ConvergencePoint` and existing event payloads were unchanged. Divergence override now consistently prioritizes `NumericalStatus{NaNDetected/InfDetected}` and `ExecutionStatus{CrashDivergent}` over metric-specific pattern mapping.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/convergence.rs::{CanonicalConvergence,classify_convergence,classify_all_convergence}`; tests `test_classify_convergence_*`.

4. **OpenMM CSV parsing now accepts StateDataReporter-like traces while preserving backward compatibility.** `parse_openmm_energy_series` detects CSV mode from `#"` headers, resolves the `"Step"` and `"Potential Energy"` columns dynamically, and falls back to the prior whitespace parser when CSV markers are absent.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs::{parse_openmm_energy_series,parse_openmm_csv_energy_series}`; tests `test_mock_adapter_parses_openmm_statedatareporter_csv_pairs`, `test_mock_adapter_whitespace_parser_backward_compat`.

5. **OpenMM divergent classification required a real adapter fix, not just fixture adjustment.** During scenario D validation, OpenMM parsed `NaN` values but did not emit `NumericalStatus`, preventing Divergent override. Adapter now emits `NumericalStatus::{NaNDetected,InfDetected}` for non-finite energy rows with causal link to the source `EnergyRecord`.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs` (OpenMM energy loop numerical-status emission); tests `test_openmm_csv_divergent_fixture_emits_nan_status`, `test_equivalence_scenario_d_divergent_nan`.

6. **Cross-framework equivalence coverage now demonstrates canonical labeling across six scenarios.** Scenarios A-F are represented in tests; GROMACS/OpenMM match canonical labels for converged/oscillating/stalled/divergent/insufficient/threshold-boundary conditions, and VASP asymmetry for oscillation/stall is explicitly documented as SCF-vs-ionic semantic difference (not taxonomy failure).  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/tests/mod.rs::test_equivalence_scenario_a_steady_state_converged` through `..._f_threshold_boundary`.

7. **Prototype gates remain clean after Session 20 changes.** Full suite passes 119/119 tests; strict clippy passes with zero warnings.  
   Evidence: `cargo test`; `cargo clippy -- -D warnings`.

**Implications:**

- **WDK#42 is resolved in prototype scope.** Canonical convergence taxonomy is now implemented as code-backed projection with explicit mapping tests and cross-framework scenario checks.
- **WDK#44 is resolved in prototype scope.** Placement decision is now explicit and implemented: adapter-inline derivation preserved, shared utility extraction adopted, no post-pass architecture introduced.
- **WDK#43 remains preserved and strengthened.** Existing derivation semantics are retained while duplication risk is removed.

**Open Threads:**

1. Validate OpenMM CSV behavior against additional real-world reporter variants (column order/optional fields) beyond current synthetic fixtures.
2. If production indexing needs differ, revisit post-pass architecture only with explicit provenance-preservation proofs and measured Stage 1->2 cost/benefit.

### 2026-02-24: Session 19 — WDK#41 Closure + WDK#43 Convergence-Summary Derivation
**Date:** 2026-02-24
**Scope:** Phase 1 bookkeeping closure for WDK#41 (bead + trace-semantics synthesis) and Phase 2 prototype implementation for WDK#43 convergence-summary derivation in GROMACS/OpenMM.
**Method:** (1) Verified adversarial-reward WDK#41 completion evidence from Session 7 and Accumulated Findings (`research/adversarial-reward/FINDINGS.md`), (2) closed stale bead `athena-apb` with cross-track citation, (3) updated trace-semantics Accumulated Findings to move WDK#41 to resolved, (4) studied VASP `ConvergencePoint` emission pattern (`parse_oszicar`) as reference behavior, (5) implemented per-adapter inline derivation for GROMACS and OpenMM mock adapter from existing `EnergyRecord` / `NumericalStatus` / `ExecutionStatus` events, (6) added synthetic convergence/oscillation/insufficient-data/provenance tests, (7) executed targeted tests plus full quality gates (`cargo test`, `cargo clippy -- -D warnings`).

**Findings:**

1. **WDK#41 bookkeeping closure is complete across tracker + synthesis.** `athena-apb` is now closed with explicit reason citing adversarial-reward Sessions 1-7 and locked recommendation artifact. Trace-semantics Accumulated Findings now marks WDK#41 resolved and removes it from open WDK inventory.  
   Evidence: `bd show athena-apb --json`; `research/trace-semantics/FINDINGS.md` (Accumulated Findings, Resolved/Narrowed #41); `research/adversarial-reward/FINDINGS.md` Session 7 + What We Know entries.

2. **VASP remains the reference implementation for native convergence events.** `parse_oszicar` emits per-iteration `ConvergencePoint` events from DAV/RMM rows, then back-patches `converged=Some(true)` when ionic-step `F=` lines appear, with adapter-level causal linkage to INCAR and downstream energy events.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/vasp_adapter.rs`.

3. **GROMACS convergence-summary derivation is now implemented inline in the adapter parse path.** Rules consume existing `EnergyRecord` totals (plus latest `ExecutionStatus`/`NumericalStatus` for provenance) and emit a derived `ConvergencePoint` when minimum data are available.  
   Minimum input condition: at least 4 `EnergyRecord` points (`GROMACS_MIN_CONVERGENCE_WINDOW=4`).  
   Rule set:
   - `derived_convergence_rel_delta_max` + `converged=Some(true)` when `max_rel_delta <= 1e-4`
   - `derived_oscillation_rel_delta_mean` + `converged=Some(false)` when `sign_changes >= 2` and `mean_rel_delta > 1e-4`
   - `derived_stall_rel_delta_mean` + `converged=Some(false)` otherwise  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/gromacs_adapter.rs`.

4. **OpenMM convergence-summary derivation is now implemented inline in the mock adapter parse path with matching semantics.** The adapter can parse synthetic reporter-like `<step> <energy>` input and derive `ConvergencePoint` using the same minimum condition and threshold family as GROMACS, while preserving uncertainty (no event emitted under insufficient input).  
   Minimum input condition: at least 4 parsed energy points (`OPENMM_MIN_CONVERGENCE_WINDOW=4`).  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs`.

5. **Provenance anchoring for derived convergence summaries is explicit and test-validated.** Derived `ConvergencePoint.causal_refs` carry source energy event IDs plus terminal execution (and latest numerical-status if present), and confidence metadata is marked `Completeness::Derived { from_elements }`, preserving traceability without synthetic certainty.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/gromacs_adapter.rs`; `research/trace-semantics/prototypes/lel-ir-prototype/src/adapter.rs`; tests below.

6. **Prototype quality gates remain clean after Session 19 changes.** New tests cover convergence detection, oscillation detection, insufficient-data suppression, and provenance references for both GROMACS and OpenMM paths. Full suite now passes 100/100 with strict clippy clean.  
   Evidence: `research/trace-semantics/prototypes/lel-ir-prototype/src/tests/mod.rs`; `cargo test`; `cargo clippy -- -D warnings`.

**Implications:**

- **WDK#43 is resolved in prototype scope,** reducing uncertainty from representation-only discussion to executable derivation rules with reproducible tests.
- **WDK#42 is narrowed:** taxonomy now has a concrete implemented baseline (converged/stall/oscillation via metric-name conventions), so remaining work is cross-framework naming/confidence harmonization rather than first-principles design.
- **WDK#44 is narrowed:** prototype placement decision is now explicit (per-adapter inline derivation during parsing, with event-ID provenance anchors). Remaining question is whether Stage 1->2 post-pass placement offers better long-run determinism/indexing tradeoffs.

**Open Threads:**

1. **WDK#42 (narrowed):** finalize cross-framework `ConvergencePattern` naming + confidence semantics that align VASP-native SCF markers with derived GROMACS/OpenMM summaries.
2. **WDK#44 (narrowed):** evaluate whether convergence-summary derivation should remain adapter-inline or migrate to Stage 1->2 overlay/post-pass for production determinism and indexing strategy.
3. **OpenMM production path:** validate the same derivation rules against real reporter streams (not just mock parser input) and measure any instrumentation overhead.
4. **Bead hygiene flag:** `athena-fom` appears potentially stale (branch-protection enforcement proof already established elsewhere); defer closure/investigation to a dedicated cleanup session per scope guard.

### 2026-02-21: WDK#26 — INCAR Classification Table Completeness
**Date:** 2026-02-21
**Scope:** Validate completeness of the INCAR parameter classification table; assess ambiguous parameters; propose classification strategy
**Method:** (1) Enumerated all INCAR parameters currently classified in `vasp_adapter.rs:12-96`, (2) cross-referenced against the ambiguous parameter catalog in `cross-framework-synthesis.md §2.3` and `vasp-trace-analysis.md §2.3`, (3) assessed each ambiguous parameter for conditions under which it shifts between theory-affecting and implementation-only, (4) identified additional ambiguous parameters not yet cataloged, (5) evaluated classification strategies against ATHENA's adapter contract requirements.

**Findings:**

**1. Current prototype classification coverage**

The `classify_incar_parameter()` function in `vasp_adapter.rs:12-96` explicitly classifies 16 parameters across four layers:
- **Theory (PrimaryLayer):** GGA, METAGGA, ISMEAR [PROVEN, `vasp_adapter.rs:19-23`]
- **Theory (DualAnnotated):** ENCUT (secondary: Implementation), PREC (secondary: Implementation), SIGMA (secondary: Methodology) [PROVEN, `vasp_adapter.rs:24-51`]
- **Methodology (PrimaryLayer):** IBRION, NSW, ISIF, POTIM, EDIFF, EDIFFG [PROVEN, `vasp_adapter.rs:52-66`]
- **Implementation (PrimaryLayer):** NCORE, KPAR, NPAR, NSIM, NELM [PROVEN, `vasp_adapter.rs:67-70`]
- **Implementation (DualAnnotated):** ALGO (secondary: Methodology), LREAL (secondary: Theory) [PROVEN, `vasp_adapter.rs:72-87`]
- **Fallback:** All unrecognized parameters default to `Implementation` with `ContextDependent` classification and a note "VASP parameter not in classification table" [PROVEN, `vasp_adapter.rs:88-95`]

This covers 16 of approximately 50-80 commonly used parameters (`vasp-trace-analysis.md §2.4`), meaning roughly 65-80% of commonly used parameters fall through to the ContextDependent fallback. [PROVEN by enumeration]

**2. The six identified ambiguous parameters and their conditional behavior**

The cross-framework synthesis (`cross-framework-synthesis.md §2.3`) and VASP trace analysis (`vasp-trace-analysis.md §2.3`) identify six parameters with non-trivial theory-implementation coupling:

**(a) PREC (Accurate / Normal / Low / SinglePrec / High)**
- **When theory-affecting:** Always. PREC sets the ENCUT default (if ENCUT is not explicitly specified), FFT grid density (NGX/NGY/NGZ), and augmentation grid density. These directly determine the numerical accuracy of DFT energies, forces, and stress tensors. Switching from PREC=Normal to PREC=Accurate changes the physics for systems sensitive to basis set completeness or wrap-around errors. [PROVEN, `vasp-trace-analysis.md §2.3`, `cross-framework-synthesis.md:144`]
- **When implementation-only:** Never purely implementation. Even when ENCUT is explicitly set, PREC still controls the FFT grid and augmentation grid independently. [CONJECTURE — the interaction between explicit ENCUT and PREC-controlled grids is documented in VASP Wiki but the precise override behavior requires version-specific testing]
- **Interaction:** PREC sets the default ENCUT. If a user sets ENCUT explicitly, PREC's ENCUT-setting role is overridden but its grid-density role persists. This partial override creates a subtle trap: a user who believes they have controlled for basis set by setting ENCUT explicitly may still have PREC-dependent results via the FFT grid. [CONJECTURE — based on documented VASP behavior, but the magnitude of the FFT-grid-only effect is system-dependent]
- **Current classification:** DualAnnotated(Theory, Implementation) — ADEQUATE. [PROVEN, `vasp_adapter.rs:33-41`]

**(b) ALGO (Normal / Fast / Very_Fast / All / Damped / Conjugate / Subrot / Exact / Nothing / CHI / GW0 / scGW / GW / BSE)**
- **When theory-affecting:** For systems with multiple SCF minima (magnetically frustrated systems, strongly correlated materials with competing ground states, systems near metal-insulator transitions). Different algorithms traverse the SCF energy landscape differently and can converge to different local minima, yielding physically distinct solutions. Estimated 10-20% of systems exhibit this behavior (`cross-framework-synthesis.md:499`). [CONJECTURE — the 10-20% estimate is from community reports, not systematic study]
- **When implementation-only:** For well-behaved systems (single SCF minimum, non-magnetic insulators, simple metals with adequate smearing). All algorithms converge to the same solution; the choice affects only speed and stability. Estimated 80-90% of standard calculations. [CONJECTURE — same source as above]
- **Detection conditions for ambiguity:** ISPIN=2 with non-trivial MAGMOM values, LDAU=.TRUE., small band gap systems (ISMEAR=0 with small SIGMA), systems where SCF convergence is difficult (NELM approached or reached). [CONJECTURE — these are domain heuristics, not formally validated thresholds]
- **Current classification:** DualAnnotated(Implementation, Methodology) — QUESTIONABLE. For pathological systems where ALGO affects solution identity, the secondary layer should be Theory, not Methodology. The current classification does not distinguish "ALGO as solver strategy" from "ALGO as theory-relevant SCF landscape navigator." [PROVEN that the current code uses Methodology as secondary, `vasp_adapter.rs:72-78`; CONJECTURE that Theory would be more appropriate for pathological cases]

**(c) LREAL (Auto / .TRUE. / .FALSE. / On)**
- **When theory-affecting:** For small unit cells (typically fewer than ~20 atoms, though the threshold is element-dependent due to PAW radius variation). Real-space projection introduces controlled approximation errors that can shift forces by several meV/Angstrom. For precision-sensitive properties (phonons, elastic constants, precise relaxation), LREAL=.FALSE. may be required regardless of cell size. [PROVEN for the general mechanism, `vasp-trace-analysis.md §2.3`; CONJECTURE for the ~20 atom threshold — this is a widely cited community heuristic, not a formally derived bound]
- **When implementation-only:** For large cells (typically >~100 atoms), where the real-space projection error is below the convergence threshold of the calculation. [CONJECTURE — same heuristic source]
- **Interaction:** LREAL=Auto lets VASP choose based on cell size. The automatic decision threshold may differ between VASP versions. [CONJECTURE — version-dependent behavior documented but specific threshold changes are not publicly cataloged]
- **Current classification:** DualAnnotated(Implementation, Theory) — ADEQUATE but the primary/secondary ordering is debatable. For small cells, Theory should arguably be primary. The classification should ideally be context-dependent on cell size. [PROVEN for current code, `vasp_adapter.rs:80-87`; CONJECTURE for the context-dependency recommendation]

**(d) ADDGRID (.TRUE. / .FALSE.)**
- **When theory-affecting:** Always, in principle — ADDGRID adds a finer support grid for augmentation charges, improving the accuracy of PAW augmentation integrals. The effect is largest for elements with hard pseudopotentials or small PAW radii. [PROVEN, `cross-framework-synthesis.md:146`]
- **When implementation-only:** Never purely implementation. However, for many systems the accuracy improvement is below the convergence threshold, making it practically irrelevant. [CONJECTURE — system-dependent]
- **Current classification:** NOT CLASSIFIED in `vasp_adapter.rs`. Falls through to the ContextDependent fallback. [PROVEN, `vasp_adapter.rs:88-95`] This is a gap — ADDGRID should be DualAnnotated(Theory, Implementation) based on the cross-framework synthesis assessment. [CONJECTURE for the recommended classification]

**(e) ENCUT (explicit setting)**
- **When theory-affecting:** Always. ENCUT determines basis set completeness — it is the single most important convergence parameter in plane-wave DFT. An unconverged ENCUT means the physics is wrong. [PROVEN, `vasp-trace-analysis.md §5.4`]
- **When implementation-only:** Never purely implementation. However, the practical ENCUT value is often a compromise between theory requirements (convergence) and computational budget (memory/time). [PROVEN for the general mechanism, `vasp-trace-analysis.md §2.3`; `cross-framework-synthesis.md:148`]
- **Interaction with PREC:** If ENCUT is not explicitly set, PREC determines it. If ENCUT IS explicitly set, it overrides the PREC default. This interaction means the classification of ENCUT depends on whether it was user-specified or PREC-derived. [CONJECTURE — the adapter cannot currently distinguish user-specified from PREC-derived ENCUT because vasprun.xml reports the resolved value, not the origin]
- **Current classification:** DualAnnotated(Theory, Implementation) — ADEQUATE. [PROVEN, `vasp_adapter.rs:24-32`]

**(f) NBANDS**
- **When theory-affecting:** When NBANDS is too low, unoccupied states needed for the physics (partial occupancy in metals, conduction band states for optical properties, states needed by hybrid functionals or GW) are missing. The calculation completes but produces incorrect results. [PROVEN, `cross-framework-synthesis.md:179`]
- **When implementation-only:** When NBANDS is set above the minimum required for physical correctness, the excess bands are purely computational overhead (more memory, larger eigenvalue problem). [CONJECTURE — domain reasoning]
- **Current classification:** NOT CLASSIFIED in `vasp_adapter.rs`. Falls through to ContextDependent fallback. [PROVEN, `vasp_adapter.rs:88-95`] This is a gap — NBANDS should be DualAnnotated(Theory, Implementation). [CONJECTURE for the recommended classification]

**3. Additional ambiguous parameters NOT yet identified**

Beyond the six parameters above, the following VASP parameters exhibit theory-implementation ambiguity but are not discussed in any existing ATHENA analysis document:

**(g) ISYM (0 / 1 / 2 / 3 / -1)** — Controls symmetry usage. ISYM=0 disables symmetry entirely. For most calculations, symmetry is purely an implementation optimization. However, for symmetry-broken states (Jahn-Teller distortions, magnetic ordering that breaks crystal symmetry), enforcing symmetry can prevent the system from finding the correct lower-symmetry ground state. Usually implementation, but theory-affecting when the ground state has lower symmetry than the initial structure. [CONJECTURE — domain knowledge, not yet documented in ATHENA sources]

**(h) SYMPREC (default: 1e-5)** — Symmetry detection precision. Too tight: symmetry is lost, increasing computation but not affecting physics. Too loose: non-equivalent atoms are treated as equivalent, introducing errors. Usually implementation (detection threshold), but theory-affecting when the chosen precision incorrectly merges or splits symmetry-inequivalent sites. [CONJECTURE — domain knowledge]

**(i) LASPH (.TRUE. / .FALSE.)** — Controls aspherical contributions within PAW spheres. For f-electron systems, transition metals with partially filled d-shells, and systems with GGA+U or meta-GGA functionals, LASPH=.TRUE. is required for correct physics. Theory-affecting for d/f-electron systems, implementation-irrelevant for s/p-electron systems. [CONJECTURE — widely documented in VASP community but not yet in ATHENA sources]

**(j) LMAXMIX (default: depends on elements)** — Maximum l-quantum number for charge density mixer. For d-electron systems, LMAXMIX=4 is required; for f-electron systems, LMAXMIX=6. Incorrect LMAXMIX can cause SCF convergence failure or convergence to wrong electronic state. Usually implementation, but theory-affecting when incorrect values cause convergence to wrong state. [CONJECTURE — VASP Wiki recommendation]

**(k) ENAUG (default: derived from POTCAR)** — Cutoff for augmentation charge grid. Similar to ADDGRID but more explicit. If set too low, PAW augmentation errors increase. Ambiguous: theory (accuracy) vs implementation (memory). [CONJECTURE — domain knowledge]

**(l) NGX / NGY / NGZ (FFT grid dimensions)** — Explicit FFT grid sizes. Usually auto-determined by PREC and ENCUT. When explicitly set, they override the automatic grid. Too small: aliasing errors affect physics. Ambiguous in the same way as PREC but at a lower level. [CONJECTURE — domain knowledge]

**4. Completeness assessment**

- The existing ATHENA analysis identifies 6 ambiguous parameters (PREC, ALGO, LREAL, ADDGRID, ENCUT, NBANDS). [PROVEN, `cross-framework-synthesis.md §2.3`]
- The prototype `classify_incar_parameter()` explicitly handles 4 of these 6 as DualAnnotated (PREC, ALGO, LREAL, ENCUT). [PROVEN, `vasp_adapter.rs:12-96`]
- 2 identified ambiguous parameters (ADDGRID, NBANDS) fall through to the ContextDependent fallback — these are gaps. [PROVEN by code enumeration]
- 6 additional ambiguous parameters not yet identified in ATHENA sources (ISYM, SYMPREC, LASPH, LMAXMIX, ENAUG, NGX/NGY/NGZ). [CONJECTURE — requires domain expert validation]
- The total ambiguous parameter count is therefore approximately 12 (6 known + 6 newly identified), out of approximately 50-80 commonly used parameters. [CONJECTURE for the total; PROVEN for the known-6 count]
- Beyond the ambiguous set, many clearly-theory and clearly-implementation parameters are also missing from the explicit table. Notable gaps: ISPIN, MAGMOM, LDAU/LDAUU/LDAUJ (all pure theory), LWAVE/LCHARG/LELF (all pure implementation), LPLANE/ISTART/ICHARG (all pure implementation). [PROVEN by comparing `vasp-trace-analysis.md` tables against `vasp_adapter.rs`]

**5. Classification strategy assessment**

Four candidate strategies evaluated:

**Strategy A (Static lookup table, current approach):** Necessary but insufficient — cannot handle context-dependent parameters. [CONJECTURE]

**Strategy B (Static table with context-dependent flags) — RECOMMENDED:** Extend the static table to return a `ContextDependency` descriptor alongside the primary classification. The descriptor specifies what input-specification properties determine whether the parameter is theory-affecting in this particular calculation (e.g., for LREAL: cell_atom_count; for ALGO: spin_polarization AND correlation_correction; for LASPH: element_l_character). Keeps the table static and testable; pushes context resolution to a separate module with access to POSCAR/POTCAR/KPOINTS metadata. [CONJECTURE]

**Strategy C (Version-aware lookup):** Useful as an enhancement to Strategy B but not standalone. Version detection from `vasprun.xml <generator>` is reliable but version-specific behavioral differences are not comprehensively documented. [CONJECTURE]

**Strategy D (Decision tree):** Over-engineered for the prototype stage. Strategy B provides equivalent expressive power with better separation of concerns. [CONJECTURE]

**6. Parameter interaction analysis**

- **PREC -> ENCUT (default):** PREC sets ENCUT when not explicitly specified. Adapter cannot distinguish user-specified from PREC-derived ENCUT in vasprun.xml. [CONJECTURE]
- **PREC -> NGX/NGY/NGZ:** PREC controls FFT grid dimensions when not explicitly set. [PROVEN from VASP documentation]
- **ALGO + ISPIN + MAGMOM:** ALGO's theory-relevance depends on competing magnetic states, indicated by ISPIN=2 and specific MAGMOM patterns. [CONJECTURE]
- **LREAL + cell size (from POSCAR):** LREAL's accuracy impact depends on atom count and cell volume from POSCAR, not INCAR. Cross-file context dependency. [PROVEN, `vasp-trace-analysis.md §2.3`]

**Implications:**

- The INCAR classification table in the prototype is a valid starting point but covers only ~20% of commonly used parameters explicitly and misses 2 of 6 identified ambiguous parameters (ADDGRID, NBANDS).
- An additional 6 ambiguous parameters (ISYM, SYMPREC, LASPH, LMAXMIX, ENAUG, NGX/NGY/NGZ) should be added to the ATHENA ambiguous-parameter catalog and to the adapter classification table.
- The classification strategy should evolve from flat static lookup (Strategy A, current) to static table with context-dependent flags (Strategy B), separating static "what conditions matter" knowledge from dynamic "what conditions hold" evaluation.
- The ALGO secondary layer should be reconsidered: for pathological systems, Theory (not Methodology) is the more accurate secondary layer.
- Parameter interactions (PREC->ENCUT, PREC->NGX, ALGO+ISPIN+MAGMOM, LREAL+cell_size) create compound ambiguities requiring cross-parameter and cross-file context.
- The 70-80% confidence estimate for standard VASP calculations from Decision Gate 1 remains plausible: the additional ambiguous parameters primarily affect the same ~20-30% of pathological/precision-sensitive calculations already identified.

**Open Threads:**

1. Empirical validation of the ~20 atom threshold for LREAL across element types and cell sizes.
2. VASP version-specific default changes catalog between VASP 5.x and 6.x for Strategy C implementation.
3. vasprun.xml origin tracking for ENCUT: can the adapter distinguish user-specified from PREC-derived ENCUT?
4. Domain expert review of the 6 newly identified ambiguous parameters (ISYM, SYMPREC, LASPH, LMAXMIX, ENAUG, NGX/NGY/NGZ).
5. Automation potential for classification table construction via LLM-assisted VASP Wiki documentation analysis.
6. NELMIN ambiguity: currently classified as pure Implementation, but may warrant DualAnnotated classification.

---

### 2026-02-21: WDK#25 — VASP Closed-Source Observability Ceiling
**Date:** 2026-02-21
**Scope:** Enumerate VASP failure modes and classify by observability from external outputs; estimate frequency across DFT workflow types; identify which failure modes hit the closed-source ceiling and what partial mitigations exist
**Method:** Systematic cross-referencing of the VASP failure mode taxonomy (16 modes from cross-framework-synthesis.md §3.3), the VASP trace completeness assessment (vasp-trace-analysis.md §7, cross-framework-synthesis.md §4.3), and the three-stage audit mapping (vasp-trace-analysis.md §6.2). Each failure mode is classified into one of three observability tiers: (A) fully observable from external outputs, (B) partially observable with heuristic mitigations, (C) hits the closed-source ceiling. Frequency estimates draw on domain knowledge of DFT workflow distributions across four canonical workflow types, anchored to documented custodian error handler patterns and Materials Project operational experience.

**Findings:**

**1. Failure Mode Observability Classification**

Each of the 16 VASP failure modes from cross-framework-synthesis.md §3.3 is classified by whether fault isolation is achievable from vasprun.xml + OUTCAR + stdout alone.

**Tier A — Fully observable (fault isolation possible from external outputs):**

- V-I1 (memory crash): Non-zero exit code + stdout/stderr crash message. [PROVEN, `vasp-trace-analysis.md §5.3`]
- V-I2 (segfault): Non-zero exit code + stderr SIGSEGV message. [PROVEN, `vasp-trace-analysis.md §5.3`]
- V-I3 (MPI error): Non-zero exit code + MPI error in stderr. [PROVEN, `vasp-trace-analysis.md §5.3`]
- V-I4 (binary/compilation issue): Crash before any output produced. [PROVEN, `vasp-trace-analysis.md §5.3`]
- V-M1 (insufficient NELM): SCF step count = NELM with dE > EDIFF, detectable from vasprun.xml `<scstep>` count per `<calculation>` block. [PROVEN] pymatgen `Vasprun.converged_electronic` implements this check. [`vasp-trace-analysis.md §5.1`]
- V-M2 (insufficient NSW): Ionic step count = NSW with forces > EDIFFG, detectable from vasprun.xml `<calculation>` count + final forces. [PROVEN] pymatgen `Vasprun.converged_ionic` implements this check. [`vasp-trace-analysis.md §5.2`]
- V-M5 (SIGMA too large): Entropy contribution (TOTEN - TOTEN_free) observable from vasprun.xml energy decomposition. [PROVEN] Standard practice: check entropy term exceeds 1 meV/atom. [`vasp-trace-analysis.md §3.1`; `cross-framework-synthesis.md §3.3`]

**Tier B — Partially observable (heuristic mitigations, not definitive):**

- V-M3 (inappropriate ALGO): SCF convergence trajectory shape (oscillation, slow decay, divergence) observable from OSZICAR/vasprun.xml. [PROVEN] However, distinguishing "wrong ALGO" from "the functional cannot describe this system" requires domain knowledge about system pathology. Signal ambiguous in ~10-20% of cases per `cross-framework-synthesis.md §6.3`. [CONJECTURE for the 10-20% estimate]
- V-M4 (wrong ISMEAR): ISMEAR value observable from vasprun.xml `<incar>`; system metallic/insulating character partially inferrable from eigenvalues/DOS. [PROVEN for parameter visibility] [CONJECTURE: automated determination requires electronic structure analysis which can itself be wrong]
- V-T1 (ENCUT too low): ENCUT and POTCAR ENMAX values both present in vasprun.xml. Cross-check (ENCUT >= 1.3 * max(ENMAX)) implementable. [PROVEN] But 1.3x heuristic catches gross inadequacy, not marginal underconvergence. True convergence requires multiple ENCUT values. [`vasp-trace-analysis.md §5.4`; `cross-framework-synthesis.md §6.3`]
- V-T2 (k-point mesh inadequate): K-point specification visible in vasprun.xml `<kpoints>`; cell dimensions visible in `<structure>`. Heuristic check implementable. [PROVEN for data availability] [CONJECTURE: "adequate" depends on system type, geometry, and target property]
- V-A1 (SCF non-convergence, ambiguous origin): SCF convergence trajectory fully observable. [PROVEN] Root cause (ALGO vs functional vs PREC/LREAL) NOT determinable from trace alone. Custodian handles via sequential fixes (increase NELM -> switch ALGO -> increase ENCUT -> switch mixing). [CONJECTURE for custodian cascade effectiveness]
- V-A3 (symmetry error): OUTCAR warnings sometimes present. [PROVEN] Whether this is methodology error vs implementation issue requires comparing input geometry against expected symmetry. [`vasp-trace-analysis.md §5.6`]

**Tier C — Hits the closed-source ceiling (fault isolation NOT possible from external outputs):**

- V-T3 (wrong pseudopotential): POTCAR identity partially present in vasprun.xml. [PROVEN] Whether the chosen POTCAR variant is appropriate requires domain knowledge about semi-core states; VASP provides no signal when a suboptimal variant is used. [CONJECTURE: rule-based lookup is partial mitigation] [`vasp-trace-analysis.md §5.6`; `cross-framework-synthesis.md §3.3`]
- V-T4 (inappropriate XC functional): No signal whatsoever. [PROVEN] The functional choice is visible in `<incar>`, but whether it is appropriate is pure domain knowledge. No heuristic can reliably determine "this functional is wrong for this system." [`vasp-trace-analysis.md §6.3`]
- V-T5 (missing/incorrect DFT+U): LDAU parameters visible in vasprun.xml. [PROVEN] Whether the U value is correct requires comparison against experimental data not in the trace. [CONJECTURE: known-systems lookup covers obvious cases]
- V-A2 (PREC-induced inaccuracy): PREC value visible in vasprun.xml. [PROVEN] Whether PREC matters for the specific system requires running both and comparing. [CONJECTURE: conservative rule (flag PREC != Accurate) catches ~5-10% per `cross-framework-synthesis.md §6.3`]
- Internal numerical issues (FFT aliasing, PAW reconstruction errors, non-deterministic MPI reductions): No external signal. [PROVEN] Fundamentally hidden by closed-source nature. [`vasp-trace-analysis.md §7.1, §7.4`]

**2. Frequency Distribution Across DFT Workflow Types**

[CONJECTURE — frequency estimates based on domain knowledge of DFT practice, anchored to published Materials Project workflow statistics and custodian error handler activation patterns. No primary data source available for exact frequencies.]

- **Bulk metals (~30% of workflows):** ~85-90% Tier A/B observable. Metals are relatively well-behaved for ATHENA. Dominant failures: V-M1 (5-10%), V-M5 (5%), V-I1/I2/I3 (2-5%). Ceiling failures: V-T4 (<2%), V-T3 (1-3%).
- **Surfaces/interfaces (~20%):** ~75-85% Tier A/B observable. More vulnerable to silent theory failures. Dominant: V-M2 (10-15%), V-M1 (10%), V-T2 (5-10%). Ceiling: V-T4 (5%), V-A2 (3-5%).
- **Molecules/clusters (~15%):** ~75-80% Tier A/B observable. Niche VASP use case, more prone to methodology errors. Dominant: V-M4 (10%), V-T2 (5%), V-M1 (5-10%). Ceiling: V-T4 (5-10%), V-T3 (3-5%).
- **Strongly correlated systems (~15%):** ~50-65% Tier A/B observable. **This is where the closed-source ceiling hits hardest.** Dominant: V-A1 (20-30%), V-T5 (10-20%), V-T4 (15-20%), V-M3 (10-15%). Ceiling: V-T4 (15-20%), V-T5 (10-20%), V-A2 (5-10%), internal solver instabilities (5%).
- **Other workflows (phonons, band structure, NEB, AIMD; ~20%):** ~70-80% Tier A/B on average, with significant variation by sub-type.

**3. Aggregate Closed-Source Ceiling Impact**

[CONJECTURE — aggregated from per-workflow estimates, weighted by approximate workflow frequencies]

- **Weighted average: ~70-80% of VASP failure instances allow fault isolation from external outputs alone** (Tier A + Tier B with effective heuristics). Aligns with `cross-framework-synthesis.md §6.3` estimate.
- **~15-25% of failures are partially degraded** — ATHENA can detect the symptom but cannot definitively isolate the root cause layer without additional calculations or domain expert judgment.
- **~5-10% of failures are at or near the hard ceiling** — fault isolation fundamentally impossible from external outputs.
- **The ceiling impact is NOT uniformly distributed.** Concentrates heavily on strongly correlated systems (~35-50% unobservable) and is minimal for simple bulk metals (~10-15%).

**4. Partial Mitigations for Ceiling Failures**

| Mitigation | Failures Addressed | Effectiveness |
|:---|:---|:---|
| Custodian error handler cascade | V-A1, V-M3 | [CONJECTURE] Resolves ~60-70% of V-A1 operationally, but finds a fix, not a diagnosis |
| POTCAR best-practice lookup | V-T3 | [CONJECTURE] Catches ~70-80% for standard applications; fails for novel elements/pressures |
| ENCUT vs. ENMAX cross-check | V-T1 | [PROVEN] Catches gross underconvergence; does not catch marginal cases |
| K-mesh density heuristic | V-T2 | [CONJECTURE] Effective for standard cases; fails for unusual geometries |
| Conservative PREC rule | V-A2 | [CONJECTURE] Overly conservative — false positives in ~40-50% of cases |
| Known-system DFT+U table | V-T5 | [CONJECTURE] Effective for well-studied systems; fails for novel compounds |
| Companion open-source DFT code | V-T4, V-A2, internal issues | [CONJECTURE] Only mitigation for hard-ceiling failures; adds significant cost |

**5. The Key Question Answered**

[CONJECTURE — synthesized from the above analysis]

**For routine DFT workflows (bulk metals, simple surfaces, standard materials): degradation is infrequent (~10-20% of failure instances).** Most failures are implementation crashes (Tier A), methodology errors detectable from convergence metrics (Tier A/B), or theory errors catchable by rule-based cross-checks (Tier B).

**For challenging DFT workflows (strongly correlated systems, novel materials, high-accuracy requirements): degradation is frequent (~35-50% of failure instances).** The dominant failure modes — SCF non-convergence with ambiguous root cause, functional inadequacy, PREC/ALGO sensitivity — are precisely where internal solver state would be needed for definitive diagnosis.

**The degradation is asymmetric in a problematic way for ATHENA's value proposition.** The workflows where fault isolation matters most (challenging systems where researchers struggle to determine why their calculation failed) are exactly the workflows where the closed-source ceiling bites hardest. Easy systems rarely need sophisticated fault isolation; hard systems need it but the ceiling prevents it.

**However, the ceiling does NOT invalidate ATHENA's approach for VASP.** Even 70-80% fault isolation coverage significantly exceeds the current state of practice, where most researchers perform no systematic fault isolation at all.

**Implications:**

1. ATHENA's VASP support should be marketed with honest scope boundaries. Standard workflows: useful fault isolation. Strongly correlated / pathological systems: symptom detection and heuristic classification, not guaranteed correct layer assignment.
2. The differential-diagnosis approach (custodian-style cascade) is more appropriate than single-pass classification for ceiling-hitting failures.
3. Supporting at least one open-source DFT code (Quantum ESPRESSO or GPAW) is a functional requirement for validating VASP fault classifications, not merely a generalizability test.
4. WDK#25 should be NARROWED, not RESOLVED. The qualitative analysis bounds the ceiling impact but frequency estimates need empirical validation.

**Open Threads:**

1. Empirical validation of frequency estimates against real VASP failure corpora (e.g., Materials Project job logs, custodian error statistics).
2. VASP 6 REPORT file analysis (WDK#7): may partially close the ceiling for SCF solver internals.
3. Custodian error handler coverage quantification: diagnostic accuracy across the 16 failure modes.
4. Whether differential-diagnosis cascades can be formalized as ATHENA fault-isolation strategies (connects to adversarial-reward track).
5. Cross-validation protocol design for companion open-source DFT code.

---

### 2026-02-21: WDK#39 — prediction_id Type Harmonization
**Date:** 2026-02-21
**Scope:** Document String vs SpecElementId type mismatch in prediction tracking; evaluate harmonization options
**Method:** Traced `prediction_id` through its full lifecycle across `event_kinds.rs`, `common.rs`, `overlay.rs`, and all three adapter files (`adapter.rs`, `gromacs_adapter.rs`, `vasp_adapter.rs`). Examined the parse-at-query-time workaround in `compare_predictions`, reviewed test cases for failure mode coverage, and cross-referenced the Step 12 ComparisonProfileV1 interface contract to assess forward-compatibility constraints.

**Findings:**

1. **The type mismatch is precisely located.** `EventKind::ComparisonResult.prediction_id` is `String` (`event_kinds.rs:88`), while `PredictionRecord.id` is `SpecElementId(u64)` (`common.rs:13,104`). The `PredictionComparison` output struct uses `Option<SpecElementId>` (`overlay.rs:36`), meaning the conversion from `String` to typed ID happens at query time in `compare_predictions`. [PROVEN: `event_kinds.rs:88`, `common.rs:13,104`, `overlay.rs:36,261`]

2. **The parse-at-query-time workaround has four identifiable failure modes.**
   (a) **Silent mismatch on non-numeric strings.** If `prediction_id` is a non-numeric string (e.g., `"pred-energy-1"`), the `parse::<u64>()` at `overlay.rs:261` returns `None`, the prediction is unresolvable, and the comparison emits `prediction_id: None, variable: "unknown"`. No error is raised — the mismatch is silently absorbed. [PROVEN: `overlay.rs:261-264`; test `test_compare_predictions_unresolvable_prediction_id` at `tests/mod.rs:1476` validates this behavior]
   (b) **Numeric but non-existent ID.** If the string parses to a valid `u64` but no `PredictionRecord` exists with that ID, the join against `predictions_by_id` at `overlay.rs:263` produces `None`, and the comparison falls back to `variable: "unknown"` while still carrying `prediction_id: Some(SpecElementId(n))`. This creates a partial-success state where the ID looks resolved but carries no semantic meaning. [PROVEN: `overlay.rs:262-264`; no test currently covers this specific case]
   (c) **Leading-zero and whitespace sensitivity.** `String::parse::<u64>()` rejects leading/trailing whitespace; leading zeros do not cause mismatch. Adapter-side whitespace in the prediction_id string would cause silent fallback. [CONJECTURE: inferred from Rust stdlib `str::parse` behavior; no adapter currently emits whitespace]
   (d) **Overflow on large numeric strings.** If an adapter emits a prediction_id string exceeding `u64::MAX`, the parse silently returns `None`. [CONJECTURE: no adapter currently emits such values]

3. **No existing adapter currently emits `ComparisonResult` events.** All three adapter implementations produce only Stage 1/Stage 2 event types. The only code constructing `ComparisonResult` events is in test helpers (`tests/mod.rs:1380,1441,1505,1589,1607,1671`). [PROVEN: exhaustive search across all three adapter files]

4. **ComparisonResult events are architecturally "derived" (R17), not adapter-emitted.** Per `event_kinds.rs:86` "(derived)" annotation and FINDINGS.md finding #51, ComparisonResult events are generated by the IR's comparison logic, not parsed from DSL traces. Adapter impact of changing the field type is zero. [PROVEN: `event_kinds.rs:86`; FINDINGS.md #51]

5. **The Step 12 ComparisonProfileV1 interface contract already specifies `prediction_id: SpecElementId`.** The current `EventKind::ComparisonResult.prediction_id: String` is already inconsistent with the forward design intent. [PROVEN: FINDINGS.md lines 327, 324-333]

6. **Evaluation of harmonization options:**

   **(a) Change `ComparisonResult.prediction_id` to `SpecElementId` — RECOMMENDED.** Eliminates parse-at-query-time workaround. Aligns with `PredictionRecord`, `ContractTerm`, `InterventionRecord`, and `ControlledVariable` — all use `SpecElementId` for their `id` field (`common.rs:96,104,113,121`). Aligns with ComparisonProfileV1 forward design. Zero adapter impact since no adapter constructs `ComparisonResult` (finding #3). Only breakage: 6 test construction sites. Risk: LOW. [PROVEN for adapter impact; CONJECTURE for production-scope assessment]

   **(b) Change `PredictionRecord.id` to `String` — NOT RECOMMENDED.** Weakens type safety across the entire spec system. All four spec element types use `SpecElementId`. Loses `Hash`/`Copy`/`Eq` guarantees, complicating HashMap-based lookups. Diverges from ComparisonProfileV1. [PROVEN: four spec element types use `SpecElementId`]

   **(c) Introduce a shared `PredictionId` newtype — VIABLE BUT PREMATURE.** Decouples prediction identity from `SpecElementId` space. But adds a new type for a single-use case with no demonstrated need for different semantics. Currently all four spec element types share the same ID space — introducing a separate type only for predictions breaks uniformity without motivation. [CONJECTURE: assessment of future need]

7. **`SpecElementId` is a newtype wrapper around `u64` (`common.rs:12-13`), not an enum.** No variant extension needed for harmonization. [PROVEN: `common.rs:12-13`]

**Implications:**

- Option (a) is the clear recommendation: eliminates workaround, aligns with four other spec element types, matches ComparisonProfileV1, zero adapter impact since `ComparisonResult` is a derived event type.
- The change is small (1 type change, 1 method simplification, 6 test updates) but flagged as ADR-scoped work, so can remain documented until production design decisions.
- The `String` type was likely a placeholder from before the spec element ID system was fully established, not a deliberate design choice.
- If production requires non-numeric prediction IDs (multi-experiment namespacing), option (c) becomes relevant, but `SpecElementId` would need to evolve for all spec element types.

**Open Threads:**

- The comparison engine that will construct `ComparisonResult` events in non-test contexts does not yet exist. When built, it should construct `prediction_id` as `SpecElementId` directly.
- Finding 2b (numeric ID with no matching `PredictionRecord`) is not tested. A test for this case would clarify whether the fallback behavior is intentional or a gap.
- `TraceEvent.spec_ref: Option<SpecElementId>` (`lel.rs:78`) provides a general-purpose spec element link at the event level, which could serve as an alternative to carrying `prediction_id` inside `ComparisonResult`'s payload. This payload-level vs event-level spec linkage question is broader than WDK#39.

---

### 2026-02-21: WDK#35 + WDK#36 — ContractTerm Value Extensions for VASP
**Date:** 2026-02-21
**Scope:** Investigate what VASP preconditions need machine-readable values in ContractTerm and what Value variants are needed for spectral/grid data
**Method:** Systematic code analysis of `common.rs` (ContractTerm, Value, PredictionRecord), `vasp_adapter.rs` (current usage), `lel.rs` (ExperimentSpec), and `event_kinds.rs` (EventKind consumers of Value). Cross-referenced against VASP domain knowledge from `dsl-evaluation/vasp-trace-analysis.md` and `dsl-evaluation/cross-framework-synthesis.md`. Steel-manned each design option before stress-testing.

**Findings:**

**WDK#35 — ContractTerm `value` field:**

1. [PROVEN] The current `ContractTerm` struct (`common.rs:94-99`) has only three fields: `id: SpecElementId`, `description: String`, `layer: Layer`. The `description` field is the sole carrier of precondition semantics, making machine-readable checking impossible. [`common.rs:94-99`; test usage at `tests/mod.rs:33-36`]

2. [PROVEN] The VASP adapter (`vasp_adapter.rs:592-594`) currently sets both `preconditions` and `postconditions` to empty `Vec::new()`. The same is true for GROMACS (`gromacs_adapter.rs:632-634`) and MockOpenMM adapters. ContractTerm is not exercised in any adapter path. [`vasp_adapter.rs:592-594`]

3. [PROVEN] Other spec-level types already carry machine-readable `Value` fields: `PredictionRecord.predicted_value: Value` (`common.rs:106`), `InterventionRecord.values: Vec<Value>` (`common.rs:115`), `ControlledVariable.held_value: Value` (`common.rs:123`). ContractTerm is the sole spec-level type lacking a value field. [`common.rs:94-123`]

4. [CONJECTURE — domain knowledge] VASP preconditions that require machine-readable values fall into at least five concrete categories:
   - **POTCAR family/identity**: Categorical match (`Value::KnownCat("PBE")`). [`vasp-trace-analysis.md` lines 92, 346-347]
   - **ENCUT vs POTCAR ENMAX threshold**: Numeric comparison (`Value::Known(f64, "eV")`). [`vasp-trace-analysis.md` lines 354-361]
   - **KPOINTS mesh density**: Vector value for mesh grid. [`vasp-trace-analysis.md` lines 363-373]
   - **ISMEAR appropriateness**: Categorical precondition. [`vasp-trace-analysis.md` lines 83-84]
   - **POSCAR atom count / species consistency**: Structural match. [`cross-framework-synthesis.md` line 434]

5. [CONJECTURE — design analysis] Three design options evaluated:
   - **Option A (`value: Option<Value>`) — RECOMMENDED for prototype.** Minimal, backward-compatible, consistent with `ControlledVariable.held_value` pattern. The checking logic is domain-specific and belongs in adapters/LFI, not the IR schema. All five precondition categories representable with existing Value variants.
   - **Option B (typed predicate `ContractCheck`)**: Machine-checkable from IR but significantly more complex; operator set would grow per domain.
   - **Option C (hybrid `value` + `check_rule: Option<String>`)**: Introduces string-keyed dispatch.

   Option A is correct because the IR's job is to carry the expected value so the comparison is recordable; the comparison logic lives in `compare_predictions` (`overlay.rs:230-277`), not in the schema.

**WDK#36 — Value enum KnownMatrix/grid variant:**

6. [PROVEN] The current Value enum (`common.rs:201-213`) has four variants: `Known(f64, Unit)`, `KnownVec(Vec<f64>, Unit)`, `KnownCat(String)`, `Havoc { expected_type, reason }`. None can represent 2D or higher-dimensional data. [`common.rs:201-213`]

7. [PROVEN] `PredictionRecord.predicted_value: Value` (`common.rs:106`) is the primary consumer needing matrix/grid values for spectral predictions. `ObservableMeasurement.value: Value` (`event_kinds.rs:69`) and `ObservableMeasurement.uncertainty: Option<Value>` (`event_kinds.rs:70`) are secondary consumers. [`common.rs:102-108`, `event_kinds.rs:66-73`]

8. [PROVEN from domain sources] VASP produces at least five distinct spectral/grid data types not representable by current Value variants:
   - **Band structure (EIGENVAL)**: `[n_kpoints][n_bands][n_spin]` of f64 in eV. [`vasp-trace-analysis.md:29`]
   - **Total DOS (DOSCAR)**: `[n_energy_points][2+]` of f64 with units (eV, states/eV). [`vasp-trace-analysis.md:30`]
   - **Projected DOS/bands (PROCAR)**: `[n_kpoints][n_bands][n_ions][n_orbitals]` of f64, up to 1 GB. [`vasp-trace-analysis.md:31`]
   - **Charge density (CHGCAR)**: `[nx][ny][nz]` of f64, 10 MB to 10 GB. [`vasp-trace-analysis.md:32`]
   - **Local potential (LOCPOT)**: Same shape as CHGCAR. [`vasp-trace-analysis.md:34`]

9. [CONJECTURE — design analysis] Four design options evaluated. **Recommended: Add two variants — `KnownGrid` for inline spectral data and `DataRef` for volumetric references.**

   ```
   KnownGrid {
       axes: Vec<(String, Vec<f64>, Unit)>,  // (label, values, unit) per axis
       values: Vec<f64>,                      // flattened row-major data
       value_unit: Unit,                      // unit of the values
   },
   DataRef {
       path: String,                          // reference to external data
       data_type: String,                     // e.g., "charge_density"
       shape: Vec<usize>,                     // shape metadata
       unit: Unit,
   },
   ```

   This follows the existing pattern where small data is inline (`Known`, `KnownVec`) and large data is referenced (`StateSnapshot.data_ref`). Band structure/DOS fit KnownGrid; CHGCAR/LOCPOT/PROCAR fit DataRef.

10. [CONJECTURE — cross-framework] OpenMM and GROMACS do not produce spectral data. These extensions are VASP-specific within the current three-framework scope but would serve future DFT adapters (Quantum ESPRESSO, CASTEP, CP2K). [`cross-framework-synthesis.md` line 437]

**Implications:**

- WDK#35 can be closed with `value: Option<Value>`. All five VASP precondition categories are representable with existing Value variants. No new Value types needed for WDK#35.
- WDK#36 requires two new Value variants: `KnownGrid` (inline spectral data) and `DataRef` (volumetric references). `ValueType` enum (`common.rs:24-28`) needs corresponding `Grid` and `DataRef` variants.
- Neither change is blocking. Both are VASP Stage 3 features. `Havoc` variant can serve as placeholder until implemented.
- Step 12 ComparisonProfileV1 is affected: spectral data comparison requires profile-level extensions (e.g., `SpectralDivergence` variant). Downstream dependency, not a blocker.

**Open Threads:**

- Axis convention for KnownGrid: canonical ordering per data_type in adapter conventions, not in the IR schema.
- DataRef resolution mechanism: how does the IR consumer locate external files? Same unresolved question as `StateSnapshot.data_ref`.
- Spectral comparison for R17: distance metrics for band structures / DOS curves not covered by current `DivergenceMeasure`. Defers to adversarial-reward track.
- VASP adapter Stage 2/3 implementation: once ContractTerm.value and new Value variants exist, the adapter needs to populate preconditions and predictions.

---

### 2026-02-22: Step 14 — UncertaintySummary Schema for Divergence Metrics (WDK#40)

**Scope:** Resolve/narrow What We Don't Know #40 ("What minimal `UncertaintySummary` schema should accompany each divergence metric so one comparison profile can support both V&V/effect-size reporting and Bayesian/active-learning reward calibration without adapter-specific branching"), while preserving Step 12 `ComparisonProfileV1` guarantees (G1-G5) and Step 13 convergence-summary compatibility.

**Method:**  
1. External survey (scoped) of uncertainty representation patterns in ASME V&V/VVUQ pages, ArviZ `summary`, SALib Sobol analysis outputs, and UQpy inference/sampling/distribution docs.  
2. Six-consumer trace (LFI Stage 3, BSE post-experiment, BSE pre-experiment type-compatibility, Adversarial Experiment Designer calibration, Mode Controller, ConvergenceSummary) against Candidates A/B/C.  
3. Cross-adapter feasibility pass using current prototype outputs for VASP/GROMACS/OpenMM adapters.  
4. Steel-man/stress-test on the three schema candidates against priority criteria (Primary: consumer coverage, G5 compliance, cross-adapter feasibility; Secondary: minimality, type safety, extensibility; Tertiary: information preservation).  
5. Recommended type specification with explicit divergence-kind interaction and `ConvergenceSummary` relationship.

**Findings:**

1. **[PROVEN] The current prototype has no measurement-uncertainty type on divergence outputs; only scalar divergence values are carried through Stage 3 comparison paths.**  
Mechanism: `ComparisonOutcome` stores `agreement + Option<DivergenceMeasure> + detail`; each `DivergenceMeasure` variant is scalar `f64`; overlay query logic forwards this outcome unchanged into `PredictionComparison`.  
Conditions: this is sufficient for falsification routing but insufficient for uncertainty-aware calibration (Step 12 gap).  
Evidence: `prototypes/lel-ir-prototype/src/common.rs:134-150`; `prototypes/lel-ir-prototype/src/overlay.rs:230-277`; Step 12 contract block (`ComparisonProfileV1`) at `FINDINGS.md:200-227`.

2. **[PROVEN] Existing `ConfidenceMeta` is data-completeness metadata, not measurement uncertainty, and cannot substitute for `UncertaintySummary`.**  
Mechanism: `ConfidenceMeta` encodes completeness/inference provenance (`FullyObserved`, `PartiallyInferred`, etc.) and field coverage; it does not encode standard errors, intervals, distributional shape, or posterior sample summaries.  
Evidence: `prototypes/lel-ir-prototype/src/common.rs:266-280`.

3. **[PROVEN FROM EXTERNAL SOURCES + CONJECTURE BOUNDARY] Across surveyed ecosystems, the dominant pattern is layered: a point-level summary plus optional richer uncertainty/distributional payload, closest to Candidate C (often with an inner tagged payload resembling B).**  
Observed pattern mapping:

| External schema | Observed uncertainty shape | Closest candidate |
| :--- | :--- | :--- |
| ASME V&V 20 product description + ASME VVUQ 10.2 description | Validation-comparison accuracy quantified using errors/uncertainties in simulation and data; explicit inclusion of model-form/numerical/input uncertainties and validation metrics including uncertainty. | C |
| ArviZ `az.summary()` | Fixed point/interval/diagnostic columns (`mean/sd/hdi/mcse/ess/r_hat`) in one summary table; optional formatting for diagnostics vs stats. | C |
| SALib Sobol | Point sensitivity indices plus paired confidence outputs (`S1` with `S1_conf`, `ST` with `ST_conf`, optional `S2` with `S2_conf`). | C (with B-like typed subkeys) |
| UQpy inference/sampling/distributions | Posterior represented via sampler samples (tensor-shaped arrays) with distribution objects/methods (`pdf/cdf/log_pdf/rvs`) and downstream summarization. | C with tagged distribution payload |

Evidence: ASME V&V 20 page (`https://www.asme.org/codes-standards/find-codes-standards/standard-for-verification-and-validation-in-computational-fluid-dynamics-and-heat-transfer`), ASME V&V 10 page (`https://www.asme.org/codes-standards/find-codes-standards/standard-for-verification-and-validation-in-computational-solid-mechanics`), ASME VVUQ 10.2 page (`https://www.asme.org/codes-standards/find-codes-standards/the-role-of-uncertainty-quantification-in-verification-and-validation-of-computational-solid-mechanics-models`), ASME V&V 20 press-release description (`https://www.asme.org/about-asme/media-inquiries/press-releases/asme-announces-a-new-standard-for-verification-and`), ArviZ docs (`https://python.arviz.org/en/v0.21.0/api/generated/arviz.summary.html`), SALib basics/API (`https://salib.readthedocs.io/en/stable/user_guide/basics.html`, `https://salib.readthedocs.io/en/latest/api.html`), UQpy docs (`https://uqpyproject.readthedocs.io/en/stable/inference/bayes_parameter_estimation.html`, `https://uqpyproject.readthedocs.io/en/latest/sampling/mcmc/index.html`, `https://uqpyproject.readthedocs.io/en/latest/distributions/`).  
Conjecture boundary: full ASME standards are paywalled; tolerance-band/acceptance-detail claims should be verified directly against purchased standard text before production ADRs.

4. **[PROVEN REASONING] Candidate A (flat optional struct) is minimal in syntax but weak on machine-checkable omission semantics (G5) and high on combinatorial ambiguity.**  
Steel-man: easiest incremental fit with current prototype optional patterns.  
Stress-test: consumers cannot distinguish "not provided" from "not applicable" without extra conventions; presence/absence combinations become combinatorial; predicted-vs-actual calibration comparisons require brittle field-presence logic.  
Evidence: Step 12 G5 requirement (`FINDINGS.md:221-227`) plus current optionality-heavy prototype patterns (`event_kinds.rs:65-83`, `common.rs:266-280`).

5. **[PROVEN REASONING] Candidate B (single tagged union by uncertainty regime) improves type safety and explicit NoUncertainty encoding, but loses information when point and distributional representations coexist in one metric.**  
Steel-man: strong variant-level validity guarantees; explicit `NoUncertainty` is machine-checkable for G5.  
Stress-test: single-variant selection forces adapter or derivation layer to choose one regime when both point summary and empirical/parametric payload are available; this creates avoidable information loss for dual consumers (point-only + distributional).  
Conditions: holds under the Step 4 definition where variants are mutually exclusive.  
Evidence: Step 12 contract requires support for both point consumers and posterior-aware consumers (`FINDINGS.md:200-227`); external patterns above usually expose both summaries and richer payloads.

6. **[PROVEN] Candidate C (always point summary + optional distribution payload) is the only candidate that satisfies all six consumers without adapter-specific branching and without forcing scalar/distributional trade-off.**  
Mechanism: point-only consumers read the always-present point layer; distributional consumers read payload when available, otherwise point fallback with explicit omission semantics.  
Evidence: architecture consumer responsibilities (`ARCHITECTURE.md:111-117`, `:168`, `:206-208`, `:214-243`) and Step 13 convergence compatibility requirement (`FINDINGS.md:92-94`, `:101-114`).

Consumer trace (Need Met / Requires Branching / Information Lost):

| Consumer | Candidate A | Candidate B | Candidate C |
| :--- | :--- | :--- | :--- |
| (a) LFI Stage 3 (`std_error` or CI + `sample_size`) | Y / Y / Medium (missingness ambiguous) | Y / Y / Low-Medium (variant mismatch risk) | Y / N / Low |
| (b) BSE post-experiment (KL calibration metadata) | Y / Y / Medium-High (field ambiguity) | Y / Y / Medium (single-variant tradeoff) | Y / Y (payload-only) / Low |
| (c) BSE pre-experiment type-compatibility (predicted vs actual) | N / Y / High | Y / Y / Medium | Y / Y (payload-only) / Low |
| (d) Adversarial Designer predicted-vs-actual gain calibration | Y / Y / Medium | Y / Y / Medium | Y / Y (payload-only) / Low |
| (e) Mode Controller convergence/confidence trend inputs | Y / Y / Medium | Y / Y / Medium | Y / N / Low |
| (f) ConvergenceSummary residual uncertainty + pattern confidence hook | Y / Y / Medium | N / Y / High | Y / Y (payload-only) / Low |

7. **[PROVEN] Cross-adapter feasibility favors Candidate C because all adapters can populate the point layer now, while distribution payload remains optional and reporter/trace richness dependent.**  
VASP: currently emits SCF convergence points (`dE`), energy records, and execution status; supports point uncertainty baselines (residual, iteration count, CI only if derivable).  
GROMACS: emits repeated energy records and numerical statuses from logs; supports point baselines now and empirical interval/bootstrap-style payloads when sufficient sampling windows are available.  
OpenMM (current prototype path): only mock reporter events (`ParameterRecord`, `ResourceStatus`, `EnergyRecord`, `ExecutionStatus`); distribution payload is reporter-dependent and cannot be assumed.  
Evidence: `prototypes/lel-ir-prototype/src/vasp_adapter.rs:175-276`, `:488-618`; `prototypes/lel-ir-prototype/src/gromacs_adapter.rs:258-530`, `:532-655`; `prototypes/lel-ir-prototype/src/adapter.rs:40-197`; Step 13 Findings 1-3.

8. **[PROVEN] Recommended minimal schema is a layered Candidate C with an inner tagged distribution payload.**

Proposed type (conceptual, findings-level):

```text
UncertaintySummary {
  point: PointUncertainty                 // always present
  distribution: Option<DistributionPayload> // optional richer payload
}

PointUncertainty =
  | Summary {
      sample_size: Option<u32>
      standard_error: Option<f64>
      interval: Option<IntervalEstimate>   // bounds + confidence/credibility level + sidedness
      method_ref: String                   // estimator/CI method identifier
    }
  | NoUncertainty {
      reason: UncertaintyUnavailableReason // adapter_cannot_observe | insufficient_samples | not_computed | invalidated
    }

DistributionPayload =
  | Parametric { family: String, parameters: Vec<NamedValue>, support: Option<Support> }
  | Empirical  { quantiles: Vec<QuantilePoint>, sample_count: Option<u32> }
  | BoundedInterval { lower: f64, upper: f64, coverage: Option<f64>, basis: String }
```

Field justification:
- `sample_size`, `standard_error`, `interval`: required by LFI Stage 3 quality checks and effect-size/V&V consumers.  
- `method_ref`: required for auditability/calibration diagnosis (G4).  
- `NoUncertainty.reason`: required for G5 explicit omission semantics.  
- `DistributionPayload` variants: required for BSE/adversarial predicted-vs-actual calibration compatibility without adapter branching.

9. **[PROVEN] Interaction with `DivergenceKind` should be non-redundant: `DivergenceKind` encodes metric semantics; `UncertaintySummary` encodes uncertainty of estimating that metric value.**  
Implication: do not duplicate Bayes-factor/KL semantic payloads in `UncertaintySummary`; only include estimation uncertainty (e.g., MCSE, interval, empirical quantiles, bounded ranges) around `MetricComponent.value`.  
Evidence: Step 12 `MetricComponent` contract (`FINDINGS.md:210-218`).

10. **[PROVEN + CONSTRAINED DESIGN INFERENCE] `MetricComponent.uncertainty` and `ConvergenceSummary.uncertainty` should share the same numeric `UncertaintySummary` core, while convergence-pattern confidence remains a separate field until WDK#42 is resolved.**  
Mechanism: residual/rate uncertainty uses the same numeric schema; categorical pattern-confidence taxonomy is a separate unresolved question (WDK#42-44).  
Evidence: Step 13 recommended `ConvergenceSummary` fields (`FINDINGS.md:101-114`) and open WDK#42-44 definitions (`FINDINGS.md:1348-1352`).

11. **[PROVEN] No prototype code changes are warranted in Step 14.**  
Rationale: WDK#40 is narrowed analytically with a concrete schema recommendation; implementing types before WDK#41 aggregation and WDK#42-44 convergence-taxonomy decisions would risk rework.  
Evidence: Step 12/13 scope boundaries and no-code-change precedent (`FINDINGS.md:20`, `:1379-1381`).

**Implications:**  
- **WDK#40 is NARROWED to a concrete schema direction:** Candidate C with mandatory point layer + optional tagged distribution payload + explicit `NoUncertainty` reason.  
- **ComparisonProfileV1 integration is now explicit:** `MetricComponent.uncertainty` can satisfy both V&V/effect-size reporting and Bayesian calibration consumers without adapter-specific branching.  
- **Adversarial calibration compatibility is preserved:** predicted-vs-actual gain comparisons can use a type-compatible uncertainty container across pre/post experiment paths.  
- **Prototype impact:** deferred by design; findings-level contract is sufficient at this stage.

**Open Threads:**  
- WDK#41: scalar aggregation from multi-metric uncertainty-aware profiles remains unresolved (connection only).  
- WDK#42-44: convergence pattern taxonomy/derivation/timing remain unresolved; only numeric uncertainty core is settled here.  
- Field-level canonicalization still needed before implementation (e.g., required quantile set, `method_ref` registry, interval-sidedness enum).

---

### 2026-02-22: Step 13 — Convergence Trajectory Representation (WDK#13)

**Scope:** Resolve/narrow What We Don't Know #13 ("How convergence trajectories should be represented in the IR") with explicit evidence from the current prototype, steel-man/stress-test of Options A/B/C, hybrid Option D analysis, external library survey (pymatgen/ASE/Custodian), and downstream consumer trace against Step 12 `ComparisonProfileV1`.

**Method:**  
1. Empirical code inventory of convergence event usage and indexing across adapters/types (`event_kinds.rs`, `vasp_adapter.rs`, `gromacs_adapter.rs`, `adapter.rs`, `lel.rs`, tests).  
2. Steel-man then stress-test for Option A (Raw Time Series), Option B (Classified Patterns), Option C (Derived Features), with explicit mechanism + conditions for failure.  
3. Hybrid Option D design pass: evaluate candidate `ConvergenceSummary` fields against consumer needs and five design tensions (cross-framework asymmetry, silent-failure detection, R17 interface, WDK#40, adapter burden).  
4. Scoped external survey focused on representation choices: pymatgen, ASE, Custodian docs.  
5. Consumer trace mapping from convergence evidence to LFI Stage 1/2/3, Bayesian Surprise Evaluator, and Step 12 `ComparisonProfileV1`.

**Findings:**

1. **[PROVEN] `ConvergencePoint` is currently a VASP-only raw trajectory event in the prototype.**  
Mechanism: VASP `parse_oszicar` emits one `ConvergencePoint` per `DAV:`/`RMM:` SCF line (`metric_name: "dE"`, `converged: None`) and back-patches the most recent one to `Some(true)` when an `F=` line is parsed.  
Conditions: this behavior appears only in OSZICAR parsing path, not in shared adapter logic.  
Evidence: `prototypes/lel-ir-prototype/src/event_kinds.rs:95-101`; `prototypes/lel-ir-prototype/src/vasp_adapter.rs:175-276`, especially `:184-216` and `:219-240`.

2. **[PROVEN] GROMACS and OpenMM adapter paths do not emit `ConvergencePoint`; convergence-like signals are represented asymmetrically.**  
Mechanism: GROMACS emits `EnergyRecord`, `NumericalStatus`, and terminal `ExecutionStatus`; OpenMM mock emits `ParameterRecord`/`ResourceStatus`/`EnergyRecord`/`ExecutionStatus` only.  
Conditions: asymmetry is structural in current prototype paths, not a temporary index omission.  
Evidence: `prototypes/lel-ir-prototype/src/gromacs_adapter.rs:429-530`; `prototypes/lel-ir-prototype/src/adapter.rs:82-197`; repo grep on `ConvergencePoint` usage is adapter-local to VASP.

3. **[PROVEN] Current indexes do not provide first-class convergence queryability by variable or DAG node.**  
Mechanism: `by_variable` indexes only `ParameterRecord` and `ObservableMeasurement`; `parse_oszicar` does not set `dag_node_ref` for convergence events, so `by_dag_node` has no convergence anchors.  
Conditions: impacts Stage 2/3 graph-linked queries and any consumer wanting direct convergence lookups without full stream scan.  
Evidence: `prototypes/lel-ir-prototype/src/lel.rs:130-141`; `prototypes/lel-ir-prototype/src/lel.rs:145-149`; `prototypes/lel-ir-prototype/src/vasp_adapter.rs:190-211`.

4. **[PROVEN + CONJECTURE] Cardinality can become large enough that raw-only representation is expensive for every downstream consumer to re-interpret.**  
Mechanism: sample fixture has six SCF convergence events (`2` ionic steps x `3` SCF points each); realistic runs can be much larger.  
Conditions: estimate of ~1500 events assumes ~50 ionic steps x ~30 SCF iterations and is a back-of-envelope bound, not yet benchmarked in this repo.  
Evidence: sample data and assertion at `prototypes/lel-ir-prototype/src/tests/mod.rs:3071-3079` and `:3377-3385`.

5. **[PROVEN REASONING] Option A (Raw Time Series) steel-man + stress-test.**  
Steel-man: maximally preserves information; prevents anti-pattern of irreversible lossy compression; already matches implemented VASP path; supports post-hoc detection of silent-failure trajectories not anticipated at parse time.  
Stress-test: (i) cardinality growth and repeated scan cost, (ii) no convergence-specific index support today, (iii) cross-framework asymmetry leaves GROMACS/OpenMM with no equivalent raw convergence stream, (iv) every consumer must duplicate feature/pattern extraction logic.  
Evidence: Findings 1-4 above; `prototypes/lel-ir-prototype/src/lel.rs:130-141`.

6. **[CONJECTURE WITH BOUNDED MECHANISM] Option B (Classified Patterns) steel-man + stress-test.**  
Steel-man: strong compression (order-of-magnitude plausible), direct routing signal for LFI Stage 2 and BSE, and normalization layer across frameworks.  
Stress-test: taxonomy is domain-loaded and incomplete by construction; pattern-first storage is lossy for unseen failure modes; detection logic placement creates either adapter bloat or a new summarization component; confidence scoring is mandatory to avoid false certainty.  
Evidence: architecture need for calibrated/invalidation-aware downstream signals (`ARCHITECTURE.md:198-202`, `:206`, `:113`, `:168`); no primary pattern-first convergence representation found in scoped external survey (Finding 8).

7. **[PROVEN + INFERENCE] Option C (Derived Features Only) steel-man + stress-test.**  
Steel-man: smallest storage and easiest immediate consumer integration; aligns with common library APIs that expose convergence booleans/counters.  
Stress-test: loses trajectory shape (e.g., oscillation, two-plateau, late divergence), making silent-failure mechanisms harder to falsify; feature set hard-codes assumptions and can violate AP5-style anti-loss constraints if raw trace is discarded.  
Evidence: prototype only stores per-point convergence in VASP today (`prototypes/lel-ir-prototype/src/event_kinds.rs:95-101`; `prototypes/lel-ir-prototype/src/vasp_adapter.rs:184-240`); anti-pattern context from candidate schema work (Step 5a) and current WDK#13 framing.

8. **[PROVEN FROM EXTERNAL SOURCES] Ecosystem pattern is layered raw + derived, not classified-pattern-primary.**  
`pymatgen`: the VASP parser exposes derived convergence booleans on `Vasprun` (`converged`, `converged_electronic`, `converged_ionic`), but also keeps step-level trajectories via OSZICAR/Vasprun structures (`electronic_steps`, `ionic_steps`). Mechanistically this is dual representation: lightweight decision flags plus recoverable trajectory detail for diagnosis. This matches Option D's layered direction rather than Option C-only collapse.  
`ASE`: optimizer APIs return a boolean convergence result (`run(...)->bool`) and keep explicit trajectory-writing hooks (`self.attach(self._traj_write_image, ...)` when a trajectory is configured). The representation split is again derived decision signal plus optional full path history for audit/restart.  

`Custodian`: convergence handling is rule-based on raw artifacts, not on a precomputed global pattern class. `NonConvergingErrorHandler` monitors OSZICAR behavior over recent ionic steps, while `UnconvergedErrorHandler` checks convergence from `vasprun.xml`. This is derived-feature operational logic anchored to raw trace files, with no evidence that classified patterns are the canonical store.  
Inference boundary: scoped survey did not identify a mainstream package where convergence-pattern taxonomy is the primary persisted representation.  
Evidence: pymatgen docs (`https://pymatgen.org/pymatgen.io.vasp.html`), especially `Vasprun.converged*` and OSZICAR step lists; ASE optimizer docs/source (`https://ase-lib.org/_modules/ase/optimize/optimize.html`); Custodian handler docs (`https://materialsproject.github.io/custodian/custodian.vasp.handlers.html`).

9. **[PROVEN + CONSTRAINED DESIGN INFERENCE] Consumer trace requires multi-resolution convergence data, not a single abstraction.**  
LFI Stage 1: implementation audit needs fast, derived checks (`converged`, `iteration_count`, and `iteration_count/max_iterations` when available) to decide whether execution satisfied solver-budget and numerical-completion conditions.  
LFI Stage 2: methodological audit needs pattern-level interpretation plus parameter linkage (e.g., convergence behavior versus algorithmic choices) to test whether methodology was adequate versus merely executed.  
LFI Stage 3: theoretical evaluation needs final residual/state summaries (`final_residual`, `converged`) for prediction-observation comparison under R17.  
BSE: requires predicted-vs-observed divergence on convergence outcomes and must honor LFI invalidation semantics, so summary outputs need validity/provenance/uncertainty hooks.  
ComparisonProfileV1 (Step 12 Candidate B): convergence evidence should map to multiple `MetricComponent` entries (status, residual, rate/pattern metrics), not one collapsed scalar, with optional `uncertainty` for WDK#40 compatibility.  
Conditions: this mapping is constrained by architecture roles and Step 12 contract shape (`metrics: Vec<MetricComponent>`, optional `uncertainty`).  
Evidence: LFI stages in `ARCHITECTURE.md:198-202`; BSE role and invalidation in `ARCHITECTURE.md:113` and `:168`; Step 12 contract block in this file (`ComparisonProfileV1`/`MetricComponent`); current prototype comparison types at `prototypes/lel-ir-prototype/src/common.rs:134-150` and `prototypes/lel-ir-prototype/src/overlay.rs:230-277`.

10. **[RECOMMENDATION, NARROWED] Option D (Hybrid: raw events + boundary summary) is the strongest representation under current evidence.**  
Mechanism: keep adapters raw (preserve discovery and silent-failure analyzability), compute `ConvergenceSummary` at Stage 1->2 boundary alongside/after `CausalOverlay::from_log` to supply derived/pattern features once for all consumers.  
Conditions: summary must retain provenance pointers to raw events and represent uncertainty explicitly when available.  
Evidence: overlay boundary location and construction point (`prototypes/lel-ir-prototype/src/overlay.rs:54-84`); Step 12 multi-metric contract and WDK#40 uncertainty requirement (`research/trace-semantics/FINDINGS.md` Step 12 contract block; WDK#40 item).

Proposed `ConvergenceSummary` field decisions (candidate list evaluated against consumer trace):

| Field | Decision | Rationale |
| :--- | :--- | :--- |
| `metric_name`, `scope` | Keep | Needed for cross-framework normalization and metric routing. |
| `iteration_count` | Keep | Stage 1 budget/limit checks. |
| `max_iterations` | Keep as `Option<u64>` | Required for ratio checks, but unavailable in some traces without parameter join. |
| `converged` | Keep | Direct Stage 1/3 signal. |
| `final_residual` | Keep as `Option<Value>` | Stage 3 and BSE need final observed error magnitude when defined. |
| `convergence_rate` | Keep as optional + method reference | Useful but method-dependent; must remain auditable. |
| `pattern`, `pattern_confidence` | Keep as optional | Stage 2/BSE utility with explicit uncertainty; taxonomy remains open. |
| `first_event_id`, `last_event_id`, `event_count` | Keep | Provenance/audit bridge to raw trajectory. |
| `uncertainty: Option<UncertaintySummary>` | Keep (hook only) | Required compatibility path to WDK#40 and Candidate B metric components. |
| `parameter_refs: Vec<EventId>` | **Add** | Required for Stage 2 parameter cross-reference without adapter-side pattern logic. |

Option D vs five design tensions:
- **Cross-framework asymmetry:** mitigated by allowing raw-rich VASP and summary-only derivation for GROMACS/OpenMM from existing events.
- **Silent failure detection:** preserved because raw trajectory remains canonical, summaries are derived views.
- **R17 interface:** satisfied by mapping summary outputs to multiple `MetricComponent` entries (Step 12 Candidate B).
- **WDK#40 connection:** explicit `uncertainty` hook retained without forcing immediate schema commitment.
- **Adapter burden:** minimized by keeping classification/feature derivation out of adapter parsing path.

**Implications:**  
- **WDK#13 is NARROWED:** representation direction is now explicit (Option D hybrid), while taxonomy and cross-framework derivation details remain open.  
- **ComparisonProfileV1 compatibility is demonstrable:** convergence can populate multiple metric components with provenance + uncertainty hooks instead of forcing scalar collapse.  
- **Novel vs existing boundary is explicit:** raw+derived layering is existing practice (pymatgen/ASE/Custodian); convergence-pattern taxonomy and confidence calibration are novel ATHENA-specific research tasks.

**Open Threads:**  
- Define minimal `ConvergencePattern` taxonomy + confidence calibration rules with domain expert input (do not overfit VASP-first motifs).  
- Specify GROMACS/OpenMM summary derivation rules from `EnergyRecord`/`NumericalStatus` without synthetic artifacts.  
- Decide whether convergence-related events need `dag_node_ref` population strategy for stronger Stage 2/3 graph attribution.  
- Connect summary-level uncertainty fields to WDK#40 `UncertaintySummary` without coupling to a single inference family.  
- Fix summary computation timing contract (during `from_log` vs post-pass) with explicit performance and determinism criteria.

---

### 2026-02-22: Step 12 — R17 Quantitative Comparison Formalization and Interface Contract

**Scope:** Resolve/narrow What We Don't Know #28 ("How to formalize the quantitative prediction-observation comparison method (R17)") and define the bridge contract from trace-semantics output to the not-yet-started adversarial-reward track without modifying `research/adversarial-reward/FINDINGS.md`.

**Method:**  
1. Literature-grounded survey across five domains: computational V&V, Bayesian experimental design, hypothesis testing, replication effect-size practice, and active learning uncertainty.  
2. Type-level mapping from formalization requirements to prototype artifacts: `ComparisonOutcome` and `DivergenceMeasure` (`prototypes/lel-ir-prototype/src/common.rs:134-150`), and `compare_predictions`/`is_falsified` behavior (`prototypes/lel-ir-prototype/src/overlay.rs:230-277`).  
3. Steel-man + stress-test of three candidate formalizations (A/B/C) under ATHENA non-negotiables (DSL-only, warm-started priors, bounded adversarial design).  
4. Downstream compatibility scoring for adversarial reward, Bayesian Surprise Evaluator, IR simplicity, Stage 2->3 tractability, and adapter burden.

**Findings:**

1. **The five literature domains impose conflicting input contracts; no single scalar formalization satisfies all of them without loss.**  
Mechanism: each domain optimizes a different objective class, so the required sufficient statistics differ.  
Conditions: conflict appears when one comparison output must support both binary falsification decisions and continuous information-gain optimization.

| Domain | Canonical Formalization | Required Input | Compute Cost | Reward Composability |
| :--- | :--- | :--- | :--- | :--- |
| Computational V&V | ASME V&V 10/20 validation metrics; Oberkampf-Roy validation perspective | Point estimates + uncertainty/tolerance bands (often CIs or validation intervals) | Low-Moderate | Adequate after normalization; not intrinsically EIG-native |
| Bayesian Experimental Design | Lindley information gain; Knowledge Gradient | Prior + predictive/posterior distributions over outcomes | Moderate-High (integration / Monte Carlo) | Strong (direct expected gain objective) |
| Hypothesis Testing | Neyman-Pearson tests; Bayes factors | NP: test statistic + null distribution; BF: model priors + marginal likelihoods | NP Low; BF Moderate-High | NP Weak for reward shaping (binary thresholding); BF Strong (continuous evidence) |
| Scientific Replication | Cohen's d, Hedges' g, Cliff's delta | Point estimates + variance/sample size (or ordinal ranks for Cliff) | Low | Adequate as standardized evidence, weak as standalone EIG surrogate |
| Active Learning | Uncertainty sampling; Query-by-Committee disagreement | Predictive uncertainty or committee/posterior disagreement distributions | Moderate | Strong for acquisition scoring; needs calibration guardrails |

Citations: ASME V&V 10/20; Oberkampf & Roy (2010); Lindley (1956); Frazier et al. (2008); Neyman & Pearson (1933); Kass & Raftery (1995); Cohen (1988); Hedges (1981); Cliff (1993); Settles (2009); Seung et al. (1992).

2. **Current prototype output is structurally scalar-first, which is sufficient for yes/no falsification routing but insufficient for calibrated information-gain reward.**  
Mechanism: `ComparisonOutcome` stores at most one `Option<DivergenceMeasure>` and `compare_predictions` collapses outcome to `is_falsified: bool`; neither path carries uncertainty, sample-size, or model-form metadata.  
Conditions: insufficiency appears when reward must compare predicted vs actual surprise across cycles or penalize noisy/high-variance comparisons.  
Evidence: `common.rs:134-150`, `overlay.rs:230-277`, ARCHITECTURE.md §4.5 and §5.4.

3. **Q(a) — The adversarial reward interface needs both a scalar and a metric profile; a posterior distribution is optional but not always required.**  
Mechanism: the optimizer needs one scalar for ranking candidate experiments, while calibration/noise control requires access to component metrics and uncertainty drivers (e.g., variance, sample size, model family).  
Conditions: scalar-only is acceptable only when reward is heuristic and non-Bayesian; exact EIG requires posterior-compatible terms.

4. **Q(b) — The Bayesian Surprise Evaluator needs a comparison output that is (i) calibratable and (ii) invalidatable.**  
Mechanism: §5.4 compares predicted vs actual surprise over time, so comparison output must preserve a consistent functional form and metadata needed to explain calibration drift; §4.5 requires invalidation when LFI marks implementation artifact.  
Conditions: without validity flags and normalized scale metadata, predicted-vs-actual divergence cannot be diagnosed as model misspecification vs noise-seeking.  
Evidence: ARCHITECTURE.md §4.5, §5.1 (dual-path analysis), §5.4.

5. **Q(c) — `DivergenceMeasure` should remain an enum for metric identity, but it needs a metadata wrapper for uncertainty, support, and provenance.**  
Mechanism: enum-only encoding preserves metric type but loses distributional semantics required by BED/active-learning style reward and by calibration feedback.  
Conditions: this gap is material when using Bayes factors/KL/effect sizes in one comparison family and when adapters emit heterogeneous observable types.  
Evidence: current enum variants in `common.rs:142-150`; calibration requirement in ARCHITECTURE.md §5.4.

6. **Candidate evaluation (steel-man + stress-test) recommends Candidate B (Multi-Metric Divergence Profile) with optional distribution hooks.**

| Candidate | (a) Adversarial Reward | (b) Bayesian Surprise Evaluator | (c) IR Simplicity | (d) Stage 2->3 Tractability | (e) Adapter Burden |
| :--- | :--- | :--- | :--- | :--- | :--- |
| A. Typed Scalar Divergence | **Adequate**: easy scalar optimization, weak anti-gaming controls | **Weak**: no posterior/uncertainty context for calibration | **Strong**: minimal type change | **Strong**: cheapest runtime | **Strong**: point values usually available |
| B. Multi-Metric Profile | **Strong**: scalarized reward + component-level controls | **Adequate-Strong**: supports calibrated surrogate surprise; can upgrade to exact forms | **Adequate**: moderate type expansion | **Adequate**: bounded extra cost per metric vector | **Adequate**: needs uncertainty + sample metadata where available |
| C. Distribution-Aware Posterior | **Strong**: direct EIG compatibility | **Strong**: direct KL-style surprise semantics | **Weak**: substantial core-type expansion | **Weak-Adequate**: expensive unless heavily approximated | **Weak**: many adapters cannot natively emit full posteriors |

7. **Recommended formalization: Candidate B as default contract, with an optional posterior payload field to preserve a migration path to Candidate C.**  
Mechanism: B satisfies bounded-adversarial needs by providing one deterministic scalar for optimization plus auditable metric components for calibration and Noisy-TV resistance; optional posterior summary prevents architectural dead-end for future exact EIG.  
Conditions: this recommendation holds when DSL adapters provide at least point estimates + uncertainty surrogates; if a domain provides robust posterior models, C-style fields can be populated without changing consumer interfaces.

8. **Bridge contract (trace-semantics -> adversarial-reward) is now specified as an interface assumption set, not an integration.**  
Mechanism: trace-semantics emits a typed comparison profile with guarantees; adversarial-reward may assume these guarantees once that track starts.

Proposed interface contract (conceptual, no prototype change in this step):

```text
ComparisonProfileV1 {
  comparison_event_id: EventId
  prediction_id: SpecElementId
  dag_node: Option<String>
  metrics: Vec<MetricComponent>
  aggregate: AggregateScore
  reward_validity: RewardValidity
  provenance: ProvenanceAnchor
}

MetricComponent {
  kind: DivergenceKind              // AbsoluteDifference|ZScore|BayesFactor|KLDivergence|EffectSize|Custom
  value: f64
  direction: Option<EffectDirection>
  uncertainty: Option<UncertaintySummary>
  sample_size: Option<u32>
  units: Option<Unit>
  method_ref: String                // metric/test definition id
}
```

Guarantees adversarial-reward can assume:
- **G1 Determinism:** identical IR inputs produce identical `metrics` and `aggregate`.
- **G2 Monotonicity Declaration:** `aggregate` includes explicit monotonic convention (higher = more contradiction evidence) and bounded support metadata.
- **G3 Validity Gating:** `reward_validity=false` when LFI classifies implementation artifact (surprise invalidation compatibility).
- **G4 Auditability:** each metric has trace/spec/DAG provenance sufficient for post-hoc calibration debugging.
- **G5 Partial Distribution Support:** when posterior metadata exists, it is attached via `uncertainty`; when absent, omission is explicit (no silent fallback).

9. **Proven vs conjectural boundary is now explicit.**  
Proven in repository: current type/behavior limitations and calibration/invalidation dependencies (`common.rs`, `overlay.rs`, ARCHITECTURE §4.5/§5.4).  
Existing technique: all listed metric families and acquisition criteria are established in literature.  
Novel research still required: (i) canonical aggregation from profile -> bounded reward scalar under Noisy-TV constraints, and (ii) uncertainty schema that is expressive enough for BED but feasible across DSL adapters.

**Implications:**  
- **R17 formalization is narrowed (not fully closed):** Candidate B is the recommended default for ATHENA's current architecture, with Candidate C compatibility hooks retained.  
- **Adversarial-reward bridge is now concrete:** the reward track can consume `ComparisonProfileV1` assumptions without requiring immediate prototype refactors.  
- **Type gap documented, not implemented:** current `ComparisonOutcome`/`DivergenceMeasure` shape does not encode the contract; this is recorded as follow-up design work, not code change, per scope.

**Open Threads:**  
- Define minimal `UncertaintySummary` schema that supports both scalar metrics and trajectory-aware convergence evidence (follow-up connection to What We Don't Know #13).  
- Define machine-checkable contradiction-chain construction from multi-metric profiles to edge-level update directives (follow-up connection to What We Don't Know #9).  
- Specify and benchmark the profile aggregation function used for bounded adversarial reward calibration under §5.4 feedback constraints.

---

### 2026-02-22: Step 11 — Hidden Confounder Prototype Litmus Test

**Scope:** Validate that the R14 confounder detection mechanism works end-to-end on VASP-derived LEL data.

**Method:** Added `test_vasp_hidden_confounder_litmus` and `test_vasp_hidden_confounder_controlled_excluded` in `prototypes/lel-ir-prototype/src/tests/mod.rs`. Parsed VASP INCAR/OSZICAR/OUTCAR via `VaspAdapter`, planted `PREC` as a common ancestor of `SIGMA` and `IBRION` by mutating `causal_refs`, rebuilt indexes, and executed `CausalOverlay::detect_confounders`.

**Findings:**

1. **`detect_confounders` identifies the planted confounder in the VASP path.** The litmus test returns non-empty candidates and includes `dag_node == "PREC"` for the planted common-cause structure.

2. **Controlled-variable exclusion works as required by R14 semantics.** When `PREC` is added to `spec.controlled_variables`, confounder detection correctly returns no candidates for the same planted graph.

3. **The validation is integrated into the main crate quality gates.** The prototype now passes 92/92 tests and strict clippy clean, with the litmus included in the standard run.

**Implications:**
- LEL IR now demonstrates causal-query diagnostic value beyond parsing fidelity.
- The hidden confounder mechanism is validated in prototype scope on realistic DFT-flavored traces.

**Open Threads:**
- Full 50-cycle hidden-confounder evaluation environment still depends on adversarial-reward formalization (`research/adversarial-reward/`).

---

### 2026-02-22: Step 10 — VASP Adapter Implementation

**Scope:** Parse VASP INCAR/OSZICAR/OUTCAR into LEL using existing types only, addressing What We Don't Know #12 ("whether one IR schema can accommodate both DFT and MD frameworks").

**Method:** Added `src/vasp_adapter.rs` mirroring the GROMACS adapter structure (classification function, per-file parsers, `DslAdapter` integration, causal wiring, `LayeredEventLogBuilder` assembly). Added 25 `test_vasp_*` tests in `src/tests/mod.rs` covering classification, parser behavior, integration, overlay construction, confounder litmus behavior, and error handling.

**Findings:**

1. **No `EventKind` or core-type changes were required for VASP.** INCAR/OSZICAR/OUTCAR content mapped into existing LEL structures, affirmatively resolving WDK#12 in prototype scope.

2. **Previously unexercised variants are now covered by adapter paths.** `ConvergencePoint` is emitted from OSZICAR SCF iterations and `StateSnapshot` is emitted from OUTCAR force-block headers.

3. **Multi-file section-marker composition works for three VASP sources.** The adapter composes marker-delimited sections order-agnostically and supports INCAR-only fallback.

4. **Quality gates remain clean after expansion.** The crate now passes 92/92 tests with strict clippy clean.

**Implications:**
- The single IR design now covers both MD (OpenMM/GROMACS) and DFT (VASP) adapter paths without schema branching.
- Hybrid overlay queries remain compatible after adding a materially different simulation paradigm.

**Open Threads:**
- Validate parser behavior against additional real VASP outputs beyond prototype samples.
- POTCAR/pseudopotential-specific parsing remains out of scope for the current adapter.

---

### 2026-02-21: Step 9: GROMACS Adapter for Cross-Framework Validation

**Scope:** Implement a hand-written GROMACS `.mdp`/`.log` adapter in the Rust prototype, map all parsed outputs onto existing LEL `EventKind` variants, and validate end-to-end compatibility with `CausalOverlay` and R14 confounder detection.

**Method:** Added `src/gromacs_adapter.rs` with deterministic MDP parameter classification (Theory/Methodology/Implementation + boundary annotations), provenance-preserving parsers (`parse_mdp`, `parse_log`), and `DslAdapter` integration that assembles `LayeredEventLog`, wires causal references post-parse, and derives `ExperimentSpec.controlled_variables` from `ref_t`/`ref_p`. Appended 22 tests to `src/tests/mod.rs` covering classifier behavior, parser correctness, adapter integration, overlay construction, and confounder detection over GROMACS-derived events.

**Findings:**

1. **Existing `EventKind` variants were sufficient for GROMACS trace semantics.** The adapter exercised `ParameterRecord`, `ResourceStatus`, `EnergyRecord`, `NumericalStatus`, and `ExecutionStatus` without adding or modifying IR type definitions.

2. **No blocking type gaps were discovered for Step 9 scope.** GROMACS `.mdp`/`.log` content mapped into the current LEL schema using existing `Value`, `BoundaryClassification`, temporal/provenance, and confidence metadata fields.

3. **Layer classification is tractable and deterministic for real `.mdp` parameters.** Implemented explicit lookup rules (including DualAnnotated rationale strings for `dt`/`tau_t`/`tau_p`, `constraints`, `rcoulomb`/`rvdw`) and a deterministic ContextDependent fallback for unknown parameters.

4. **Validation expanded from 44 to 66 passing tests with quality gates clean.** The crate now passes 66/66 tests (`44` existing + `22` Step 9 additions), and `cargo clippy --all-targets --all-features -- -D warnings` passes at zero warnings.

**Implications:**
- The Hybrid LEL+DGR prototype now demonstrates cross-framework IR generalization beyond OpenMM: the same event/type system supports GROMACS parsing, overlay construction, and causal query execution.
- Adapter-level parsing differences do not require IR schema branching, supporting the architecture claim that DSL-specific adapters can feed a common core representation.

**Open Threads:**
- Broader cross-framework validation still requires a VASP adapter path for Stage 2-3 queries.
- GROMACS energy-table parsing currently uses prototype-scoped heuristics for multi-word headers; production hardening remains future work.

---

### 2026-02-21: Step 7: R17+R18 Query Implementation

**Scope:** Implement and validate R17 and R18 query methods on `CausalOverlay`: prediction-observation comparison extraction and implicated causal node mapping with layer classification.

**Method:** Direct implementation in `prototypes/lel-ir-prototype/src/overlay.rs` reusing the existing overlay traversal infrastructure. Added a private depth-aware BFS helper (`ancestors_with_depth`) without modifying `transitive_ancestors`, grouped implicated nodes with `BTreeMap`, and resolved `ComparisonResult.prediction_id: String` to `SpecElementId` at query time via parse-at-query-time conversion.

**Findings:**

1. **R17 `compare_predictions` now executes end-to-end.** The query reads `ComparisonResult` events via `indexes.by_kind`, resolves event positions via `indexes.by_id`, parses `prediction_id` string values into `SpecElementId` where possible, joins against `spec.predictions`, and emits `PredictionComparison` outputs with falsification status and DAG-node propagation.

2. **R18 `implicate_causal_nodes` now executes end-to-end with three-way layer classification.** Starting from a comparison event index, the query traverses causal ancestors with BFS depth tracking, groups by DAG node, selects minimum causal distance per node, and returns sorted implicated nodes in Theory→Methodology→Implementation order.

3. **The String→`SpecElementId` mismatch is viable at query time in prototype scope.** Parse-at-query-time conversion handles resolvable IDs while preserving graceful fallback (`prediction_id: None`, variable `"unknown"`) for malformed or unresolvable IDs.

4. **Test coverage for Stage 3 query behavior is now explicit and passing.** Added 15 tests (7 for R17, 8 for R18) covering empty/no-event guards, matched/falsified comparisons, malformed IDs, DAG-node forwarding, layer-specific implication, mixed-layer ordering, depth correctness, ancestor node filtering, and same-node grouping.

5. **Quality gates remain clean after the expansion.** Prototype now passes 44/44 tests with strict clippy (`--all-targets --all-features -- -D warnings`) at zero warnings.

**Implications:**
- Full Stage 2-3 query surface is now validated in the prototype with implemented R14 + R17 + R18 methods.
- The Hybrid overlay design supports both confounder detection and falsification-to-causal-implication workflows without architectural changes.
- Query-time ID parsing is sufficient for prototype iteration speed while preserving deterministic behavior.

**Open Threads:**
- GROMACS adapter work remains required for cross-framework generalization of the Stage 2-3 query path.
- `prediction_id` type harmonization (`String` vs `SpecElementId`) remains deferred to a production ADR.

---

### 2026-02-21: Hybrid LEL+DGR Phase 2 Prototype — CausalOverlay + R14 Query (Step 6)

**Scope:** Implement and validate the graph-traversal half of the Hybrid architecture in the Rust prototype: `EventIndexes.by_id`, `CausalOverlay` construction/traversal, and R14 confounder detection over the overlay.

**Method:** Direct implementation in `prototypes/lel-ir-prototype/` following the approved dependency order (Task 1→5): extend indexes, add overlay module with index-only entity mapping, add R14 query method, migrate benchmark to real overlay construction path, and validate each step with `cargo test` + strict clippy.

**Findings:**

1. **`EventIndexes.by_id` is implemented and serialized.** `by_id: HashMap<EventId, usize>` now records event position at insert time. Builder wiring uses `self.events.len()` before push. Added tests for population, position correctness, and serde roundtrip.

2. **`CausalOverlay` now exists as a first-class prototype artifact (`src/overlay.rs`).** Construction is a single O(n) pass with `Vec::with_capacity(n)`, 1:1 entity mapping (`event_idx == log.events index`), `dag_node: Option<String>`, and `causal_parents` resolved through `log.indexes.by_id` using `filter_map` (dangling refs skipped).

3. **Graph traversal APIs are implemented and validated.** Accessors (`len`, `is_empty`, `entity`) plus `transitive_ancestors` (on-demand BFS, start node excluded) are covered by empty, linear-chain, diamond, and dangling-reference tests.

4. **R14 confounder detection query is implemented on the overlay.** `detect_confounders` performs variable existence guard, event-position resolution, transitive ancestor set intersection, controlled/intervention filtering, and dag-node grouping into `ConfounderCandidate` outputs. Added 7 targeted tests (all-controlled, uncontrolled-detected, intervention-excluded, no-common-ancestor, unknown-variable, multiple confounders, transitive chain).

5. **Benchmark now exercises real overlay construction, not ad-hoc HashMaps.** `src/bench.rs` uses `CausalOverlay::from_log(&log)` and reports overlay-backed counts. Observed at 10^6 events: log construction 2130.33ms, overlay construction 251.82ms, 1,000,000 overlay entities, 199,998 derivation edges, 50 DAG-node groups.

6. **Prototype quality gates passed after each task boundary.** Final crate state: 29/29 tests passing, strict clippy (`--all-targets --all-features -- -D warnings`) passes with zero warnings.

**Implications:**
- Hybrid Phase 2 is now concretely prototyped: LEL event stream can be lifted to an index-only causal overlay with O(n) construction and on-demand graph traversal.
- Phase 3 query work is unblocked for confounder-oriented causal analysis (R14 path now executable end-to-end in prototype form).
- Thread #37 is closed (implemented). Thread #38 is narrowed with empirical support: Vec-first allocation remains adequate at current scale; arena remains optional only if future profiling indicates measurable allocation overhead.

**Open Threads:**
- VASP Stage 3 representation gaps remain open (#35 `ContractTerm.value`, #36 matrix/function value support).

---

### 2026-02-21: Open Thread Resolution (Step 5c)

**Scope:** Resolve or narrow 5 open threads from Step 5a (candidate IR schemas) using LEL prototype evidence and analytical reasoning.

**Method:** Empirical benchmark (#31: overlay construction cost), analytical reasoning from prototype evidence (#32: references from day one, #34: OverlayEntity sufficiency), document-driven analysis against DSL survey findings (#33: ExperimentSpec sufficiency), theoretical analysis (#35: arena allocation).

**Findings:**

1. **Thread #31 RESOLVED: Overlay construction cost is empirically bounded.** Benchmark (`src/bench.rs`) measures O(n) HashMap-building pass over synthetic LEL events at 4 scales. Results at 10^6 events: overlay construction 80.53ms, log construction 488.96ms, ~10.7MB overlay memory, ~300K overlay entities, ~200K derivation edges. Linear scaling confirmed: 10^5→10^6 scales ~9x for overlay (8.97→80.53ms). The O(n) pass is tractable for megabyte-scale traces on commodity hardware. [bench.rs benchmark, release mode]

2. **Thread #32 NARROWED: "From day one" is the safer default; deferred resolution is a viable escape hatch.** Prototype evidence: (a) `dag_node_ref`, `spec_ref`, `causal_refs` compile and serialize with None/empty values (`test_hybrid_upgrade_fields_present`, `test_serde_roundtrip`); (b) mock adapter in adapter.rs leaves `dag_node_ref`/`spec_ref` as None — adapters can defer without structural penalty; (c) `EventIndexes.by_dag_node` index populates incrementally during `index_event()` (lel.rs:141-146) — works whether references are upfront or via deferred pass. Deferred resolution is viable via a parallel reference map (`HashMap<EventId, (Option<String>, Option<SpecElementId>)>`) applied at Stage 1→2 boundary as an O(n) pass. Remaining question narrowed to: is the two-phase adapter protocol acceptable complexity for specific adapters? This is an adapter API design decision, not an IR correctness question. [LEL prototype: adapter.rs, lel.rs:141-146, tests]

3. **Thread #33 NARROWED: ExperimentSpec sufficient for all three frameworks at Stage 1; two specific VASP Stage 3 gaps identified.** Analysis against each framework's adapter needs using DSL survey findings: OpenMM — sufficient (`createSystem()` chain is adapter-internal, not spec). GROMACS — sufficient (.mdp → `controlled_variables`/`interventions`, grompp → trace events). VASP — two gaps: (a) `ContractTerm` (common.rs:94-99) has only `description: String`, needs `value: Option<Value>` for machine-readable precondition checking (e.g., POTCAR family = PBE); (b) `PredictionRecord.predicted_value: Value` cannot represent spectral data (band structure over k-points), would need `KnownMatrix` or function variant in `Value` enum. Both gaps are non-blocking for current scope (OpenMM Stage 1). [DSL surveys: OpenMM, GROMACS, VASP; common.rs:94-99, common.rs:102-108]

4. **Thread #34 NARROWED: Lightweight OverlayEntity sufficient for Stage 2-3 queries; one missing index identified.** Analysis of three actual query patterns: R14 (confounder) traverses `causal_ancestors` → common ancestors → `dag_node_ref` against controlled variables → event lookup — OverlayEntity fields sufficient. R17 (comparison) uses `spec_ref` for prediction, `event_id` for observation — sufficient. R18 (causal implication) traverses derivation edges → `dag_node_ref` — sufficient. One gap: `EventIndexes` lacks `by_id: HashMap<EventId, usize>` for O(1) event lookup by ID. Currently `events` is `Vec<TraceEvent>` with no ID→index mapping. OverlayEntity's `event_id` field requires this to avoid O(n) linear search. Small addition (~8 bytes/event). [LEL prototype: lel.rs:88-96 EventIndexes, lel.rs:51-85 TraceEvent]

5. **Thread #35 DEFERRED with concrete guidance: Vec-first, benchmark at Phase 2.** The Hybrid's overlay construction is a single batch O(n) pass — all OverlayEntities allocated in one sweep. For batch allocation, `Vec<OverlayEntity>` with `Vec::with_capacity(n)` achieves the same cache locality as an arena allocator. Arena provides benefit only when allocations are interleaved with other work (preventing heap fragmentation) — not the Hybrid's pattern. Recommendation: start with Vec, benchmark at 10^6 scale during Phase 2, add arena crate (`bumpalo` or `typed-arena`) only if allocation overhead is measurable. [Theoretical analysis; benchmark confirms batch pattern at scale]

6. **Benchmark artifact produced.** `src/bench.rs` as `[[bin]]` target, zero new dependencies (uses `std::time::Instant`). Tests construction at 4 scales (10^3, 10^4, 10^5, 10^6) with realistic event distributions (70/20/10 layer split, 30% `dag_node_ref`, 10% `causal_refs`). Reports wall-clock time, entity/edge counts, memory estimates. [bench.rs]

**Implications:**
- All 5 open threads from Step 5a are now resolved (1), narrowed (3), or deferred with concrete guidance (1). No thread remains open-ended.
- The Hybrid LEL+DGR architecture's key performance claim (O(n) overlay construction at megabyte scale) is now empirically validated.
- Two concrete tasks for future work identified: (a) add `by_id: HashMap<EventId, usize>` to `EventIndexes` for Phase 2 CausalOverlay; (b) add `value: Option<Value>` to `ContractTerm` for VASP Stage 3.
- The deferred reference resolution strategy provides a viable escape hatch for adapters where per-event entity resolution is costly, without requiring IR structural changes.
- Arena allocation is deferred with clear trigger: benchmark Vec at Phase 2 scale; adopt arena only if measurable overhead.

**Open Threads:**
- None. All threads resolved, narrowed, or deferred with concrete guidance.

---

### 2026-02-20: Candidate IR Schemas with Hybrid LEL-DGR Recommendation (Step 5a)

**Scope:** Synthesize all accumulated evidence (R1-R29 requirements, coverage matrix, pattern catalog, cross-framework synthesis) into concrete IR schema designs. Evaluate candidates against requirements, anti-patterns, streaming constraints, and stage-specific performance. Produce a recommendation for Step 5b prototyping.

**Method:** Schema design driven by three inputs: (1) requirements-coverage-matrix.md (R1-R29 coverage codes, gap analysis, three-input data flow architecture), (2) ir-pattern-catalog.md (7 transferable patterns, 9 anti-patterns, candidate previews, unified architecture), (3) cross-framework-synthesis.md (adapter contract, failure modes, boundary parameters). Candidates evaluated against a 7-criterion weighted framework (R1-R29 coverage 25%, anti-pattern compliance 20%, streaming 15%, Stage 1 efficiency 15%, Stage 2-3 causal reasoning 15%, implementation complexity 5%, incremental adoptability 5%). Key design decision: 2 candidates + 1 hybrid, replacing TAL standalone with LEL-DGR Hybrid based on coverage matrix conclusions.

**Findings:**

1. **TAL deferred to query-layer role, replaced by LEL-DGR Hybrid.** The coverage matrix (requirements-coverage-matrix.md §8) concluded TAL "works better as a query interface layer than a data representation." TAL has the highest novelty risk (no close precedent), weakest causal graph traversal support, and its core strength (sequential assertion checking) functions identically as a query interface over LEL or DGR substrates. TAL's assertion-checking pattern is preserved as the recommended LFI query interface. The Hybrid candidate addresses open questions #2 (incremental path) and #5 (LEL→DGR viability), which are more architecturally informative than a high-risk novelty candidate. [candidate-ir-schemas.md §0]

2. **A common structural foundation shared by all candidates was defined.** Seven shared types: `Layer` enum (Theory/Methodology/Implementation) for R19, `BoundaryClassification` enum (PrimaryLayer/DualAnnotated/ContextDependent) resolving OQ4, `ObservationMode` enum for R28, `Value` enum with `Havoc` variant (Boogie P6) for R26, `TemporalCoord` struct (simulation_step/wall_clock_ns/logical_sequence) for R21, `ProvenanceAnchor` struct for R20, `ExperimentRef` struct for R22/R29, and `ConfidenceMeta` struct for R25. These types ensure consistent semantics regardless of which candidate is chosen. [candidate-ir-schemas.md §1]

3. **LEL (Layered Event Log) scores 82/100.** STRONG for Stage 1 (7/7 requirements), streaming (pure append-only), and implementation simplicity. WEAK for R14 (confounder query — requires multi-way joins unsupported by flat log) and R18 (causal implication — requires transitive causal ancestry unsupported without graph traversal). PARTIAL on AP7 (implicit causal ordering — causal_refs are optional best-effort). [candidate-ir-schemas.md §2, §5, §6, §9]

4. **DGR (Dual-Graph IR) scores 82/100.** STRONG for all R1-R29 requirements (full coverage including R14 and R18 via graph traversal) and Stages 2-3 causal reasoning. PARTIAL on AP2 (post-mortem-only — spec_graph pre-built before trace, acceptable). MODERATE for streaming (graph construction from streaming data requires forward-reference management) and Stage 1 efficiency (graph construction overhead for the most common classification path). Same total as LEL but with inverted strengths/weaknesses. [candidate-ir-schemas.md §3, §5, §6, §9]

5. **Hybrid (LEL core + DGR overlay) scores 94/100.** Captures LEL's strengths (streaming, Stage 1 efficiency) and DGR's strengths (causal reasoning, R1-R29 coverage). PASS on all 9 anti-patterns (the only candidate with no PARTIAL ratings). Stage 1 operates as pure LEL (append-only, early termination if implementation fault found). CausalOverlay built at Stage 1→2 boundary via single O(n) pass over events. Key constraint: LEL events must carry `dag_node_ref` and `spec_ref` from the start to enable overlay construction. [candidate-ir-schemas.md §4, §5, §6, §9]

6. **R17 (prediction-observation comparison) resolved structurally via ComparisonResult + DivergenceMeasure.** The IR provides a structural container with six divergence measure variants (AbsoluteDifference, ZScore, BayesFactor, KLDivergence, EffectSize, Custom). The comparison method is pluggable — the IR stores results, not logic. The LFI selects the appropriate measure per prediction type. The comparison formalization research question is now scoped to LFI logic, not IR structure. [candidate-ir-schemas.md §3, §8 OQ1]

7. **The LEL→DGR incremental path is viable.** The Hybrid demonstrates viability by construction. Key constraint identified: LEL events must include DGR-compatible references (dag_node_ref, spec_ref, causal_refs) from day one. If these are omitted during initial implementation, overlay construction requires re-parsing. Implication for Step 5b: the LEL prototype must include these fields even though Stage 1 does not use them. [candidate-ir-schemas.md §4, §8 OQ2]

8. **Causal reasoning substrate is per-stage.** Stage 1: sequential search sufficient (filter-and-inspect on implementation-tagged events, O(n) with early termination). Stages 2-3: graph traversal required (transitive causal ancestry for R14 confounder queries, structural path finding for R18 causal implication). This per-stage answer directly motivates the Hybrid design. [candidate-ir-schemas.md §8 OQ3]

9. **BoundaryClassification enum resolves the boundary parameter representation question.** Three variants: PrimaryLayer (unambiguous), DualAnnotated (primary layer for routing + secondary layer annotation, e.g., GROMACS dt), ContextDependent (default layer + context note, e.g., VASP ALGO). Avoids both a fourth "boundary" layer and entity duplication. [candidate-ir-schemas.md §1, §8 OQ4]

10. **Step 5b recommendation: LEL prototype on OpenMM traces.** Scope: R1-R7 + R19 + R20 + R21. Target: validate event typing, layer tagging, specification separation with minimal complexity. Critical: include dag_node_ref/spec_ref/causal_refs fields for Hybrid upgrade path. Evolution: LEL → Hybrid (overlay) → full DGR as Stages 2-3 mature. [candidate-ir-schemas.md §10]

**Implications:**
- The IR design question is now resolved to a recommended architecture (Hybrid LEL+DGR) with a concrete prototyping plan (LEL first, OpenMM target).
- Step 5b can proceed immediately with a well-scoped prototype: LEL core on OpenMM traces, Stage 1 requirements only, with Hybrid upgrade path preserved.
- The common structural foundation (Section 1 types) should be implemented first and shared across any candidate, ensuring consistent semantics regardless of which IR representation is used.
- The R17 comparison formalization is now scoped to LFI logic, not IR structure — it can be researched independently of IR prototyping.
- TAL as a query-layer interface should be designed alongside the LFI, not as an IR component.

**Open Threads:**
- DGR overlay construction cost at the Stage 1/2 boundary for megabyte-scale traces (10^5-10^6 events). The O(n) pass is theoretically fast but untested empirically. Performance validation needed during Step 5b or an early Hybrid prototype.
- Whether HybridIR events need full DGR-compatible references (dag_node_ref, spec_ref, causal_refs) from day one, or whether a deferred reference-resolution pass is acceptable. The current recommendation is "from day one" for safety, but this pushes entity resolution complexity into the adapter during Stage 1, when it's not needed.
- Arena allocation strategy for the CausalOverlay. The overlay entities reference back to LEL events — the allocation pattern and cache friendliness of this indirection need benchmarking.
- Whether the ExperimentSpec struct is sufficient for all three frameworks or whether framework-specific extensions are needed. The current design is generic; adapter-specific spec fields may be needed.
- The OverlayEntity is lightweight (wraps an LEL event reference + graph relationships). Whether this indirection is sufficient for Stage 2-3 queries or whether richer overlay entities (carrying computed fields, derived values) are needed.

---

### 2026-02-20: Requirements Coverage Matrix and Gap Analysis (Step 3b)

**Scope:** Cross-reference R1-R29 requirements against the trace capability matrix from Step 1d (cross-framework-synthesis.md). For each requirement × framework cell, classify data availability using six codes (DA/DI/ER/FU/NT/DE). Perform gap analysis for all non-DA cells. Assess per-stage feasibility. Evaluate Decision Gate 4.

**Method:** Systematic assessment of each R1-R29 requirement against OpenMM, GROMACS, and VASP trace capabilities. Evidence drawn from cross-framework-synthesis.md (trace capability matrix §1, boundary assessment §2, failure modes §3, completeness §4, adapter contract §5.3, coverage implications §7.1), ir-pattern-catalog.md (pattern coverage annotations, candidate designs §6), and evaluation/hidden-confounder/README.md (R27-R29 context). Requirements assessed in order: Stage 1 (R1-R7), Cross-cutting (R19-R29), Stage 2 (R8-R14), Stage 3 (R15-R18). Each cell classified with code + evidence note + confidence + source reference.

**Findings:**

1. **Stage 1 (R1-R7) is fully satisfiable for all three frameworks.** OpenMM requires the most custom instrumentation (4 DI cells vs. 0 for GROMACS/VASP) because it lacks built-in parameter echo and requires API queries for specification/resource data. GROMACS has the best default Stage 1 coverage (5 DA cells). VASP has good structured output but exit code unreliability for SCF non-convergence (R1 caveat). [requirements-coverage-matrix.md §1]

2. **31% of requirements (9 of 29) are NT — external to the Trace Semantics Engine.** R9, R10, R11, R15, R18, R22, R23, R28 come from experiment specification, hypothesis, or DAG. R29 (cycle_id) comes from the workflow controller. This confirms the IR is a composite multi-source structure, not a pure trace-log derivative. [requirements-coverage-matrix.md §5.1 Strategy C]

3. **R19 (layer tag) has the widest framework variance: OpenMM=DA, GROMACS=DI+ER, VASP=ER.** OpenMM's API-enforced boundary yields clean layer tags. GROMACS needs a moderate classification table (~10 boundary params). VASP needs an extensive table (~200-300 INCAR tags) with context-dependent ambiguity for ~5-10 tags. This is the only cross-cutting requirement with framework-dependent difficulty. [requirements-coverage-matrix.md §2]

4. **Stage 2 (R8-R14) is the weakest stage, limited by external context rather than trace data.** The IR contributes only R8 (observable values) and R12 (sampling metadata) to Stage 2. The remaining 5 requirements are NT (from experiment spec/hypothesis/DAG) or DE (computed from other elements + DAG). This is consistent with the accumulated finding that methodology failures are invisible to all frameworks. [requirements-coverage-matrix.md §3]

5. **Stage 3 (R15-R18) is feasible but blocked on one research element.** R17 (prediction-observation comparison) is DE (computable) but the quantitative comparison method — effect size measures, divergence metrics, tolerance thresholds for scientific predictions — is novel research not yet formalized. All other Stage 3 requirements are satisfiable. [requirements-coverage-matrix.md §4]

6. **FU cells are narrowly scoped and below the 10% threshold.** No full requirement is FU for any framework. Partial FU exists only for R6 (sub-component numerical internals) in all three frameworks: OpenMM ~5% (GPU precision), GROMACS ~5% (constraint solver internals), VASP ~5-10% (FFT/PAW internals). The surface-level metrics are available (DA/DI). [requirements-coverage-matrix.md §5.1 Strategy E]

7. **Decision Gate 4: PASS.** No LFI stage has FU requirements blocking >10% of expected failure classifications. Four conditions: (a) OpenMM custom reporter required, (b) VASP INCAR classification table required, (c) VASP 20-30% degraded confidence for ambiguous params accepted per DG1, (d) R17 comparison method requires formalization. [requirements-coverage-matrix.md §7]

8. **DGR (Dual-Graph IR) is the recommended primary candidate for Step 5a.** The coverage matrix reveals the IR is fundamentally a three-input composite (trace data + external context + domain rules). DGR's graph structure naturally represents entities from all three sources with qualified relationships. LEL is strongest for Stage 1 (high DA density). TAL works better as a query interface layer than as a standalone IR. [requirements-coverage-matrix.md §8]

**Implications:**
- Step 5a (candidate IR schemas) is now unblocked. The coverage matrix provides: (a) concrete data availability per requirement per framework, (b) gap fill strategies with complexity estimates, (c) the three-input data flow architecture as an organizing principle, (d) candidate-specific coverage pattern analysis.
- The IR's three-input architecture (trace + external + domain rules) should be the organizing principle for candidate evaluation, not just trace parsing capability.
- OpenMM adapter requires the most engineering (10 DI cells) but has the cleanest structural foundation (R19=DA). VASP adapter requires the most domain knowledge (R19=ER, ~200-300 tag table) but has good default trace output.
- The prediction-observation comparison formalization (R17) is a discrete, well-scoped research problem that should be elevated as a prerequisite for Stage 3 capability.

**Open Threads:**
- Per-force-group energy decomposition overhead in OpenMM (R6 DI) — untested, affects custom reporter design decisions. [What We Don't Know #2]
- Quantitative prediction-observation comparison method — the single unresolved research element blocking Stage 3. Related to DRAT propositional-to-statistical adaptation. [ir-pattern-catalog.md §7 Open Thread]
- Whether the LEL→DGR incremental path is viable — start with LEL for Stage 1 prototype, evolve toward DGR. Depends on whether adding graph structure is incremental or requires redesign. [ir-pattern-catalog.md §7 Question 5]
- VASP INCAR classification table completeness and validation — needed before VASP adapter design. [cross-framework §6.4]

---

### 2026-02-20: Comparative IR Synthesis (Step 2c)

**Scope:** Synthesis of RCA/formal verification IR survey and provenance/workflow IR survey into a unified pattern catalog. Resolution of the MLIR-dialects vs. PROV-DM-hybrid tension. Decision Gate 2 assessment.

**Method:** Systematic comparison of 20 patterns across both surveys, distilled into 7 pattern categories. Each pattern evaluated against LFI audit stage requirements, R1-R29 coverage, and Rust/streaming compatibility. Anti-patterns cataloged from both surveys with severity ratings. Tension resolution through compositional analysis (MLIR for routing, PROV-DM for provenance).

**Findings:**

1. **Seven transferable pattern categories identified with stage mappings.** Counter-example traces (MEDIUM), Entity-Activity-Agent (HIGH data model / LOW tech stack), typed event chains (HIGH), SSA data flow (MEDIUM-HIGH), multi-level dialects (HIGH), spec-implementation contracts (HIGH), causal dependency/conformance (MEDIUM). Patterns 5 (dialects) and 6 (contracts) are the highest-transferability patterns. [ir-pattern-catalog.md §1]
2. **Stage 2 (methodology audit) is the weakest stage across all patterns.** No surveyed system provides native methodology adequacy checking. Patterns provide structural scaffolding for encoding methodology metadata, but domain-specific adequacy rules are external to IR design. This is consistent with the DSL trace finding that methodology failures are invisible to all frameworks. [ir-pattern-catalog.md §2]
3. **MLIR dialects and PROV-DM are complementary, not contradictory.** Dialects answer "WHERE does an element belong?" (classification/routing). PROV-DM answers "HOW are elements causally related?" (causal structure). The unified architecture uses dialect structure as primary organization with PROV-DM-like causal graphs within each layer. [ir-pattern-catalog.md §4]
4. **Decision Gate 2: Hybrid adaptation, MEDIUM risk.** ~65-70% transfers from existing systems (12 specific patterns). ~30-35% requires novel design: three-way layer typing vocabulary, fault classification ontology, quantitative prediction-observation comparison formalization, methodology detection rules. [ir-pattern-catalog.md §5]
5. **Nine anti-patterns cataloged with avoidance guidance.** CRITICAL: specification-implementation conflation. HIGH: post-mortem-only design, full-granularity recording, binary pass/fail, lossy compression without principled selection. [ir-pattern-catalog.md §3]
6. **Three candidate IR designs mapped to pattern sources.** LEL (Layered Event Log) is simplest, strongest for Stage 1. DGR (Dual-Graph IR) is the natural synthesis of both surveys, strongest for Stages 2-3. TAL (Typed Assertion Log) is most ATHENA-specific and highest-novelty-risk. [ir-pattern-catalog.md §6]

**Implications:**
- The IR structural foundation is now defined: MLIR-style dialect tags for layer routing + PROV-DM-inspired causal graphs within layers + Boogie-style contracts for specification.
- The technology stack is resolved: Rust-native implementation, no RDF/SPARQL, per ADR 001.
- Four novel elements flagged as requiring original research (not available from surveyed systems).
- Step 5a (candidate IR schemas) can now proceed with clear structural foundation and pattern-to-candidate mapping.

**Open Threads:**
- Quantitative prediction-observation comparison formalization — DRAT is propositional, scientific refutation is quantitative. Bridging mechanism undefined.
- How to handle events spanning multiple dialects simultaneously (e.g., VASP's PREC parameter).
- Which causal reasoning substrate (log search, graph traversal, assertion chains) best matches LFI's actual query patterns — requires enumeration of specific queries derived from R1-R29.
- Whether the unified architecture can be incrementally implemented (start with LEL, evolve toward DGR).

---

### 2026-02-20: Cross-Framework Trace Synthesis (Step 1d)

**Scope:** Comparative analysis of OpenMM, GROMACS, and VASP trace output systems. Production of trace capability matrix, failure mode taxonomy, trace completeness assessment, and Decision Gate 1 assessment.

**Method:** Systematic cross-referencing of the three DSL trace analysis documents. Seven trace element categories compared across frameworks with format, access method, and layer classification. Failure modes harmonized into a cross-framework taxonomy. Theory-implementation boundary assessed per-framework with boundary parameter catalog.

**Findings:**

1. **Trace capability matrix completed across 7 categories.** State snapshots, energy series, convergence metrics, error/warning messages, parameter echo, execution metadata, and trajectory data compared across all three frameworks with specific file formats, access methods, and layer tags. [cross-framework-synthesis.md §1]
2. **Theory-implementation boundary: OpenMM CLEAN, GROMACS SEMI-CLEAN, VASP DIRTY.** OpenMM has API-enforced separation at `ForceField.createSystem()`. GROMACS has .mdp separation with ~10 boundary parameters (dt, nsteps, rlist, etc.) requiring dual-annotation. VASP has flat INCAR namespace requiring external classification of ~200-300 tags. Twenty boundary parameters cataloged across all three frameworks. [cross-framework-synthesis.md §2]
3. **49 failure modes taxonomized across three frameworks.** OpenMM: 17 modes (5 impl, 5 methodology, 4 theory, 3 ambiguous). GROMACS: 16 modes. VASP: 16 modes. 8 common cross-framework patterns identified (numerical overflow, constraint/convergence failure, memory exhaustion, etc.) plus 7 DSL-specific modes. [cross-framework-synthesis.md §3]
4. **Trace completeness varies: OpenMM 30-40% default / 70-80% max, GROMACS 60-70% / 75-85%, VASP 50-60% / 50-60% ceiling.** VASP hits a hard ceiling due to closed-source constraints. All frameworks require custom instrumentation for three-way fault classification. [cross-framework-synthesis.md §4]
5. **Seven common IR core elements generalize across all frameworks:** timestamped events, energy time series, parameter records, error events, state snapshots, convergence trajectories, data absence records. Framework-specific elements require DSL-specific adapters. [cross-framework-synthesis.md §5]
6. **Decision Gate 1: VASP FAILS the clean-boundary test but should be accepted.** External classification table is finite, static, and domain-knowledge-based (not novel research). Dropping VASP loses the DFT domain. 70-80% of standard VASP calculations classifiable with full confidence; 20-30% have degraded confidence from ambiguous parameters. Five items flagged for adversarial review. [cross-framework-synthesis.md §6]
7. **Adapter contract defined: 7 mandatory + 7 optional methods.** The adapter interface establishes the boundary between DSL-specific parsing and common IR construction. [cross-framework-synthesis.md §5.3]

**Implications:**
- The IR cannot be a universal schema — it must be common core + adapter extensions.
- The temporal axis must be generic (step intervals for MD, SCF/ionic iterations for DFT).
- Error classification requires IR-imposed taxonomy through pattern matching (no framework provides structured error codes).
- Crash-state is unreliable across all frameworks — IR must work with "last known state" semantics.
- Step 3b (requirements refinement) can now cross-reference R1-R29 against the trace capability matrix.

**Open Threads:**
- INCAR classification table needs domain expert review before VASP adapter design is finalized.
- The "ambiguous for pathological systems" threshold for VASP parameters (ALGO, PREC) needs empirical validation.
- Whether classification tables can be partially automated (LLM-assisted documentation analysis) or are inherently manual — affects ATHENA's generalizability claim.
- Closed-source ceiling practical impact needs stress-testing with real VASP failure cases.

---

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

### 2026-02-20 — Provenance Data Models and Scientific Workflow IR Survey

**Scope:** Survey W3C PROV-DM, ProvONE, scientific workflow provenance systems (Kepler, Taverna, VisTrails, Galaxy, CWL), process mining (XES, conformance checking), and provenance query languages (SPARQL over PROV) for applicability to ATHENA's trace semantics IR. Central assessment question: can these models represent the theory-implementation distinction deterministically?

**Method:** Systematic analysis of W3C PROV-DM (§2-5), PROV-O (§3), and PROV-CONSTRAINTS (§5-8) specifications. Mapped PROV-DM's Entity-Activity-Agent model to ATHENA's theory-methodology-implementation trichotomy. Evaluated ProvONE's scientific workflow extensions (Program, Port, Channel, Controller, Workflow) for DSL simulation fit. Assessed provenance query expressiveness (SPARQL path queries for causal chain traversal). Analyzed process mining conformance checking as an expected-vs-actual comparison mechanism. Evaluated scalability at megabyte-scale traces for Rust implementation. Cataloged seven transferable patterns and five anti-patterns.

**Findings:**

1. PROV-DM's Entity-Activity-Agent model provides approximately 60-70% of ATHENA's IR requirements. Entity and Activity map well to simulation states and steps. The Agent model is the weakest mapping — it captures "who is responsible" but not "what kind of responsibility" (theory vs. implementation vs. methodology). This is the central gap.

2. PROV-DM qualified relations (§3) substantially improve resolution. Qualified Usage records *how* entities participate in activities (roles), and qualified Association with Plans provides a mechanism for encoding expected behavior (hypothesis predictions) against which actual execution can be compared. Plans are the closest PROV-DM gets to expected-vs-actual representation, but they are unstructured entities requiring ATHENA-specific formalization.

3. PROV-CONSTRAINTS provides temporal ordering, derivation chain integrity, and uniqueness constraints that can encode *some* LFI audit preconditions (particularly temporal consistency checks for Stage 1 implementation audit). It cannot encode domain-specific constraints (parameter bounds, precision requirements).

4. ProvONE's prospective/retrospective separation is the most directly relevant extension. Prospective provenance (workflow definition) maps to specification; retrospective provenance (execution trace) maps to actual execution. This provides a two-way split (specification vs. execution) rather than ATHENA's required three-way split (theory vs. methodology vs. implementation). The methodology layer is collapsed into the specification layer.

5. ProvONE's typed Ports provide a natural mechanism for parameter classification. Theory-Ports (force field parameters, equation coefficients) vs. implementation-Ports (GPU device, memory allocation) vs. methodology-Ports (sampling frequency, convergence criteria) can structurally encode the three-layer distinction at the API boundary.

6. Process mining conformance checking (alignment-based) is directly relevant to LFI Stages 1 and 2. Comparing expected process models against actual execution traces identifies structural deviations (missing steps, unexpected events) that signal implementation or methodology failures.

7. For Rust implementation: PROV-DM's RDF/SPARQL technology stack is incompatible with the throughput requirement. No mature Rust RDF triple stores exist. However, PROV-DM's *data model* (concepts and relations) can be adopted without its technology stack, implemented as a Rust-native graph structure (petgraph or custom adjacency list) with purpose-built query functions.

8. Scalability assessment: megabyte-scale traces produce 10^4 to 10^6 PROV triples. Custom Rust graph implementations handle this in milliseconds for path queries. RDF triple stores take 10-1000ms. The hybrid approach (PROV-DM concepts in Rust structures) is viable at this scale.

**Implications:**

- Decision Gate 2 outcome: PROV-DM is viable as a *conceptual foundation* but not as a *complete IR*. Three mandatory extensions are needed: (a) three-layer agent/activity/entity typing, (b) fault semantics vocabulary, (c) expected-vs-actual comparison primitives.
- The recommended approach is a hybrid: adopt PROV-DM's data model concepts (Entity, Activity, Agent, derivation chains, qualified relations, Plans) implemented in Rust-native structures, with ATHENA-specific extensions built into the core type system rather than layered as attributes.
- The theory-implementation-methodology distinction must be structural (in the type system), not attributional (in metadata). This is the single most critical design decision for the IR.
- ProvONE's typed Ports provide the most promising mechanism for encoding the three-layer distinction at DSL API boundaries.
- A novel IR designed from scratch would carry higher risk (no existing specification) but could be ATHENA-optimal. The hybrid approach trades some optimality for maturity.

**Open Threads:**

- How should theory/methodology/implementation layer assignments be determined for each DSL's API parameters? This is a per-DSL classification problem that needs investigation.
- Can conformance checking (process mining) be integrated with PROV-DM derivation chains to provide both structural and value-level deviation detection?
- What is the minimum granularity of provenance recording needed for the LFI? Full-granularity is an anti-pattern (10^8+ nodes); DSL-API-call level seems right but needs validation against actual trace data.
- ProvONE's prospective/retrospective split collapses methodology into specification. Can the prospective layer be further split into theory-prospective and methodology-prospective sub-layers? This requires investigation.

**Output:** `dsl-evaluation/provenance-workflow-ir-survey.md` — Complete survey with PROV-DM analysis, ProvONE analysis, query language assessment, workflow system survey, process mining assessment, scalability analysis, expected-vs-actual representation patterns, seven transferable patterns, and five anti-patterns.

### Entry 1 — 2026-02-20: RCA and Formal Verification IR Survey

**Scope.** Next Step 2: Survey existing IR designs in RCA and formal verification. Identify design patterns that transfer to ATHENA's trace semantics problem (translating DSL trace logs into structured representations for three-stage fault classification).

**Method.** Surveyed intermediate representations across four categories:
1. **LLM-based RCA:** arxiv:2403.04123 (LLM agents for RCA), arxiv:2601.22208 (reasoning failures in LLM RCA).
2. **Structured RCA:** Chain-of-Event (FSE 2024, typed event chains for microservice RCA), Jaeger/Zipkin (OpenTelemetry span-based distributed tracing).
3. **Formal verification IRs:** LLVM IR (SSA form), MLIR (multi-level dialect system), Boogie (specification-implementation contracts), Why3 (ghost state, theories, refinement), DRAT (machine-checkable refutation proofs), AIGER (counter-example witness traces).
4. **Program analysis:** Clang Static Analyzer (path-sensitive bug reports), Soot/WALA (JVM analysis IRs), Facebook Infer (compositional bi-abductive analysis).

Evaluated each IR against: spec-vs-execution separation, causal ordering representation, queryability, root cause ranking, and compatibility with Rust zero-copy/streaming parsing. Produced a transferable patterns catalog (13 patterns), an anti-patterns catalog (6 anti-patterns), and a prioritized recommendation.

**Findings.**

*Primary structural insight:* MLIR's dialect system is the most directly transferable pattern. It maps naturally to ATHENA's core requirement of separating theory-layer and implementation-layer trace events. Defining three dialects (theory, methodology, implementation) would give the LFI structural routing to the correct audit stage. The multi-level coexistence property means a single IR can carry all three layers simultaneously, linked by explicit lowering relationships that encode how theory-level specifications were realized by implementation-level execution.

*Second key insight:* Boogie/Why3 specification-implementation contracts provide the mechanism for the LFI's sequential audit. An experiment specification becomes a contract (requires/ensures/modifies). Stage 1 checks execution against implementation-level contract terms. Stage 2 checks whether the contract is adequate to test the hypothesis. Stage 3 checks whether contract-satisfying execution contradicts predictions. This three-level contract checking maps to the three-stage audit in ARCHITECTURE.md 5.3.

*Third key insight:* The failure modes cataloged in arxiv:2601.22208 (Stalled, Biased, Confused) map directly to IR requirements. "Stalled" requires explicit observability boundaries (Boogie's `havoc` for unobserved state). "Biased" requires evidence-backed causal chains (CoE's typed event chains with evidence links). "Confused" requires isolation of parallel causal paths within the IR structure.

*Negative finding:* LLM-based RCA systems (arxiv:2403.04123) use no formal IR — chain-of-thought reasoning serves as an implicit, non-queryable, non-reproducible "representation." The ~21% Top@1 accuracy on unstructured traces is consistent with this architectural limitation. The absence of a formal IR is the root cause of low accuracy, not insufficient LLM capability.

*Streaming compatibility:* All 13 identified transferable patterns are compatible with Rust zero-copy/streaming parsing. The three primary patterns (dialects, contracts, typed event chains) are particularly efficient: dialect tags are enum variants, contracts are parsed once from experiment specifications, and event chains are constructed incrementally.

*Anti-pattern identification:* Six anti-patterns identified, with "specification-implementation conflation" (AP2) as the most critical to avoid — it would directly disable the three-stage audit.

**Implications.**

1. The IR design is not a blank-slate research problem. Three well-established patterns from formal verification (MLIR dialects, Boogie contracts, Why3 ghost state) provide structural foundations. The research challenge is adapting these patterns to empirical trace analysis (post-execution, quantitative, streaming) rather than static/deductive verification (pre-execution, logical, batch).
2. The dialect-based layer separation pattern should be the primary structural decision for the IR. It provides the routing mechanism the LFI needs and maps directly to the DSL API separation constraint.
3. The contract pattern resolves a previously implicit requirement: the IR must carry the experiment specification alongside the trace events, as a first-class entity. Without this, Stage 2 (methodology audit) and Stage 3 (theoretical evaluation) cannot function.
4. Six open questions identified for subsequent investigations (see survey document Section 7).

**Open Threads.**

- Dialect boundaries per DSL: How to determine which trace events belong to theory/methodology/implementation for each target DSL. Requires the DSL trace format survey (Next Step 1).
- Contract extraction: Can experiment specification contracts be automatically derived from DSL experiment scripts? Determines practicality of the contract pattern.
- Streaming completeness trade-off: How much trace data must be buffered vs. streamed for each audit stage? Stage 1 may be fully streaming; Stage 3 may require the full trace.
- Quantitative refutation logic: DRAT-style refutation chains need adaptation from propositional to statistical reasoning for Stage 3.
- Ghost state validation: Methodological ghost state (sampling sufficiency, confounder control) depends on DAG quality, connecting to the bootstrapping error risk (ARCHITECTURE.md 8.3).

**Artifact.** `dsl-evaluation/rca-formal-verification-ir-survey.md` — Full survey with 13 transferable patterns, 6 anti-patterns, prioritized recommendations, and open questions.

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

**Artifact.** `dsl-evaluation/vasp-trace-analysis.md`

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

**Artifact.** `dsl-evaluation/gromacs-trace-analysis.md`

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

**Framework Trace Analysis and Baseline**

1. **All three target DSLs provide sufficient structured output for the Trace Semantics Engine to operate, but none provides sufficient *default* output for three-way fault classification.** OpenMM: only 4 of 17 failure modes detectable from default reporters. GROMACS: error messages are free-text with no classification taxonomy; most common failures (constraint violations) are inherently ambiguous. VASP: the most dangerous failures (insufficient ENCUT, inadequate k-points, wrong functional) are completely silent. Each framework requires custom instrumentation or supplementary analysis. [OpenMM log 2026-02-20; GROMACS log 2026-02-20; VASP log 2026-02-20]

2. **The theory-implementation boundary quality varies across frameworks.** OpenMM: clean API boundary at `ForceField.createSystem()`, structurally separating theory (ForceField/Topology/System) from implementation (Platform/Context). GROMACS: relatively clean .mdp parameter separation with 10+ "boundary" parameters requiring dual-annotation; mdrun command-line parameters are purely implementation-layer. VASP: boundary exists but is not API-declared; INCAR mixes theory and implementation in a flat namespace; requires external classification table (~200-300 tags). [OpenMM log 2026-02-20; GROMACS log 2026-02-20; VASP log 2026-02-20]

3. **Methodology-layer failures are invisible to all three frameworks.** No DSL framework detects or reports insufficient equilibration, inadequate sampling, wrong ensemble choice, inappropriate functional, or confounder non-control. These must be assessed by external domain rules, not parsed from trace data. [OpenMM log 2026-02-20 Finding 4; GROMACS log 2026-02-20 Finding 5; VASP log 2026-02-20 Finding 5]

4. **Correct fault classification requires multi-source correlation in every framework.** OpenMM needs per-force-group energy decomposition + reporter data + exception info. GROMACS needs .log + .edr + .tpr + structural context. VASP needs vasprun.xml + OUTCAR + stdout/stderr. No single output file/stream in any framework contains sufficient information for the LFI's three-way classification. [GROMACS log 2026-02-20 Finding 4; VASP log 2026-02-20 Finding 1; OpenMM log 2026-02-20 Finding 2]

5. **Pre-execution validation coverage varies.** GROMACS grompp catches syntactic/structural errors comprehensively but not physical correctness. OpenMM validates force field template matching at `createSystem()` but not parameter physical adequacy. VASP validates INCAR parameters against POTCAR but not convergence adequacy. This creates a consistent pattern: pre-execution catches Stage 1 syntax issues; runtime and post-run analysis handle Stages 2-3. [GROMACS log 2026-02-20 Finding 5; OpenMM log 2026-02-20 Finding 1; VASP log 2026-02-20 Finding 2]

6. **VASP's input is purely declarative; OpenMM requires Python scripting; GROMACS uses a hybrid (declarative .mdp + topology files).** This structural difference means the IR needs DSL-specific adapters rather than a universal input parser. [VASP log 2026-02-20 Finding 8; OpenMM log 2026-02-20; GROMACS log 2026-02-20]

28. **Trace completeness varies substantially: OpenMM 30-40% default / 70-80% max, GROMACS 60-70% / 75-85%, VASP 50-60% / 50-60% ceiling.** VASP hits a hard closed-source ceiling that cannot be overcome with custom instrumentation. OpenMM has the widest gap between default and instrumented coverage, meaning custom reporters provide the most marginal value. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §4]

29. **49 failure modes taxonomized: 8 harmonized cross-framework patterns, 7 DSL-specific modes.** Common patterns include: numerical overflow, constraint/convergence failure, memory exhaustion, parameter misspecification, silent methodology inadequacy. DSL-specific modes include VASP SCF non-convergence, GROMACS domain decomposition failure, OpenMM platform-dependent precision divergence. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §3]

30. **Decision Gate 1 resolved: VASP accepted with external classification table.** 70-80% of standard VASP calculations classifiable with full confidence; 20-30% have degraded confidence from ambiguous parameters (PREC, ALGO, LREAL). Five items flagged for adversarial review. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §6]

31. **Seven common IR core elements identified.** Timestamped events, energy time series, parameter records, error events, state snapshots, convergence trajectories, and data absence records generalize across all three frameworks and form the universal IR schema core. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §5]

32. **Adapter contract defined: 7 mandatory + 7 optional methods.** Mandatory: extract_parameters, extract_energy_series, extract_state_snapshots, extract_errors, extract_convergence_metrics, extract_execution_metadata, declare_data_completeness. Optional: validate_preprocessing, extract_runtime_adjustments, extract_scf_convergence, extract_electronic_structure, validate_silent_failures, extract_force_field_compilation, compare_platforms. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §5.3]

37. **The 21% Top@1 figure in VISION.md is uncited.** It carries no reference number, unlike most other claims in the document. The anchoring number for ATHENA's value proposition is unsourced. [Baseline log 2026-02-20]

38. **The 21% figure almost certainly originates from cloud/AIOps RCA benchmarks, not scientific computing.** The domain is structurally harder than ATHENA's target on every relevant dimension: trace structure, candidate set size, causal complexity, and observability. The figure is a conservative contrast, not a direct baseline. [Baseline log 2026-02-20; evidence quality B]

39. **ATHENA's three-way classification (implementation/methodology/theory) has a candidate set of 3 with random baseline 33%. Cloud RCA Top@1 operates over 50-500+ candidates with random baseline 0.2-2%.** These are fundamentally different metrics and should not be directly compared. [Baseline log 2026-02-20]

40. **Six specific structural properties of traces improve RCA accuracy** (with estimated improvements): temporal/causal ordering (+15-25%), event type taxonomies (+10-20%), schema conformance (+10-20%), causal annotations/dependency graphs (+20-35%), severity levels (+5-10%), layer/component separation (+10-15%). Improvements interact positively. [Baseline log 2026-02-20; evidence quality B]

**IR Architecture Foundations**

7. **LLM-based RCA without formal IR achieves ~21% Top@1 accuracy on unstructured traces.** The root cause is architectural (no structured representation of causal chains), not a capability limitation of the LLMs. [RCA/FV survey 2026-02-20; arxiv:2403.04123; ARCHITECTURE.md 4.5]

8. **Three LLM RCA failure modes (Stalled, Biased, Confused) map directly to IR requirements.** "Stalled" (missing context) requires explicit observability boundaries. "Biased" (prior-dominated reasoning) requires evidence-backed causal chains. "Confused" (conflated causal paths) requires structural isolation of parallel chains. [RCA/FV survey 2026-02-20; arxiv:2601.22208]

9. **MLIR's dialect system provides the structural pattern for theory/methodology/implementation separation in the IR.** Three dialects, multi-level coexistence, explicit lowering relationships between layers. This maps directly to the DSL API separation constraint and the LFI's three-stage audit routing. [RCA/FV survey 2026-02-20]

10. **Boogie/Why3 specification-implementation contracts provide the pattern for the LFI's sequential audit.** Experiment specifications as contracts (requires/ensures/modifies), checked at three levels. This resolves the implicit requirement that the IR must carry the experiment specification as a first-class entity. [RCA/FV survey 2026-02-20]

11. **Specification-implementation conflation is the most critical anti-pattern to avoid.** Representing "what was specified" and "what executed" in the same namespace directly disables the three-stage audit. [RCA/FV survey 2026-02-20]

12. **All identified transferable patterns are compatible with Rust zero-copy/streaming parsing.** Dialect tags as enum variants, contracts as structured records, event chains constructed incrementally. [RCA/FV survey 2026-02-20; ADR 001]

13. **PROV-DM covers approximately 60-70% of ATHENA's IR requirements.** Entity-Activity-Agent model maps well to simulation states and steps. The Agent model is the weakest mapping — it captures "who" but not "what kind" of responsibility. [Provenance survey 2026-02-20; W3C PROV-DM §2-5]

14. **PROV-DM does not natively represent the theory-implementation-methodology trichotomy.** The distinction must be added as a structural extension, not as metadata attributes. [Provenance survey 2026-02-20]

15. **ProvONE's typed Ports provide a natural mechanism for parameter classification at DSL API boundaries** (theory-Ports vs. implementation-Ports vs. methodology-Ports). [Provenance survey 2026-02-20]

16. **PROV-DM's RDF/SPARQL technology stack is incompatible with Rust throughput requirements, but the data model can be adopted without the technology stack.** Custom Rust graph implementations handle megabyte-scale traces (10^4-10^6 triples) in milliseconds for path queries. [Provenance survey 2026-02-20; ADR 001]

17. **Process mining conformance checking is directly applicable to LFI Stages 1 and 2** for detecting structural deviations between expected and actual execution. [Provenance survey 2026-02-20]

18. **No existing provenance system natively supports the theory-implementation distinction or fault classification** at the DSL-internal semantic level ATHENA requires. [Provenance survey 2026-02-20]

33. **MLIR dialects and PROV-DM serve complementary roles in the IR architecture.** Dialects provide classification/routing (which LFI stage handles an element); PROV-DM provides causal structure (how elements relate within each stage). The unified architecture uses dialect structure as primary organization with PROV-DM-like causal graphs within each layer. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §4]

34. **Decision Gate 2 resolved: hybrid adaptation, MEDIUM risk.** ~65-70% transfers from existing systems. ~30-35% requires novel design: three-way layer typing vocabulary, fault classification ontology, quantitative prediction-observation comparison formalization, methodology detection rules. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §5]

35. **Nine anti-patterns cataloged with severity ratings and avoidance guidance.** CRITICAL: specification-implementation conflation (directly disables three-stage audit). HIGH: post-mortem-only design (blocks streaming per ADR 001), full-granularity recording (10^8+ nodes), binary pass/fail (collapses three-way), lossy compression without principled selection. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §3]

36. **Three candidate IR designs have distinct pattern-source profiles.** LEL (Layered Event Log): simplest, Stage 1 strongest, log-based. DGR (Dual-Graph IR): natural synthesis of both surveys, Stages 2-3 strongest, graph-based. TAL (Typed Assertion Log): most ATHENA-specific, highest novelty risk, assertion-based. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §6]

41. **Hybrid LEL+DGR is the recommended IR architecture.** Scores 94/100 vs. 82/100 for either standalone candidate (LEL or DGR). Provides per-stage optimized operation: LEL streaming efficiency for Stage 1 (the common classification path) and DGR-like causal reasoning for Stages 2-3 (the differentiating path). PASS on all 9 anti-patterns. Supersedes the suspicion that DGR alone was strongest. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §9-10]

42. **The LEL→DGR incremental implementation path is viable.** Demonstrated by the Hybrid candidate's construction: LEL events carry dag_node_ref/spec_ref/causal_refs from day one; CausalOverlay built at Stage 1→2 boundary via single O(n) pass. Key constraint: LEL events must include DGR-compatible references from initial construction, pushing some entity resolution into the adapter even during Stage 1. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §4, §8 OQ2]

43. **A common structural foundation (7 shared types) is independent of candidate choice.** Layer enum, BoundaryClassification enum, ObservationMode enum, Value enum with Havoc variant, TemporalCoord struct, ProvenanceAnchor struct, ExperimentRef struct, and ConfidenceMeta struct are shared across all candidates. These can be implemented first and reused regardless of which IR representation is chosen. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §1]

44. **TAL is better as a query interface than a storage format.** The coverage matrix and candidate evaluation confirm that TAL's assertion-checking pattern functions identically as a query layer over LEL or DGR substrates. TAL's core strength (sequential audit assertions with evidence chains) does not require a standalone IR representation. Adopted as the recommended LFI query interface. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §0, §10]

45. **BoundaryClassification enum resolves the boundary parameter representation question.** Three variants (PrimaryLayer, DualAnnotated, ContextDependent) handle the full spectrum from unambiguous parameters to context-dependent ones like VASP ALGO. Primary layer determines LFI routing; secondary annotations provide diagnostic context. Avoids both a fourth "boundary" layer and entity duplication. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §1, §8 OQ4]

46. **R17 has a baseline pluggable structural slot in the prototype, but that slot is not sufficient for reward-calibrated comparison semantics.** `ComparisonResult` + `DivergenceMeasure` (6 variants) supports Stage 3 falsification routing and query execution; Step 12 shows additional uncertainty/aggregation metadata is needed for adversarial-reward and Bayesian surprise calibration. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §3, §8 OQ1; Step 12 log 2026-02-22]

47. **The causal reasoning substrate question has a per-stage answer.** Stage 1: sequential search sufficient (filter-and-inspect on implementation-tagged events). Stages 2-3: graph traversal required (transitive causal ancestry for R14 confounders, structural path finding for R18 causal implications). This per-stage resolution directly motivates the Hybrid design. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §8 OQ3]

**Requirements Analysis**

19. **The IR must represent a minimum of 29 distinct semantic elements (R1-R29) to support the LFI's three-stage audit.** Derived by backward analysis from each deterministic question the LFI must answer per ARCHITECTURE.md §5.3. [LFI requirements log 2026-02-20]

20. **The IR is not a pure trace-log derivative.** It is a composite of trace-extracted data (R1, R2, R6-R8, R12, R16), experiment specification data (R3, R4, R10, R13), hypothesis-derived data (R15), computed/derived elements (R5, R17), and DAG cross-references (R9, R11, R14, R18). [LFI requirements log 2026-02-20]

21. **R19 (layer tag) is the load-bearing structural distinction.** Without it, the three-stage sequential audit cannot function. The DSL's API separation is what makes this tagging possible. [LFI requirements log 2026-02-20; ARCHITECTURE.md §3.1]

22. **The IR must explicitly represent its own observability gaps (R26).** Silent omission of unobservable elements causes the LFI to misattribute failures. The LFI must distinguish "checked and fine" from "not checkable." [LFI requirements log 2026-02-20; ARCHITECTURE.md §8.4]

23. **Stage 2 requirements (R8-R14) are bounded by DAG accuracy.** Every confounder judgment must be traceable to specific DAG edges consulted, enabling reclassification when the DAG changes. [LFI requirements log 2026-02-20; ARCHITECTURE.md §5.3, §8.5]

24. **Stage 1 requirements (R1-R7) are fully satisfiable for all three frameworks.** OpenMM has 4 DI cells (highest instrumentation burden: no parameter echo, API-only access). GROMACS has 5 DA cells (best default coverage). VASP has 4 DA cells but exit code unreliability for SCF non-convergence. Coverage matrix confirms Stage 1 is the most tractable stage. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §1]

25. **31% of R1-R29 requirements (9 of 29) are NT — data sources external to the Trace Semantics Engine.** R9, R10, R11, R15, R18, R22, R23, R28, R29(cycle_id) come from experiment specification, hypothesis, DAG, or workflow controller. This quantifies the IR's composite nature first identified in item 20. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §5.1]

26. **R19 (layer tag) availability varies: OpenMM=DA (clean API), GROMACS=DI+ER (~10 boundary params), VASP=ER (~200-300 INCAR tags).** This is the only cross-cutting requirement with framework-dependent classification difficulty. OpenMM's clean boundary confirms the DSL constraint's value. VASP's ER burden is bounded (finite, static tag set) and accepted per Decision Gate 1. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §2]

27. **Decision Gate 4: PASS — no LFI stage blocked by FU requirements.** FU cells exist only as partial R6 (sub-component numerical internals) at ~5-10% per framework, well below the 10% threshold. Four conditions attached: OpenMM custom reporter, VASP classification table, VASP degraded confidence for ambiguous params, R17 comparison method formalization. [Coverage matrix log 2026-02-20; requirements-coverage-matrix.md §7]

**Prototype Implementation and Interface Contracts**

48. **Real CausalOverlay construction cost is empirically bounded at 10^6 scale.** With benchmark wired to `CausalOverlay::from_log`, observed overlay construction is 251.82ms at 10^6 events (22.62ms at 10^5), with 1,000,000 overlay entities and 199,998 derivation edges. Construction remains single-pass O(n) and tractable for prototype-scale traces. [Step 6 log 2026-02-21; `lel-ir-prototype/src/bench.rs`]

49. **`EventIndexes.by_id` is now implemented and removes the Phase 2 lookup blocker.** The prototype now carries `by_id: HashMap<EventId, usize>` with insert-time population and serde coverage, enabling O(1) EventId→event-position lookup during overlay construction and graph queries. [Step 6 log 2026-02-21; `lel-ir-prototype/src/lel.rs`, tests]

50. **R14 confounder detection now executes end-to-end on the overlay prototype.** `detect_confounders` performs ancestor-intersection + controlled/intervention filtering with grouped `ConfounderCandidate` outputs; 7 targeted tests validate controlled-variable exclusion, intervention exclusion, multi-confounder grouping, transitive ancestry, and unknown-variable guards. [Step 6 log 2026-02-21; `lel-ir-prototype/src/overlay.rs`, tests]

51. **R17 comparison query now executes end-to-end on the overlay prototype.** `compare_predictions` resolves `ComparisonResult` events via `by_kind`/`by_id`, parses `prediction_id` strings to `SpecElementId` at query time, joins to `spec.predictions`, and emits falsification-ready `PredictionComparison` records with DAG-node forwarding and malformed-ID fallback behavior. [Step 7 log 2026-02-21; `lel-ir-prototype/src/overlay.rs`, tests]

52. **R18 causal implication query now demonstrates three-way layer classification.** `implicate_causal_nodes` traverses ancestor paths with depth tracking, groups by DAG node, and returns stable Theory→Methodology→Implementation ordering with minimum-distance selection per node. [Step 7 log 2026-02-21; `lel-ir-prototype/src/overlay.rs`, tests]

53. **Prototype Stage 2-3 query surface is now validated end-to-end (`R14 + R17 + R18`).** The crate passes 44/44 tests with zero clippy warnings after adding comparison and implication query coverage. [Step 7 log 2026-02-21; `lel-ir-prototype/src/tests/mod.rs`]

54. **Cross-framework IR generalization remains demonstrated across GROMACS and VASP adapters using existing LEL types, and quality gates remain clean after Sessions 19-20 convergence work.** `src/gromacs_adapter.rs` and `src/vasp_adapter.rs` map MD and DFT trace sources into the same `EventKind`/LEL structures and remain compatible with `CausalOverlay::from_log` + R14 query behavior. Crate quality gates now pass at 119/119 tests with strict clippy clean. [Step 9 log 2026-02-21; Step 10 log 2026-02-22; Session 19 log 2026-02-24; Session 20 log 2026-02-23; `lel-ir-prototype/src/tests/mod.rs`]

55. **WDK#12 is resolved in prototype scope: one IR schema accommodates both DFT and MD traces.** Step 10 VASP adapter implementation required no core IR type changes and passed integration/overlay tests alongside OpenMM and GROMACS paths. [Step 10 log 2026-02-22; `lel-ir-prototype/src/vasp_adapter.rs`, `lel-ir-prototype/src/tests/mod.rs`]

56. **`ConvergencePoint` and `StateSnapshot` are now exercised by a concrete adapter path.** Step 10 VASP parsing emits `ConvergencePoint` from OSZICAR SCF trajectories and `StateSnapshot` from OUTCAR force snapshots, closing prior coverage gaps for these variants. [Step 10 log 2026-02-22; `lel-ir-prototype/src/vasp_adapter.rs`, `lel-ir-prototype/src/tests/mod.rs`]

57. **Hidden confounder detection is now validated end-to-end on VASP-derived LEL data.** Step 11 litmus tests demonstrate positive detection of planted `PREC` confounders and correct controlled-variable exclusion behavior. [Step 11 log 2026-02-22; `lel-ir-prototype/src/tests/mod.rs`]

58. **R17 formalization is narrowed to a recommended default: Multi-Metric Divergence Profile (Candidate B) with optional posterior hooks.** This satisfies ATHENA's bounded-adversarial requirements by combining scalar optimization with component-level calibration visibility, while preserving migration compatibility toward distribution-aware reward models. [Step 12 log 2026-02-22; ARCHITECTURE.md §5.4]

59. **A trace-semantics -> adversarial-reward interface contract is now specified at the findings level.** The reward track can assume deterministic comparison profiles with monotonic aggregate semantics, explicit validity gating, and provenance-backed metric components, without requiring immediate prototype integration. [Step 12 log 2026-02-22; ARCHITECTURE.md §4.5, §5.4]

60. **Current prototype comparison types are insufficient for calibrated information-gain reward in their present form.** `ComparisonOutcome` + scalar `DivergenceMeasure` + `is_falsified` do not encode uncertainty/support metadata required by Bayesian design and active-learning style reward calibration. [Step 12 log 2026-02-22; `lel-ir-prototype/src/common.rs:134-150`; `lel-ir-prototype/src/overlay.rs:230-277`]

61. **Convergence evidence is now represented across all three adapter paths: native in VASP and derived in GROMACS/OpenMM.** VASP still emits native `ConvergencePoint` trajectories from OSZICAR; Session 19 adds adapter-inline derived `ConvergencePoint` summaries for GROMACS/OpenMM from existing `EnergyRecord`/`NumericalStatus`/`ExecutionStatus` streams with explicit minimum-data guards and provenance refs. [Step 13 log 2026-02-22; Session 19 log 2026-02-24; `lel-ir-prototype/src/vasp_adapter.rs`; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`]

62. **Current indexing does not make convergence trajectories first-class query targets.** `EventIndexes.by_variable` indexes only `ParameterRecord` and `ObservableMeasurement`, and VASP convergence events are emitted without `dag_node_ref`, so convergence lookups require stream scans or post-hoc derivation. [Step 13 log 2026-02-22; `lel-ir-prototype/src/lel.rs:130-141`; `lel-ir-prototype/src/vasp_adapter.rs:190-211`]

63. **External ecosystem practice converges on layered representation (raw trajectory + derived convergence indicators), not pattern-only primacy.** Pymatgen exposes both convergence booleans and ionic/electronic step data; ASE exposes optimizer convergence booleans plus trajectory persistence; Custodian handlers evaluate derived convergence features from raw VASP artifacts. [Step 13 log 2026-02-22; `https://pymatgen.org/pymatgen.io.vasp.html`; `https://ase.gitlab.io/ase/ase/optimize.html`; `https://materialsproject.github.io/custodian/custodian.vasp.handlers.html`]

64. **WDK#13 is narrowed to a concrete direction: hybrid Option D (raw canonical events + Stage 1->2 `ConvergenceSummary` derivation).** This preserves silent-failure analyzability and enables consumer-specific features/patterns without forcing adapters to classify patterns at parse time. [Step 13 log 2026-02-22; `lel-ir-prototype/src/overlay.rs:54-84`; ARCHITECTURE.md §5.3, §5.4]

65. **Convergence summaries can map cleanly to Step 12 Candidate B (`ComparisonProfileV1`) as multi-component metrics.** The required shape is `metrics: Vec<MetricComponent>` with optional uncertainty/provenance hooks, allowing Stage 3 and Bayesian-surprise consumers to avoid scalar collapse. [Step 12 log 2026-02-22; Step 13 log 2026-02-22; `FINDINGS.md` Step 12 contract block]

66. **External UQ ecosystem practice aligns with a layered uncertainty shape (point summary + optional richer distributional payload), not scalar-only uncertainty metadata.** ASME V&V/VVUQ descriptions, ArviZ `summary`, SALib Sobol outputs, and UQpy posterior/distribution APIs all expose point-level summaries while allowing richer uncertainty structure when available. [Step 14 log 2026-02-22; `https://www.asme.org/codes-standards/find-codes-standards/standard-for-verification-and-validation-in-computational-fluid-dynamics-and-heat-transfer`; `https://www.asme.org/codes-standards/find-codes-standards/the-role-of-uncertainty-quantification-in-verification-and-validation-of-computational-solid-mechanics-models`; `https://python.arviz.org/en/v0.21.0/api/generated/arviz.summary.html`; `https://salib.readthedocs.io/en/latest/api.html`; `https://uqpyproject.readthedocs.io/en/stable/inference/bayes_parameter_estimation.html`]

67. **Candidate C (layered point summary + optional tagged distribution payload) is the strongest WDK#40 direction under ATHENA's priority ordering.** It is the only candidate that covers all six consumers while preserving explicit missingness semantics (G5) and avoiding forced point-vs-distribution trade-offs. [Step 14 log 2026-02-22; Step 12 contract block; ARCHITECTURE.md §4.5, §5.4, §6.1-§6.2]

68. **Cross-adapter feasibility for uncertainty metadata is asymmetric but compatible with one shared type when distribution payload is optional.** VASP/GROMACS/OpenMM can all populate the point layer immediately; richer distribution payloads remain adapter/reporter dependent and can be omitted explicitly without branching by adapter identity. [Step 14 log 2026-02-22; `lel-ir-prototype/src/vasp_adapter.rs`; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`]

69. **WDK#40 is narrowed to a concrete schema contract at findings level: mandatory point uncertainty, optional tagged distribution payload, explicit `NoUncertainty` reason.** This is sufficient to support both V&V/effect-size reporting and Bayesian/adversarial calibration from one `ComparisonProfileV1` metric interface. [Step 14 log 2026-02-22]

70. **`MetricComponent.uncertainty` and `ConvergenceSummary.uncertainty` should share the same numeric uncertainty core, while convergence-pattern confidence remains separate pending WDK#42-44.** This preserves Step 13 compatibility without prematurely freezing pattern-taxonomy semantics. [Step 14 log 2026-02-22; Step 13 log 2026-02-22; WDK#42-44]

**VASP-Specific Extensions**

71. **WDK#35 resolved: `ContractTerm` needs `value: Option<Value>` for machine-readable precondition checking.** Five concrete VASP precondition categories (POTCAR family, ENCUT threshold, KPOINTS density, ISMEAR type, POSCAR consistency) are all representable with existing Value variants. Checking logic belongs in adapters/LFI, consistent with the `ControlledVariable.held_value` pattern. No new Value types needed for preconditions. [WDK#35+#36 log 2026-02-21; `common.rs:94-99,106,115,123`]

72. **WDK#36 resolved: Value enum needs two new variants — `KnownGrid` for inline spectral data and `DataRef` for volumetric references.** `KnownGrid { axes, values, value_unit }` covers band structure and DOS (typically <10MB); `DataRef { path, data_type, shape, unit }` covers CHGCAR/LOCPOT/PROCAR (10MB-100GB). Follows the existing pattern of inline small data + referenced large data (`StateSnapshot.data_ref`). OpenMM and GROMACS do not produce spectral data — these extensions are VASP/DFT-specific. [WDK#35+#36 log 2026-02-21; `common.rs:201-213`; `vasp-trace-analysis.md:29-34`]

73. **WDK#39 resolved: `ComparisonResult.prediction_id` should be `SpecElementId`, not `String`.** Zero adapter impact because `ComparisonResult` is a derived event type — no adapter constructs it. Aligns with all four spec element types (`ContractTerm`, `PredictionRecord`, `InterventionRecord`, `ControlledVariable`) and the ComparisonProfileV1 forward design. The `String` type was a placeholder, not a deliberate design choice. [WDK#39 log 2026-02-21; `event_kinds.rs:86,88`; `common.rs:96,104,113,121`]

74. **WDK#25 narrowed: VASP closed-source ceiling impact is bounded at ~20-30% of failure instances, with asymmetric distribution.** Three-tier observability classification across 16 failure modes: Tier A (fully observable) covers 7 modes, Tier B (partially observable with heuristics) covers 6 modes, Tier C (hard ceiling) covers 3 modes plus internal numerical issues. Weighted across workflow types: ~70-80% of failures allow fault isolation from external outputs. The ceiling concentrates on strongly correlated systems (~35-50% unobservable) and is minimal for bulk metals (~10-15%). [WDK#25 log 2026-02-21; `cross-framework-synthesis.md §3.3, §6.3`]

75. **WDK#26 narrowed: INCAR classification table covers ~20% of commonly used parameters explicitly and has two known gaps (ADDGRID, NBANDS).** Six additional ambiguous parameters identified (ISYM, SYMPREC, LASPH, LMAXMIX, ENAUG, NGX/NGY/NGZ), bringing total ambiguous count to ~12. Classification strategy recommendation: evolve from flat static lookup to static table with context-dependent flags (Strategy B), separating "what conditions matter" (static, testable) from "what conditions hold" (dynamic, requires POSCAR/POTCAR/KPOINTS context). The 70-80% confidence estimate from Decision Gate 1 remains plausible. [WDK#26 log 2026-02-21; `vasp_adapter.rs:12-96`; `cross-framework-synthesis.md §2.3`]

76. **WDK#41 is now resolved cross-track and no longer blocks trace-semantics synthesis.** Bead `athena-apb` is closed with explicit adversarial-reward evidence chain, and trace-semantics Accumulated Findings now treat WDK#41 as resolved with locked recommendation and architecture-integration citations. [Session 19 log 2026-02-24; `bd show athena-apb --json`; `research/adversarial-reward/FINDINGS.md`; `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md`]

77. **WDK#43 is resolved in prototype scope: GROMACS and OpenMM now derive convergence summaries from existing event streams without synthetic certainty.** Both adapters emit derived `ConvergencePoint` events from observed energy trajectories and existing terminal/numerical events, rather than inventing hidden signals. [Session 19 log 2026-02-24; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`]

78. **Minimum-data and classification rules for convergence derivation are now explicit and test-backed.** Minimum input condition is a 4-point energy window; classification uses relative-delta thresholds (`1e-4`) to separate convergence (`max_rel_delta`), oscillation (`sign_changes` + `mean_rel_delta`), and stall (`mean_rel_delta` without oscillation). [Session 19 log 2026-02-24; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`; tests]

79. **Derived convergence summaries preserve provenance and uncertainty structure.** `ConvergencePoint.causal_refs` include contributing source event IDs (energy window + execution/numerical context), and confidence metadata is explicitly `Completeness::Derived { from_elements }`, preserving evidence traceability for downstream query layers. [Session 19 log 2026-02-24; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`; tests]

80. **WDK#42 is resolved in prototype scope: canonical convergence taxonomy is implemented as a projection layer over existing events.** `CanonicalConvergence` now maps adapter-native and derived convergence signals into shared patterns (`Converged`, `Oscillating`, `Stalled`, `Divergent`, `InsufficientData`) with explicit confidence handling and divergence-priority override. [Session 20 log 2026-02-23; `lel-ir-prototype/src/convergence.rs`; `lel-ir-prototype/src/tests/mod.rs`]

81. **WDK#44 is resolved in prototype scope: convergence derivation remains adapter-inline while duplication is eliminated through shared extraction.** Shared utility `derive_energy_convergence_summary` centralizes GROMACS/OpenMM derivation mechanics without introducing a Stage 1->2 post-pass architecture, preserving natural provenance and reducing maintenance divergence. [Session 20 log 2026-02-23; `lel-ir-prototype/src/convergence.rs`; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`]

82. **OpenMM StateDataReporter CSV support is now test-validated with backward compatibility and divergent-signal emission.** CSV and whitespace energy formats both map into the same derived convergence path; non-finite energies now emit numerical-status events required for canonical Divergent override. [Session 20 log 2026-02-23; `lel-ir-prototype/src/adapter.rs`; `lel-ir-prototype/src/tests/mod.rs`]

### What We Suspect

**DSL Trace Architecture**

1. **A custom ATHENA reporter for OpenMM capturing per-force-group energy decomposition would resolve most NaN ambiguity.** The OpenMM API supports `getState(groups={i})` for energy decomposition; overhead is untested. [OpenMM log 2026-02-20]

2. **The atom type assignment gap at OpenMM's `createSystem()` is a tractable engineering problem,** recoverable via instrumentation or post-hoc comparison of System parameters against ForceField XML. [OpenMM log 2026-02-20]

3. **The `dt` (timestep) parameter may be the single most diagnostic boundary parameter for GROMACS fault classification.** Wrong dt is the most common LINCS failure cause, and produces symptoms (constraint violation, energy explosion) that appear to be implementation failures but are actually methodology errors. [GROMACS log 2026-02-20]

4. **An error pattern library for GROMACS (regex on error messages) may suffice for prototyping but is likely too brittle for production** across GROMACS versions. [GROMACS log 2026-02-20]

5. **Silent theory failures (insufficient ENCUT, inadequate k-points, inappropriate functional) may constitute a significant fraction of real VASP failures,** making domain-aware validation rules essential. [VASP log 2026-02-20]

6. **Custodian's error handler catalog could serve as a foundation for the rule-based validation layer** in the Trace Semantics Engine for VASP. [VASP log 2026-02-20]

7. **The IR will need DSL-specific adapters rather than a universal schema,** because VASP's multi-file declarative input, OpenMM's Python scripting, and GROMACS's MDP/topology format differ structurally. [VASP log 2026-02-20; confirmed across all three surveys]

**IR Design**

8. **The hybrid approach (PROV-DM data model concepts in Rust-native structures with ATHENA-specific extensions) likely offers the best risk/reward tradeoff.** Captures W3C standard maturity without RDF performance costs. [Provenance survey 2026-02-20]

9. **The theory-implementation-methodology distinction should be structural (in the type system) rather than attributional (in metadata).** Attribute-based encoding forces every LFI query to filter by metadata, adding complexity and ambiguity. [Provenance survey 2026-02-20; RCA/FV survey 2026-02-20]

10. **The dialect boundary definition will be the hardest per-DSL adaptation problem.** Determining which trace events belong to theory/methodology/implementation for each target DSL requires deep understanding of each framework's API structure. [RCA/FV survey 2026-02-20]

11. **Stage 3 (theoretical evaluation) may require full-trace buffering, breaking the streaming model.** Theoretical predictions are evaluated against aggregate outcomes. Stages 1 and 2 can likely operate in streaming mode. [RCA/FV survey 2026-02-20]

12. **Ghost state for methodological metadata inherits DAG quality problems.** If the causal DAG is wrong about confounders, methodological ghost state will encode incorrect claims, propagating the bootstrapping error. [RCA/FV survey 2026-02-20; ARCHITECTURE.md 8.3]

13. **Contract extraction from DSL experiment scripts may be partially automatable** — DSL APIs have typed parameter specifications that could serve as preconditions. Postconditions likely require manual or LLM-assisted specification. [RCA/FV survey 2026-02-20]

**Requirements and Baseline**

14. ~~**Stage 1 requirements (R1, R2, R6, R7) are the most tractable.**~~ PROMOTED to What We Know #24. Coverage matrix confirms: Stage 1 is fully satisfiable for all three frameworks, with OpenMM needing the most instrumentation (4 DI cells) and GROMACS having the best default coverage (5 DA cells). [Coverage matrix log 2026-02-20]

15. **R28 (interventional vs. observational distinction) may be a gap in ARCHITECTURE.md §5.3.** The audit description does not explicitly require it, but the hidden confounder litmus test cannot be passed without it. [LFI requirements log 2026-02-20]

16. **A shared variable naming ontology between the Trace Semantics Engine and the Causal Graph Manager is an implicit requirement** (from R9, R11, R14) not addressed by any current research investigation. [LFI requirements log 2026-02-20]

17. **The IR must preserve at least five structural properties to maintain DSL advantage over unstructured traces:** theory/implementation layer separation, typed parameters with physical constraints, causal/temporal ordering, DSL-level operation semantics, and pre-execution validation results. [Baseline log 2026-02-20; evidence quality C]

18. **DSL-constrained RCA should achieve 55-75% Top@1 accuracy** on the same failure types that score 21% on unstructured traces. Speculative but grounded in structural analysis. [Baseline log 2026-02-20; evidence quality C]

19. **Residual hard cases (10-25%) cluster into theory-theory interactions, subtle methodology insufficiency, emergent numerical failures, and multi-component interaction failures.** These require the causal DAG and Bayesian Surprise Evaluator, not just the IR. [Baseline log 2026-02-20; evidence quality C]

**Cross-Framework and IR Synthesis**

20. ~~**DGR (Dual-Graph IR) is likely the strongest candidate for Step 5a.**~~ PROMOTED to What We Know #41. Full candidate evaluation confirms Hybrid LEL+DGR is the recommended architecture, combining LEL streaming efficiency for Stage 1 with DGR causal reasoning for Stages 2-3. Scores 94/100 vs. 82/100 for either standalone candidate. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §9-10]

21. ~~**The unified architecture can likely be incrementally implemented.**~~ PROMOTED to What We Know #42. The Hybrid candidate proves incremental implementation by construction: LEL core for Stage 1, CausalOverlay added at Stage 1→2 boundary via O(n) pass. Key constraint: LEL events must carry dag_node_ref/spec_ref/causal_refs from day one for overlay construction. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §4, §8 OQ2]

22. **Classification tables for new DSL frameworks may be partially automatable** via LLM-assisted documentation analysis, reducing the per-DSL engineering cost. Untested. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §6.4]

23. **The adapter optional methods (validate_silent_failures, extract_scf_convergence, etc.) may evolve into mandatory requirements** as empirical testing reveals which framework-specific data is essential for correct fault classification. [Cross-framework synthesis log 2026-02-20; cross-framework-synthesis.md §5.3]


### What We Don't Know

**DSL-Specific**

1. **Whether sub-step instrumentation is achievable via OpenMM's `CustomIntegrator`** to close the temporal gap between reporter intervals. [OpenMM log 2026-02-20]

2. **The computational overhead of per-force-group energy decomposition** at every reporting interval in OpenMM. [OpenMM log 2026-02-20]

3. **How the IR should represent temporal gaps** ("state unknown between timestep X and Y") formally. [OpenMM log 2026-02-20]

4. **Whether GROMACS can produce a complete state dump at crash time.** Default behavior preserves only the last periodic checkpoint. [GROMACS log 2026-02-20]

5. **How GROMACS runtime auto-tuning (nstlist, rlist, PME) affects trace reproducibility** and whether the IR must track divergent auto-tuning. [GROMACS log 2026-02-20]

6. **What fraction of real-world VASP failures are "silent" vs. self-announcing.** Materials Project workflow logs might provide statistics. [VASP log 2026-02-20]

7. **Whether the VASP 6 REPORT file significantly closes the implementation-layer observability gap.** [VASP log 2026-02-20]

8. **Whether panedr's DataFrame (GROMACS) can serve as direct IR input** or needs abstraction for cross-framework compatibility. [GROMACS log 2026-02-20]

**IR Design**

9. **How to adapt quantitative/statistical refutation logic into a machine-checkable chain structure.** DRAT-style chains are propositional; scientific falsification is probabilistic. [RCA/FV survey 2026-02-20]

10. **How to handle trace events that span multiple dialects** (operations involving both theory-level and implementation-level concerns simultaneously). [RCA/FV survey 2026-02-20]

11. **What the minimum granularity of provenance recording is** that still enables correct fault classification. Full-granularity is an anti-pattern; DSL-API-call level needs validation. [Provenance survey 2026-02-20]

**Requirements and Baseline**

14. **Whether the IR should be a single composite structure or a set of joinable structures with defined interfaces.** The composite nature creates a design tension between cohesion and modularity. [LFI requirements log 2026-02-20]

15. **What classification confidence threshold (R25) separates determinate from ambiguous classifications.** This is an LFI design question, but the IR must provide input data. [LFI requirements log 2026-02-20]

16. **How cross-experiment queryability (R29) interacts with IR storage and lifecycle.** Single-experiment IR is simpler; multi-experiment requires aggregation decisions. [LFI requirements log 2026-02-20]

17. **Whether derived IR elements (R5, R17) should be stored or computed on demand.** Affects queryability performance. [LFI requirements log 2026-02-20]

18. **The exact source paper, dataset, and methodology behind the 21% figure.** Until verified, it should be treated as approximate with domain non-transferability noted. [Baseline log 2026-02-20]

19. **The candidate set size used in the 21% evaluation,** which determines whether 21% represents ~10x or ~100x above random chance. [Baseline log 2026-02-20]

20. **Whether scientific computing-specific failure analysis literature provides more directly transferable baselines** than cloud/AIOps RCA work. [Baseline log 2026-02-20]

21. **The actual RCA accuracy achievable on DSL-structured traces** — all estimates are speculative until an empirical prototype is built. [Baseline log 2026-02-20]

22. **How the success criterion should be reframed** as three-way classification accuracy rather than direct comparison to cloud RCA Top@1. [Baseline log 2026-02-20]

30. **The per-force-group energy decomposition overhead in OpenMM (R6 DI).** This is the largest unknown affecting OpenMM adapter feasibility. If overhead is prohibitive, alternative R6 strategies are needed (e.g., statistical anomaly detection on total energy only). [Coverage matrix log 2026-02-20; What We Don't Know #2]

**Cross-Framework and IR Synthesis**

27. **What the streaming/buffering trade-off is for Stage 3.** LEL is fully streaming; DGR may require partial graph buffering; TAL may require assertion reordering. Depends on how often Stage 3 needs full-trace access vs. phase-level summaries. [IR synthesis log 2026-02-20; ir-pattern-catalog.md §7]

38. **Whether arena allocation provides measurable benefit for CausalOverlay construction.** NARROWED: Vec-first allocation is now validated on the real overlay path at 10^6 scale (`CausalOverlay::from_log` = 251.82ms). Arena remains deferred and should only be introduced if future profiling shows measurable allocation overhead in broader workloads. [Step 6 log 2026-02-21; `lel-ir-prototype/src/bench.rs`]

**Candidate IR Schemas**

**Resolved / Narrowed — No Longer Blocking**

12. ~~**Whether a single IR schema can accommodate both DFT codes (VASP) and MD codes (OpenMM, GROMACS)** or whether structural differences require fundamentally different IR designs with a common interface.~~ RESOLVED: Step 10 VASP adapter maps INCAR/OSZICAR/OUTCAR into existing LEL/EventKind structures with no schema changes, alongside existing OpenMM/GROMACS paths. See What We Know #55. [Step 10 log 2026-02-22; `lel-ir-prototype/src/vasp_adapter.rs`, `lel-ir-prototype/src/tests/mod.rs`]

13. ~~**How convergence trajectories should be represented in the IR** (raw time series, classified patterns, or derived features).~~ NARROWED: Step 13 recommends Option D (hybrid raw canonical trajectory + Stage 1->2 summary derivation with derived features, optional pattern classification, and provenance anchors). Session 19 resolves cross-framework derivation rules for GROMACS/OpenMM and narrows remaining uncertainty to pattern taxonomy/confidence and long-run placement strategy. See What We Know #61-#65, #77-#79 and WDK items #42 and #44. [Step 13 log 2026-02-22; Session 19 log 2026-02-24]

23. ~~**Which causal reasoning substrate best matches the LFI's actual query patterns.**~~ RESOLVED: per-stage answer. Stage 1: sequential search sufficient. Stages 2-3: graph traversal required. See What We Know #47. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §8 OQ3]

24. ~~**How boundary parameters should be represented in a dialect-based IR.**~~ RESOLVED: BoundaryClassification enum with PrimaryLayer/DualAnnotated/ContextDependent variants. See What We Know #45. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §1, §8 OQ4]

25. **The practical impact of VASP's closed-source ceiling.** ~~How often does correct fault classification require information not present in vasprun.xml + OUTCAR + stdout? Needs stress-testing with real VASP failure cases.~~ NARROWED: Three-tier observability classification (A/B/C) across 16 failure modes estimates ~70-80% weighted fault isolation from external outputs. Degradation is asymmetric: ~10-20% for routine workflows (bulk metals), ~35-50% for challenging workflows (strongly correlated systems). Remaining uncertainty: per-workflow frequency estimates are conjectural and need empirical validation against real VASP failure corpora. [WDK#25 log 2026-02-21; cross-framework-synthesis.md §6.4]

26. **Whether the INCAR classification table is correct and complete.** ~~The tag-level classification (theory/implementation/ambiguous) for ~200-300 INCAR parameters needs domain expert validation, particularly for context-dependent tags.~~ NARROWED: Prototype covers 16 of ~50-80 commonly used parameters; 2 of 6 identified ambiguous parameters (ADDGRID, NBANDS) are missing. Six additional ambiguous parameters identified (ISYM, SYMPREC, LASPH, LMAXMIX, ENAUG, NGX/NGY/NGZ). Classification strategy recommendation: static table with context-dependent flags (Strategy B). Remaining uncertainty: the 6 newly identified parameters need domain expert validation; ALGO secondary layer (Methodology vs Theory) needs reconsideration for pathological systems; version-specific behavior catalog needed. [WDK#26 log 2026-02-21; cross-framework-synthesis.md §6.4]

28. ~~**How to formalize the quantitative prediction-observation comparison method (R17).**~~ NARROWED: Step 12 evaluated three candidate formalizations and recommends Candidate B (Multi-Metric Divergence Profile) plus optional posterior hooks, with a concrete trace-semantics -> adversarial-reward interface contract and explicit downstream scoring. Remaining uncertainty is implementation detail (uncertainty schema + aggregation calibration), not baseline formalization direction. [Step 12 log 2026-02-22]

29. ~~**Whether the LEL→DGR incremental implementation path is viable.**~~ RESOLVED: Yes. The Hybrid candidate demonstrates viability by construction. See What We Know #42. [Candidate IR schemas log 2026-02-20; candidate-ir-schemas.md §4, §8 OQ2]

31. ~~**DGR overlay construction cost at the Stage 1/2 boundary for megabyte-scale traces.**~~ RESOLVED: Empirically bounded with real `CausalOverlay::from_log` benchmark path at 251.82ms for 10^6 events (22.62ms at 10^5), with 1,000,000 overlay entities and 199,998 derivation edges. See What We Know #48. [Step 6 log 2026-02-21; `lel-ir-prototype/src/bench.rs`]

32. ~~**Whether HybridIR events need full DGR-compatible references from day one.**~~ NARROWED: "From day one" confirmed as safer default. Deferred resolution is a viable escape hatch via O(n) reference map pass at Stage 1→2 boundary. Remaining question: is the two-phase adapter protocol acceptable complexity for specific adapters? This is an adapter API design decision, not an IR correctness question. [Open thread resolution log 2026-02-21; LEL prototype evidence]

33. ~~**Whether the ExperimentSpec struct is sufficient for all three frameworks.**~~ NARROWED: Sufficient for all three at Stage 1. Two specific VASP Stage 3 gaps identified: (a) `ContractTerm` needs `value: Option<Value>` for machine-readable precondition checking; (b) `PredictionRecord.predicted_value` needs `KnownMatrix` or function variant in `Value` for spectral data. See items #35, #36 below. [Open thread resolution log 2026-02-21; DSL surveys]

34. ~~**Whether the OverlayEntity is sufficient for Stage 2-3 queries.**~~ RESOLVED (prototype scope): Lightweight OverlayEntity supports implemented Stage 2 confounder traversal (R14) with `by_id` lookup in place. R17/R18 remain design-aligned and unblocked by structure. See What We Know #49 and #50. [Step 6 log 2026-02-21; `lel-ir-prototype/src/overlay.rs`]

35. ~~**Whether `ContractTerm` needs a `value: Option<Value>` field** for machine-readable precondition checking in VASP Stage 3 (e.g., POTCAR family = PBE). Currently `ContractTerm` has only `description: String`. Non-blocking for OpenMM/GROMACS Stage 1.~~ RESOLVED (design level): Yes — `value: Option<Value>` is the correct design. Five concrete VASP precondition categories identified (POTCAR family, ENCUT threshold, KPOINTS density, ISMEAR type, POSCAR consistency), all representable with existing Value variants. Checking logic belongs in adapters/LFI, not the IR schema, consistent with the `ControlledVariable.held_value` pattern. [WDK#35+#36 log 2026-02-21; common.rs:94-99]

36. ~~**Whether `Value` enum needs a `KnownMatrix` or function variant** for VASP spectral data (band structure over k-points). `PredictionRecord.predicted_value: Value` cannot represent spectral predictions with current variants. Non-blocking for OpenMM Stage 1.~~ RESOLVED (design level): Yes — two new variants recommended: `KnownGrid` (inline spectral data: band structure, DOS) and `DataRef` (volumetric references: CHGCAR, LOCPOT, PROCAR). Follows existing pattern of inline small data + referenced large data (`StateSnapshot.data_ref`). `ValueType` enum needs corresponding extension. Non-blocking for Stage 1/2. [WDK#35+#36 log 2026-02-21; common.rs:201-213]

37. ~~**Whether `EventIndexes` needs a `by_id: HashMap<EventId, usize>` index** for O(1) event lookup by ID.~~ RESOLVED: Implemented in prototype (`EventIndexes.by_id`) with insert-time population and test/serde coverage. See What We Know #49. [Step 6 log 2026-02-21; `lel-ir-prototype/src/lel.rs`, tests]

39. ~~**Whether `EventKind::ComparisonResult.prediction_id: String` should be harmonized with `PredictionRecord.id: SpecElementId` in production.**~~ RESOLVED (design level): Yes — change `ComparisonResult.prediction_id` to `SpecElementId` (Option a). Zero adapter impact since `ComparisonResult` is a derived event type (no adapter constructs it). Aligns with all four spec element types and ComparisonProfileV1 forward design. Parse-at-query-time workaround has four failure modes (silent mismatch, partial-success, whitespace, overflow). Change scope: 1 type in `event_kinds.rs`, 1 method in `overlay.rs`, 6 test sites. [WDK#39 log 2026-02-21; `event_kinds.rs:88`; `overlay.rs:261`]

40. ~~**What minimal `UncertaintySummary` schema should accompany each divergence metric** so one comparison profile can support both V&V/effect-size reporting and Bayesian/active-learning reward calibration without adapter-specific branching.~~ NARROWED: Step 14 recommends Candidate C (layered point uncertainty + optional tagged distribution payload + explicit `NoUncertainty` reason) as the minimal adapter-agnostic direction. Remaining work is implementation/field canonicalization and downstream pattern-taxonomy dependencies (WDK#42-44), not baseline schema shape. [Step 14 log 2026-02-22]

41. ~~**How to standardize profile aggregation into a bounded, monotonic reward scalar** that remains calibratable under ARCHITECTURE.md §5.4 feedback and robust against Noisy-TV style reward hacking.~~ RESOLVED: Work completed in adversarial-reward track. See `research/adversarial-reward/FINDINGS.md` WDK#41 Sessions 1-7, locked recommendation at `research/adversarial-reward/prototypes/aggregation-candidates/aggregate_score_recommendation.md`, and architecture integration in Session 7.

42. ~~**What minimal cross-framework `ConvergencePattern` taxonomy should be adopted, and how pattern confidence should be calibrated.**~~ RESOLVED (prototype scope): Session 20 implements canonical taxonomy projection in `CanonicalConvergence` (`Converged`, `Oscillating`, `Stalled`, `Divergent`, `InsufficientData`) with code-backed mapping and confidence semantics in `classify_convergence`/`classify_all_convergence`, plus cross-framework scenario tests. [Session 20 log 2026-02-23; `lel-ir-prototype/src/convergence.rs`; `lel-ir-prototype/src/tests/mod.rs`]

43. ~~**How to derive convergence summaries for GROMACS/OpenMM when no native `ConvergencePoint` stream exists.**~~ RESOLVED (prototype scope): Session 19 implements adapter-inline derived `ConvergencePoint` emission for GROMACS and OpenMM from existing `EnergyRecord`, `NumericalStatus`, and `ExecutionStatus` streams, with explicit minimum-data condition (`window >= 4`), converged/non-converged outcomes, and provenance-carrying causal references validated by tests. [Session 19 log 2026-02-24; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`; `lel-ir-prototype/src/tests/mod.rs`]

44. ~~**Where convergence summary computation should occur and how it should attach graph/query anchors.**~~ RESOLVED (prototype scope): Session 20 confirms adapter-inline derivation as the selected placement, with shared utility extraction (`derive_energy_convergence_summary`) used by both GROMACS and OpenMM adapters. This preserves natural parse-time provenance and avoids Stage 1->2 post-pass architecture cost while eliminating duplicated code paths. [Session 20 log 2026-02-23; `lel-ir-prototype/src/convergence.rs`; `lel-ir-prototype/src/gromacs_adapter.rs`; `lel-ir-prototype/src/adapter.rs`]

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| `codex-prompt-5b-lel-prototype.md` | Codex prompt to produce the LEL IR Rust crate prototype (Step 5b) | Complete | Specifies LEL core types (§1/§2), OpenMM mock adapter, builder helpers, 11 unit tests; validates event typing, layer tagging, spec separation, Hybrid upgrade path fields |
| `lel-ir-prototype/` | LEL + Hybrid CausalOverlay Rust prototype crate | Complete | Compiles clean, 119/119 tests pass, clippy zero warnings. Validates: event typing (12 EventKind variants), layer tagging, spec separation (AP1 avoidance), serde roundtrip, `by_id` indexing, CausalOverlay construction/traversal, Stage 2-3 query behavior (`R14 + R17 + R18`), Session 19 convergence-summary derivation/provenance tests, and Session 20 canonical taxonomy + OpenMM CSV convergence coverage across OpenMM, GROMACS, and VASP adapters. |
| `lel-ir-prototype/src/overlay.rs` | CausalOverlay implementation (Steps 6-7) | Complete | Implements index-only overlay entities, `from_log` O(n) construction, `transitive_ancestors` BFS traversal, private `ancestors_with_depth`, `detect_confounders` (R14), `compare_predictions` (R17), and `implicate_causal_nodes` (R18). |
| `lel-ir-prototype/src/gromacs_adapter.rs` | GROMACS `.mdp`/`.log` parser, DslAdapter impl | Complete | Cross-framework IR generalization: maps GROMACS traces to existing LEL `EventKind`s, preserves provenance, wires causal refs, and (Session 19) derives convergence summaries from existing `EnergyRecord`/`NumericalStatus`/`ExecutionStatus` streams with explicit minimum-window and trend/oscillation rules. |
| `lel-ir-prototype/src/adapter.rs` | DslAdapter trait + mock OpenMM adapter | Complete | Defines adapter interface and (Session 19) extends mock OpenMM path with reporter-like energy-series parsing plus derived convergence-summary emission under the same minimum-window and uncertainty-preserving rules used for GROMACS. |
| `lel-ir-prototype/src/convergence.rs` | Shared convergence derivation + canonical convergence taxonomy projection | Complete | Session 20 extracted duplicated GROMACS/OpenMM derivation into `derive_energy_convergence_summary` (adapter-inline call sites preserved), added canonical taxonomy projection (`CanonicalConvergence`, `classify_convergence`, `classify_all_convergence`), and codified divergence-priority behavior over metric-level mapping. |
| `lel-ir-prototype/src/vasp_adapter.rs` | VASP INCAR/OSZICAR/OUTCAR -> LEL adapter | Complete | DFT compatibility on existing IR types, section-marker composition across 3 files, and first adapter-level exercise of `ConvergencePoint` + `StateSnapshot`. |
| `lel-ir-prototype/src/bench.rs` | CausalOverlay construction benchmark | Complete | Benchmarks real `CausalOverlay::from_log` at 4 scales (10^3-10^6). Latest result: 251.82ms overlay construction at 10^6 events (22.62ms at 10^5), confirming practical O(n) behavior. |


## Next Steps

> **Status as of 2026-02-23:** All originally scoped next steps are complete. New next steps should be appended below.

1. ~~**Survey DSL trace formats**~~ — **COMPLETE.** OpenMM, GROMACS, and VASP surveys done. See investigation logs above and `dsl-evaluation/` analysis documents.

2. ~~**Survey existing IR designs in RCA and formal verification**~~ — **COMPLETE.** RCA/formal verification survey and provenance/workflow IR survey done. See investigation logs above and `dsl-evaluation/rca-formal-verification-ir-survey.md`, `dsl-evaluation/provenance-workflow-ir-survey.md`.

3. ~~**Map LFI three-stage audit backwards to minimum IR requirements**~~ — **COMPLETE.** 29 requirements (R1-R29) derived. See LFI audit investigation log above.

4. ~~**Characterize the 21% baseline and DSL improvement**~~ — **COMPLETE** (pending verification of source). See baseline characterization investigation log above. Key action item: verify the 21% source with web access.

5. **Draft candidate IR schemas and prototype** — **Steps 5a, 5b, 5c COMPLETE.** Three candidates (LEL, DGR, Hybrid LEL+DGR) evaluated against R1-R29, 9 anti-patterns, streaming constraints, and 7-criterion weighted framework. Recommendation: Hybrid (94/100). Step 5 outputs remain valid and are now extended by Step 6 implementation details below. See `dsl-evaluation/candidate-ir-schemas.md`, `prototypes/codex-prompt-5b-lel-prototype.md`, and investigation logs above. (Beads: athena-axc, athena-9uv)

6. **Hybrid LEL+DGR Phase 2 prototype (CausalOverlay + R14 query)** — **COMPLETE.** `by_id` index added; `src/overlay.rs` implemented with O(n) construction and BFS traversal; R14 confounder detection query implemented and tested. Baseline Step 6 crate state was 29/29 tests passing with strict clippy clean; subsequent Step 7 query expansion now validates 44/44 tests with strict clippy clean. Benchmark uses real overlay path and reports 251.82ms at 10^6 events. (Tracking updates: #37 closed, #38 narrowed/validated)

7. **R17 quantitative comparison formalization and bridge contract (Step 12)** — **COMPLETE (NARROWED).** Literature survey + prototype mapping + candidate scoring completed. Recommendation: Candidate B (Multi-Metric Divergence Profile) with optional posterior hooks. Trace-semantics now defines a contract (`ComparisonProfileV1` assumptions) that adversarial-reward can consume once that track starts. Remaining work moved to What We Don't Know #40 (WDK#41 resolved cross-track; WDK#42/#43/#44 resolved in prototype scope by Sessions 19-20).

8. **Convergence trajectory representation (Step 13)** — **COMPLETE (NARROWED).** Empirical adapter inventory + A/B/C steel-man stress tests + external survey + consumer trace completed. Recommendation: Option D hybrid (raw canonical trajectory + Stage 1->2 summary). Sessions 19-20 resolve WDK#42/#43/#44 in prototype scope; WDK#40 linkage remains for uncertainty-schema implementation details.

9. **UncertaintySummary schema for divergence metrics (Step 14)** — **COMPLETE (NARROWED).** External survey + six-consumer trace + cross-adapter feasibility + A/B/C stress-test completed. Recommendation: Candidate C (mandatory point uncertainty + optional tagged distribution payload + explicit missingness reason) with shared numeric core across `MetricComponent` and `ConvergenceSummary` uncertainty fields. Remaining dependency focus is WDK#40 implementation detail, with WDK#41/#42/#43/#44 now resolved in their respective scopes.

**Synthesis steps needed before Step 5:**

- ~~**Step 1d: Cross-framework trace synthesis.**~~ — **COMPLETE.** Trace capability matrix, failure mode taxonomy (49 modes), trace completeness assessment, and Decision Gate 1 resolved (VASP accepted with external classification table). See `dsl-evaluation/cross-framework-synthesis.md` and investigation log above. (Bead: athena-ywn, CLOSED)

- ~~**Step 2c: Comparative IR synthesis.**~~ — **COMPLETE.** Seven-category pattern catalog, nine anti-patterns, MLIR/PROV-DM tension resolved (complementary: routing + provenance), Decision Gate 2 resolved (hybrid adaptation, MEDIUM risk). See `dsl-evaluation/ir-pattern-catalog.md` and investigation log above. (Bead: athena-tyt, CLOSED)

- ~~**Step 3b: Requirements refinement with coverage matrix.**~~ — **COMPLETE.** R1-R29 cross-referenced against trace capability matrix. Coverage matrix with six classification codes (DA/DI/ER/FU/NT/DE), gap analysis, per-stage feasibility assessment, and Decision Gate 4 (PASS) produced. See `dsl-evaluation/requirements-coverage-matrix.md` and investigation log above. (Bead: athena-rf6, CLOSED)
