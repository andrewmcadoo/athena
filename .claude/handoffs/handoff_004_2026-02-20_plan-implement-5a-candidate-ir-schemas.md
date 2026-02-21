# Session Handoff: Trace Semantics IR — Step 5a Candidate IR Schemas

> Generated: 2026-02-20 | Handoff #4 | Previous: handoff_003_2026-02-20_step-3b-requirements-refinement.md

---

## Continuation Directive

Plan and implement Step 5a: Draft candidate IR schemas for ATHENA's Trace Semantics Engine. Bead `athena-axc` is READY (unblocked by completion of athena-rf6). This is the culmination of the trace semantics research — synthesize all accumulated evidence (R1-R29 requirements, trace capability matrix, IR pattern catalog, coverage matrix) into 2-3 concrete IR schema designs with explicit tradeoffs. Success criteria: candidate schemas evaluated against R1-R29, with a recommendation for which to prototype in Step 5b.

---

## Task Definition

ATHENA P1: Trace Semantics IR research. Designing an intermediate representation that translates raw DSL trace logs from OpenMM, GROMACS, and VASP into semantic failure representations for three-way causal fault classification (implementation / methodology / theory). Step 5a produces the candidate IR schemas; Step 5b (athena-9uv, blocked on this) prototypes the most promising one. All prior investigation steps (1-4, synthesis steps 1d/2c/3b) are complete.

## Key Decisions & Rationale

1. **Decision Gate 1: VASP accepted with external classification table** — VASP's INCAR namespace is DIRTY (~200-300 tags, no API-declared boundary), but 70-80% of calculations classifiable with full confidence. Dropping VASP loses the DFT domain.

2. **Decision Gate 2: Hybrid IR adaptation, MEDIUM risk** — ~65-70% transfers from existing systems (MLIR dialects + PROV-DM concepts + Boogie contracts). ~30-35% novel: three-way typing, fault ontology, prediction-observation comparison, methodology detection.

3. **Decision Gate 4: PASS** — No LFI stage blocked by FU requirements. FU cells only partial R6 (sub-component internals) at ~5-10% per framework. Four conditions: OpenMM custom reporter, VASP classification table, VASP degraded confidence for ambiguous params, R17 comparison method formalization.

4. **DGR recommended as primary candidate** — Coverage matrix shows IR is fundamentally a three-input composite (trace + external context + domain rules). 31% of requirements are NT (external). DGR's graph structure naturally handles multi-source entities with qualified relationships. LEL strongest for Stage 1 (high DA density). TAL better as query interface layer than standalone IR.

5. **MLIR dialects + PROV-DM are complementary** — Dialects = routing (WHERE an element belongs → which LFI stage). PROV-DM = causal structure (HOW elements relate within stages). Unified architecture uses dialect structure as primary organization with PROV-DM causal graphs within each layer.

## Current State

### Completed (All Prerequisites for Step 5a)
- Steps 1a-1c: DSL trace surveys (OpenMM, GROMACS, VASP) — 3 analysis docs
- Steps 2a-2b: IR design surveys (RCA/FV, Provenance/Workflow) — 2 survey docs
- Step 3a: LFI audit → 29 requirements (R1-R29)
- Step 4: 21% baseline characterization
- **Step 1d** (athena-ywn, CLOSED): Cross-framework trace synthesis — trace capability matrix, 49 failure modes, Decision Gate 1
- **Step 2c** (athena-tyt, CLOSED): Comparative IR synthesis — 7 pattern categories, 9 anti-patterns, Decision Gate 2, 3 candidate design previews (LEL/DGR/TAL)
- **Step 3b** (athena-rf6, CLOSED): Requirements coverage matrix — R1-R29 × {OpenMM, GROMACS, VASP} with DA/DI/ER/FU/NT/DE codes, gap analysis, Decision Gate 4 PASS
- FINDINGS.md: 10 investigation log entries, 27 "What We Know", 23 "What We Suspect", 30 "What We Don't Know"

### In Progress
- Nothing — clean state for next session.

