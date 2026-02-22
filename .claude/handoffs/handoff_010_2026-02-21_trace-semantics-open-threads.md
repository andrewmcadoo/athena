# Session Handoff: Trace Semantics Open Threads

> Generated: 2026-02-21 | Handoff #10 | Previous: handoff_009_2026-02-21_vasp-adapter-litmus-test.md

---

## Continuation Directive

Continue researching open threads in the trace-semantics investigation. The VASP adapter and hidden confounder litmus test are done (Steps 10-11). The prototype crate is at 92/92 tests. Pick up remaining open threads from FINDINGS.md — particularly those that affect cross-track dependencies (adversarial-reward, exploration-convergence).

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** The IR design research question asks: what IR can translate DSL trace logs into semantic failure representations for three-way causal fault classification? Steps 1-11 are complete. The Hybrid LEL+DGR prototype is validated with 3 adapters (OpenMM mock, GROMACS, VASP), CausalOverlay + R14/R17/R18 queries, and a hidden confounder litmus test. Remaining work: address open threads that affect downstream research tracks or production readiness.

**Success criteria:** Resolve or narrow open threads with evidence. Each session produces a FINDINGS.md log entry.

**Constraints:** Per CLAUDE.md — append-only FINDINGS.md, prototype code only, read FINDINGS.md before starting.

## Key Decisions & Rationale

1. **Hybrid LEL+DGR architecture (94/100)**
   - LEL streaming for Stage 1, CausalOverlay graph traversal for Stages 2-3.
   - Validated through Steps 5-11 with 92 tests across 3 framework adapters.

