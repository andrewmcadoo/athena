# Session Handoff: Trace Semantics IR — Step 3b Requirements Refinement

> Generated: 2026-02-20 | Handoff #3 | Previous: handoff_002_2026-02-20_synthesis-phase-trace-semantics.md

---

## Continuation Directive

Continue the P1 Trace Semantics IR investigation: plan and implement Step 3b (requirements refinement with coverage matrix). Bead athena-rf6 is READY (unblocked by completion of athena-ywn). Cross-reference R1-R29 requirements against the trace capability matrix. Produce a coverage matrix showing requirement satisfaction per framework.

---

## Task Definition

ATHENA P1: Trace Semantics IR research investigation. Designing an intermediate representation that translates raw DSL trace logs from OpenMM, GROMACS, and VASP into semantic failure representations for three-way causal fault classification (implementation / methodology / theory). This session completed the synthesis phase (Steps 1d and 2c); the next session executes the requirements refinement phase (Step 3b).

## Key Decisions & Rationale

1. **Decision Gate 1 resolved: VASP accepted with external classification table** — VASP's theory-implementation boundary is DIRTY (flat INCAR namespace, ~200-300 tags not API-classified), but the boundary exists conceptually. 70-80% of standard calculations classifiable with full confidence; 20-30% degraded from ambiguous parameters (PREC, ALGO, LREAL). Dropping VASP would lose the DFT domain.
   - Alternative rejected: Narrow DSL set to MD-only (OpenMM, GROMACS) — loses generalizability test.

2. **Decision Gate 2 resolved: Hybrid IR adaptation, MEDIUM risk** — ~65-70% transfers from existing systems (MLIR dialects for layer routing + PROV-DM concepts for causal provenance + Boogie contracts for spec-impl separation). ~30-35% requires novel design: three-way layer typing vocabulary, fault classification ontology, quantitative prediction-observation comparison, methodology detection rules.
   - Alternative rejected: Fully novel IR design — higher risk, no validation from prior art.

3. **MLIR dialects and PROV-DM are complementary, not contradictory** — Dialects answer "WHERE does an element belong?" (routing to LFI audit stage). PROV-DM answers "HOW are elements causally related?" (causal chain traversal within stages). Technology: adopt PROV-DM concepts, reject RDF/SPARQL, Rust-native per ADR 001.

4. **Adapter architecture: 7 mandatory + 7 optional methods** — Common IR core (timestamped events, energy series, parameter records, error events, state snapshots, convergence trajectories, data absence records) with DSL-specific adapter extensions. Classification table per framework maps parameters to theory/implementation/boundary.

5. **FINDINGS.md updated with 13 new accumulated findings** — "What We Know" 28-40 (cross-framework synthesis + IR synthesis), "What We Suspect" 20-23 (DGR candidate strength, incremental implementation, classification table automation, adapter method evolution), "What We Don't Know" 23-27 (causal substrate choice, boundary parameter representation, closed-source ceiling, INCAR table validation, Stage 3 buffering).

## Current State

### Completed
- All 7 research branches merged to master (sessions 1-2)
- Steps 1a-1c: DSL trace surveys (OpenMM, GROMACS, VASP) — 3 analysis docs
- Steps 2a-2b: IR design surveys (RCA/FV, Provenance/Workflow) — 2 survey docs
- Step 3a: LFI audit → IR requirements mapping — 29 requirements (R1-R29) derived
- Step 4: 21% baseline characterization — uncited, from cloud/AIOps, non-transferable
- **Step 1d: Cross-framework trace synthesis** (athena-ywn, CLOSED) — 563-line synthesis doc with trace capability matrix, 49 failure modes, Decision Gate 1
- **Step 2c: Comparative IR synthesis** (athena-tyt, CLOSED) — 725-line pattern catalog with 7 pattern categories, 9 anti-patterns, Decision Gate 2
- FINDINGS.md: 9 investigation log entries, 40 "What We Know", 23 "What We Suspect", 27 "What We Don't Know"
- Beads: 10 closed, 3 open (1 ready, 2 blocked)