### Blocked / Open Questions
1. **R17 comparison method** — Quantitative prediction-observation comparison (effect size, divergence metrics, tolerance thresholds) is novel research. Blocks Stage 3 capability. [What We Don't Know #28]
2. **LEL→DGR incremental path** — Can you start with LEL for Stage 1 then evolve to DGR? Or must DGR be designed up-front? [What We Don't Know #29]
3. **Causal reasoning substrate** — Log search vs. graph traversal vs. assertion chains. Needs benchmarking against LFI query patterns from R1-R29. [What We Don't Know #23]
4. **Boundary parameter representation** — How to handle params spanning multiple dialects (GROMACS dt, VASP PREC). [What We Don't Know #24]

## Key Code Context

No code — research documents only. Critical structural context for Step 5a:

**Three candidate previews** are in `ir-pattern-catalog.md` §6:
- **LEL (Layered Event Log)**: Flat typed event log with dialect tags. Patterns: typed event chains + dialects + SSA + counter-examples. Stage 1 strong, Stages 2-3 weak.
- **DGR (Dual-Graph IR)**: Prospective (spec) + retrospective (execution) graphs bridged by lowering relations. Patterns: dialects + Entity-Activity-Agent + contracts + conformance. Stages 2-3 strong, Stage 1 moderate.
- **TAL (Typed Assertion Log)**: Assertion sequence ordered by audit stage. Patterns: contracts + dialects + counter-examples + typed chains. Highest novelty risk. Stage 1 strong, Stage 3 strong for comparison, weak for causal reasoning.

**Coverage matrix key numbers** (from `requirements-coverage-matrix.md`):
- OpenMM: 4 DA, 10 DI, 0 ER (highest instrumentation burden, cleanest boundary)
- GROMACS: 9 DA, 4 DI, 1 ER (best default coverage)
- VASP: 9 DA, 4 DI, 1 ER (highest domain-knowledge burden)
- All frameworks: 9 NT, 6 DE, 1 partial FU each

**IR data flow architecture** (from coverage matrix §8):
```
External Inputs (NT: 31%)     Trace Data (DA/DI)        Domain Knowledge (ER)
├── Experiment spec            ├── Execution events       ├── GROMACS param table
├── Hypothesis                 ├── Numerical metrics      └── VASP INCAR table
├── DAG references             ├── Resource state
├── Cycle ID                   ├── Parameter echo
└── Observation mode           ├── Energy/observables
                               └── Temporal markers
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master investigation log + accumulated findings (10 entries) | Updated this session |
| `research/trace-semantics/dsl-evaluation/requirements-coverage-matrix.md` | R1-R29 × framework coverage, gap analysis, DG4 | **Created this session** |
| `research/trace-semantics/dsl-evaluation/ir-pattern-catalog.md` | 7 patterns, 9 anti-patterns, 3 candidate previews, DG2 (725 lines) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/cross-framework-synthesis.md` | Trace capability matrix, 49 failure modes, DG1 (563 lines) | Input (unchanged) |
| `research/trace-semantics/dsl-evaluation/openmm-trace-analysis.md` | OpenMM trace survey | Input |
| `research/trace-semantics/dsl-evaluation/gromacs-trace-analysis.md` | GROMACS trace survey | Input |
| `research/trace-semantics/dsl-evaluation/vasp-trace-analysis.md` | VASP trace survey | Input |
| `research/trace-semantics/dsl-evaluation/rca-formal-verification-ir-survey.md` | RCA/FV IR survey | Input |
| `research/trace-semantics/dsl-evaluation/provenance-workflow-ir-survey.md` | Provenance/workflow IR survey | Input |
| `evaluation/hidden-confounder/README.md` | Litmus test spec (R27-R29 context) | Reference |

## Loop State

N/A — single-session research execution.

## Next Steps

1. `bd update athena-axc --status=in_progress` — Claim Step 5a.
2. **Read FINDINGS.md** (required by CLAUDE.md protocol) — focus on accumulated findings and the Step 2c/3b log entries for the structural foundation and coverage patterns.
3. **Design 2-3 candidate IR schemas** building on the previews in ir-pattern-catalog.md §6:
   - Define concrete data structures for each candidate (Rust-informed: enums, structs, graph types)
   - Map R1-R29 to specific schema elements per candidate
   - Evaluate against anti-pattern registry (ir-pattern-catalog.md §3)
   - Assess streaming compatibility per ADR 001
   - Address the four open questions (R17 comparison, LEL→DGR path, causal substrate, boundary params)
4. **Produce recommendation** with explicit tradeoffs: which candidate to prototype in Step 5b (athena-9uv).
5. **Deliverables:** Candidate schemas document in `dsl-evaluation/` or `prototypes/`, FINDINGS.md log entry, accumulated findings updates.
6. `bd close athena-axc` → unblocks `athena-9uv` (Step 5b: prototype).

## Session Artifacts

- Coverage matrix: `research/trace-semantics/dsl-evaluation/requirements-coverage-matrix.md`
- Previous handoff: `.claude/handoffs/handoff_003_2026-02-20_step-3b-requirements-refinement.md`

## Documentation Updated

No documentation updates — all project docs were current.