2. **One IR handles both DFT and MD (WDK#12 resolved)**
   - VASP adapter maps INCAR/OSZICAR/OUTCAR into existing LEL types with zero schema changes.
   - ConvergencePoint and StateSnapshot exercised for the first time.

3. **Hidden confounder detection validated end-to-end**
   - Planted PREC as common ancestor of SIGMA/IBRION, detect_confounders correctly identifies it.
   - Controlled variable exclusion works per R14 semantics.

4. **Normalization convention: VASP uses uppercase keys**
   - Both ParameterRecord.name and dag_node_ref use uppercase-normalized keys (deviation from GROMACS pattern).
   - Required for litmus test consistency between by_variable index and dag_node_ref.

## Current State

### Completed (Steps 1-11)
- **3 adapters:** MockOpenMmAdapter, GromacsAdapter, VaspAdapter — all on same EventKind types
- **CausalOverlay:** R14 confounders, R17 prediction comparison, R18 causal implication
- **Hidden confounder litmus:** Positive detection + controlled exclusion validated
- **Benchmark:** CausalOverlay construction 251ms at 10^6 events
- **Crate:** 92/92 tests, strict clippy clean
- **Beads closed:** athena-7ob (VASP adapter), athena-9cp (litmus test)

### Open Threads (from FINDINGS.md)

**High priority — affect downstream research tracks:**
- **WDK#28:** How to formalize quantitative prediction-observation comparison (R17). Single blocker for Stage 3 feasibility. Connects to adversarial-reward research.
- **WDK#13:** How convergence trajectories should be represented (raw series vs. classified patterns vs. derived features). Now answerable with VASP ConvergencePoint data.
- **WDK#9:** How to adapt statistical refutation into machine-checkable chains (DRAT propositional → probabilistic).

**Medium priority — affect production readiness:**
- **WDK#35:** ContractTerm needs `value: Option<Value>` for VASP Stage 3.
- **WDK#36:** Value enum needs KnownMatrix for VASP spectral data.
- **WDK#39:** ComparisonResult.prediction_id String vs. SpecElementId harmonization.
- **WDK#25:** Practical impact of VASP closed-source ceiling.
- **WDK#26:** INCAR classification table completeness (~200-300 params).

**Lower priority — narrowed or deferred:**
- **WDK#38:** Arena allocation for overlay (validated Vec-first at 10^6 scale).
- **WDK#3/10/11:** Temporal gaps, multi-dialect events, provenance granularity.
- **WDK#1-8:** DSL-specific unknowns (OpenMM instrumentation, GROMACS crash dumps, VASP silent failures).

### Blocked
- Full 50-cycle hidden confounder evaluation depends on adversarial-reward formalization.

## Key Code Context

**`src/overlay.rs:158-228`** — `detect_confounders` is the validated R14 query. Its API signature:
```rust
pub fn detect_confounders(&self, log: &LayeredEventLog, observable_var: &str, intervention_var: &str) -> Vec<ConfounderCandidate>
```
Ancestor intersection → filter controlled variables → filter intervention variable → group by dag_node.

**`src/tests/mod.rs`** — 25 VASP tests + 2 litmus tests added this session. The litmus test pattern (parse → mutate causal_refs → rebuild overlay → detect_confounders → assert) is reusable for future confounder experiments.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `prototypes/lel-ir-prototype/src/vasp_adapter.rs` | VASP INCAR/OSZICAR/OUTCAR adapter | Created this session |
| `prototypes/lel-ir-prototype/src/tests/mod.rs` | 92 tests total | Modified this session (+25 VASP, +3 litmus/error) |
| `prototypes/lel-ir-prototype/src/lib.rs` | Module registration | Modified (added vasp_adapter) |
| `prototypes/lel-ir-prototype/src/overlay.rs` | CausalOverlay + R14/R17/R18 | Reference (unchanged) |
| `prototypes/lel-ir-prototype/src/common.rs` | Shared types | Reference (unchanged) |
| `prototypes/lel-ir-prototype/src/event_kinds.rs` | 12 EventKind variants | Reference (unchanged) |
| `research/trace-semantics/FINDINGS.md` | Master investigation log | Updated (Steps 10-11, synthesis) |
| `evaluation/hidden-confounder/README.md` | Litmus test spec | Updated (status from NOT STARTED) |

## Loop State

**Iteration 3 of Claude→Codex→Claude workflow (VASP adapter, now complete):**
- **Prompt 6** (`.claude/prompts/prompt_006_...vasp-adapter-litmus-test.md`): RISEN prompt for VASP adapter + litmus test.
- **Codex executed:** All 7 steps. Hit 3 compile errors (concat! with const identifiers, Vec type inference, parallelization mutation pattern).
- **Fixes applied:** Raw literal for VASP_COMBINED_SAMPLE, explicit Vec<TraceEvent> type, mutable reference for ResourceStatus field.
- **Final result:** 92/92 tests, clippy clean. Committed as `57431d5`.

For future trace-semantics work, no Codex loop is likely needed — remaining threads are research investigations, not implementation.

## Next Steps

1. **Read `research/trace-semantics/FINDINGS.md`** — specifically the "What We Don't Know" section and open threads from Steps 10-11.
2. **Pick one high-priority thread** — WDK#28 (R17 comparison formalization) is the most impactful because it blocks Stage 3 and connects to adversarial-reward research.
3. **Alternatively, explore WDK#13** (convergence trajectory representation) since the VASP adapter now provides real ConvergencePoint data to analyze.
4. **For any investigation:** Follow CLAUDE.md session scoping — pick one thread, investigate, write a FINDINGS.md log entry before ending.
5. **If switching tracks:** `research/adversarial-reward/FINDINGS.md` is the next-priority research dependency. The trace-semantics litmus test proved the IR detects confounders; adversarial-reward determines how to choose experiments that expose them.

## Session Artifacts

- Prompt 6: `.claude/prompts/prompt_006_2026-02-21_vasp-adapter-litmus-test.md` (RISEN, VASP adapter + litmus)
- Previous handoff: `.claude/handoffs/handoff_009_2026-02-21_vasp-adapter-litmus-test.md`
- Commits: `57431d5` (VASP adapter + litmus), `425ca6f` (prompt file)
- Beads closed: `athena-7ob`, `athena-9cp`

## Documentation Updated

No documentation updates — all project docs were current.