### In Progress
- Nothing — clean state for next session

### Blocked / Open Questions
1. **VISION.md 21% claim** — Uncited, from cloud/AIOps domain. Decision deferred.
2. **R28 gap** — Interventional vs. observational distinction not in ARCHITECTURE.md §5.3. May need amendment.
3. **Variable naming ontology** — Implicit coordination requirement between Trace Semantics Engine and Causal Graph Manager (R9, R11, R14).
4. **INCAR classification table** — Needs domain expert validation before VASP adapter finalized.

## Key Code Context

No code written — research documents only. Key structural context for Step 3b:

The 29 requirements (R1-R29) are in FINDINGS.md investigation log entry "LFI Audit → IR Requirements Mapping":
- **Stage 1 (Implementation):** R1-R7 (execution status, exceptions, input spec, numerical status, resource status, data validation, exception location)
- **Stage 2 (Methodology):** R8-R14 (observables, DAG linkage, interventions, intervention-DAG linkage, sampling metadata, controlled variables, confounder query support)
- **Stage 3 (Theory):** R15-R18 (prediction record, observation record, comparison result, causal implication mapping)
- **Cross-cutting:** R19-R29 (layer tag, provenance chain, temporal ordering, experiment spec linkage, observable queryability, confounder auditability, observability gaps, classification confidence, data absence, interventional/observational distinction, cross-experiment queryability)

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Investigation log + accumulated findings (9 entries) | Updated this session |
| `research/trace-semantics/dsl-evaluation/cross-framework-synthesis.md` | Trace capability matrix, failure taxonomy, Decision Gate 1 (563 lines) | **Created this session** |
| `research/trace-semantics/dsl-evaluation/ir-pattern-catalog.md` | IR pattern catalog, anti-patterns, Decision Gate 2 (725 lines) | **Created this session** |
| `research/trace-semantics/dsl-evaluation/openmm-trace-analysis.md` | OpenMM trace survey (733 lines) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/gromacs-trace-analysis.md` | GROMACS trace survey (841 lines) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/vasp-trace-analysis.md` | VASP trace survey (565 lines) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/rca-formal-verification-ir-survey.md` | RCA/FV IR design survey (524 lines) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/provenance-workflow-ir-survey.md` | Provenance/workflow IR survey (662 lines) | Input (unchanged) |
| `research/trace-semantics/docs/research-plan.md` | 10-session research plan | Reference (unchanged) |

## Loop State

N/A — single-session research execution.

## Next Steps

1. `bd update athena-rf6 --status=in_progress` — Step 3b: Requirements refinement with coverage matrix (P2, READY). 
2. **Primary deliverable:** Cross-reference R1-R29 against the trace capability matrix from `cross-framework-synthesis.md`. For each requirement × each framework: directly available / derivable with custom instrumentation / requires external domain rules / fundamentally unavailable.
3. **Gap analysis:** For each gap: fillable by instrumentation? By pre-execution analysis? By inference? Or fundamentally unobservable?
4. **Feasibility assessment:** Per LFI stage, can it produce deterministic answers? Under what conditions?
5. **DECISION GATE 4:** If ANY LFI stage is infeasible given trace data, escalate. Options: (a) narrow DSL set, (b) require instrumentation wrappers, (c) weaken deterministic audit to probabilistic.
6. **Deliverable format:** FINDINGS.md log entry + coverage matrix document in `dsl-evaluation/`.
7. After rf6 completes, athena-axc (Step 5a: draft candidate IR schemas) unblocks. Dependency chain: rf6 → axc → 9uv.

## Session Artifacts

- Cross-framework synthesis: `research/trace-semantics/dsl-evaluation/cross-framework-synthesis.md`
- IR pattern catalog: `research/trace-semantics/dsl-evaluation/ir-pattern-catalog.md`
- Plan file: `/home/aj/.claude/plans/streamed-strolling-pike.md`

## Documentation Updated

No documentation updates — all project docs were current. CLAUDE.md accurately reflects research phase.
