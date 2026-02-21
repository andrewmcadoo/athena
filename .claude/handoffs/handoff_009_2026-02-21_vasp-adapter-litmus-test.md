# Session Handoff: VASP Adapter + Hidden Confounder Litmus Test

> Generated: 2026-02-21 | Handoff #9 | Previous: handoff_008_2026-02-21_gromacs-adapter-cross-framework.md

---

## Continuation Directive

Build a VASP adapter for the LEL IR prototype first (bead `athena-7ob`), then implement the hidden confounder litmus test end-to-end (bead `athena-9cp`). The VASP adapter stress-tests whether the IR handles DFT codes without type changes. The litmus test is the first empirical validation that the IR actually detects planted faults. Together these close out the trace-semantics research track.

## Task Definition

**Project:** ATHENA -- Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** (1) Build a VASP adapter parsing INCAR/OUTCAR/OSZICAR into `LayeredEventLog`, exercising `ConvergencePoint` and `StateSnapshot` variants that OpenMM/GROMACS never touched. (2) Implement the hidden confounder litmus test from `evaluation/hidden-confounder/README.md`: plant a confounder, run adapter->LEL->overlay->R14, verify detection.

**Success criteria:** (a) VASP adapter works with existing EventKind types -- no schema changes, (b) litmus test detects a planted confounder correctly, (c) both produce FINDINGS.md entries, (d) crate remains clippy clean with all tests passing.

**Constraints:** Per CLAUDE.md -- research artifacts only, append-only FINDINGS.md, Rust prototypes. Per ADR 001 -- Rust for perf-critical.

## Key Decisions & Rationale

1. **Hybrid LEL+DGR IR architecture (94/100)**
   - **Rationale:** LEL streaming for Stage 1, DGR graph traversal for Stages 2-3. Passes all 9 anti-patterns.
   - **Alternatives rejected:** LEL standalone (82), DGR standalone (82)

2. **GROMACS adapter validated cross-framework generalization**
   - **Rationale:** 20+ GROMACS parameters classified into Theory/Methodology/Implementation layers using existing types. No EventKind changes needed. CausalOverlay + R14 queries work on GROMACS-derived logs.
   - This establishes the pattern the VASP adapter should follow.

3. **DslAdapter trait pattern**
   - `parse_trace(&self, raw: &str) -> Result<LayeredEventLog, AdapterError>`
   - Section markers `--- MDP ---` / `--- LOG ---` delimit combined input (GROMACS). VASP will need its own markers (e.g., `--- INCAR ---` / `--- OUTCAR ---`).
   - Expose individual file parsers as public helpers for unit testing.

4. **Layer classification is deterministic per-parameter**
   - Each DSL parameter maps to one Layer + BoundaryClassification via a classifier function.
   - Unknown parameters default to Implementation + ContextDependent.
   - VASP parameters: ENCUT/PREC -> Theory, IBRION/NSW/EDIFF -> Methodology, NCORE/KPAR -> Implementation.

5. **Beads dependency: VASP adapter blocks litmus test**
   - `athena-9cp` (litmus) depends on `athena-7ob` (VASP). Do VASP first.

## Current State

### Completed
- **Steps 1-7, 9 all complete.** Full chain: DSL surveys -> candidate schemas -> LEL prototype -> overlay + R14/R17/R18 -> GROMACS adapter.
- **LEL prototype crate** at `research/trace-semantics/prototypes/lel-ir-prototype/`: 67/67 tests, clippy clean.
- **Two adapters:** MockOpenMmAdapter (mock) + GromacsAdapter (real parsing).
- **CausalOverlay** with R14 (confounders), R17 (prediction comparison), R18 (causal implication).
- **GROMACS adapter review fixes applied:** tightened Step matcher, missing Total Energy warning, expanded multi-word header list, CrashDivergent test.

### In Progress
- Nothing -- clean state for VASP adapter work.

### Blocked / Open Questions
- **FINDINGS.md "What We Don't Know" #12:** Can one IR handle both DFT (VASP) and MD (OpenMM/GROMACS)? This is what the VASP adapter answers.
- **WDK #35:** `ContractTerm` may need `value: Option<Value>` for VASP Stage 3.
- **WDK #36:** `Value` enum may need `KnownMatrix` for VASP spectral data.
- VASP has multi-file input (INCAR/POSCAR/KPOINTS) unlike GROMACS single .mdp. Adapter design must handle this.
- VASP convergence (SCF iterations in OSZICAR, ionic steps) exercises `ConvergencePoint` variant -- untested so far.
- The `evaluation/hidden-confounder/README.md` spec must be read before implementing the litmus test.

## Key Code Context

**`src/adapter.rs:36-38`** -- DslAdapter trait (the contract):
```rust
pub trait DslAdapter {
    fn parse_trace(&self, raw: &str) -> Result<LayeredEventLog, AdapterError>;
}
```

**`src/gromacs_adapter.rs`** -- Follow this pattern for VASP. Key structural elements:
- `classify_mdp_parameter()` -- deterministic layer classifier per parameter
- `parse_mdp()` / `parse_log()` -- individual file parsers exposed as pub helpers
- `parse_trace()` -- wires section markers, builds ExperimentSpec from parsed params, establishes causal refs post-build
- Causal ref wiring: MDP event IDs -> energy events, last energy -> execution status

**`src/event_kinds.rs`** -- Variants VASP should exercise that GROMACS didn't:
```rust
ConvergencePoint { iteration: u64, metric_name: String, metric_value: Value, converged: Option<bool> }
StateSnapshot { snapshot_type: SnapshotType, data_ref: String }
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `prototypes/lel-ir-prototype/src/gromacs_adapter.rs` | GROMACS adapter -- pattern to follow | Complete (67 tests) |
| `prototypes/lel-ir-prototype/src/adapter.rs` | DslAdapter trait + mock OpenMM | Reference |
| `prototypes/lel-ir-prototype/src/event_kinds.rs` | 12 EventKind variants | Reference |
| `prototypes/lel-ir-prototype/src/common.rs` | Shared types (Layer, Value, etc.) | Reference |
| `prototypes/lel-ir-prototype/src/lel.rs` | Core LEL types, builders, indexes | Reference |
| `prototypes/lel-ir-prototype/src/overlay.rs` | CausalOverlay + R14/R17/R18 queries | Reference |
| `prototypes/lel-ir-prototype/src/tests/mod.rs` | 67 tests (22 GROMACS + 44 prior + 1 crash) | Append new tests here |
| `research/trace-semantics/FINDINGS.md` | Master investigation log (54 WK items) | Append new entries |
| `evaluation/hidden-confounder/README.md` | Litmus test spec anchor | Read before litmus work |

## Loop State

**Iteration 2 of Claude->Codex->Claude review loop for GROMACS adapter (now complete):**
- **Prompt 4** (`.claude/prompts/prompt_004_...gromacs-adapter-lel-ir.md`): Full RISEN prompt for Step 9 GROMACS adapter. Codex implemented all 7 steps, 66/66 tests.
- **Code review findings:** 3 warnings (Step matcher, missing Total Energy, multi-word headers) + 1 coverage gap (CrashDivergent test).
- **Prompt 5** (`.claude/prompts/prompt_005_...gromacs-adapter-review-fixes.md`): RTF followup prompt for 4 fixes. Codex applied all, 67/67 tests.

For the VASP adapter, consider using the same prompt->Codex->review pattern. The GROMACS RISEN prompt (prompt 4) is a good structural template to adapt.

## Next Steps

1. **`bd update athena-7ob --status=in_progress`** -- claim the VASP adapter bead.
2. **Read `evaluation/hidden-confounder/README.md`** -- understand the litmus test spec before designing the VASP adapter (the adapter should produce data the litmus test can consume).
3. **Read FINDINGS.md VASP survey entries** -- grep for "VASP" in What We Know/Suspect/Don't Know sections for domain context.
4. **Plan the VASP adapter** -- likely needs: `classify_incar_parameter()`, `parse_incar()`, `parse_outcar()`, `parse_oszicar()`, section markers for combined input. Key difference from GROMACS: SCF convergence data -> `ConvergencePoint`, ionic relaxation steps, POSCAR/CONTCAR structure data -> `StateSnapshot`.
5. **Write a RISEN prompt for the VASP adapter** -- follow prompt 4's structure. Use `/prompt` to generate it.
6. **Implement, test, review** -- target ~20 new tests, clippy clean.
7. **Close `athena-7ob`**, then move to `athena-9cp` (litmus test).
8. **Implement hidden confounder litmus test** per the README spec. Plant a confounder, run full pipeline, assert detection.
9. **Close `athena-9cp`**, update FINDINGS.md with both entries.

## Session Artifacts

- Prompt 4: `.claude/prompts/prompt_004_2026-02-21_gromacs-adapter-lel-ir.md` (RISEN, GROMACS adapter)
- Prompt 5: `.claude/prompts/prompt_005_2026-02-21_gromacs-adapter-review-fixes.md` (RTF, review fixes)
- Previous handoff: `.claude/handoffs/handoff_008_2026-02-21_gromacs-adapter-cross-framework.md`
- Beads: `athena-7ob` (VASP adapter, open), `athena-9cp` (litmus test, blocked by 7ob)
- Uncommitted work: GROMACS adapter + review fixes on current branch (not yet committed to main)

## Documentation Updated

No documentation updates -- all project docs were current.
