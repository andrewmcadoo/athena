# Session Handoff: Parallel Investigation of Trace-Semantics Open Items

> Generated: 2026-02-21 | Handoff #11 | Previous: handoff_010_2026-02-21_trace-semantics-open-threads.md

---

## Continuation Directive

Use sub-agent teams to investigate the smaller open items in trace-semantics in parallel. The major steps (1-14) are all complete. Five open "What We Don't Know" items remain that are independent of each other and suitable for parallel investigation by sub-agents. Each agent should read FINDINGS.md, investigate one WDK item, and produce a FINDINGS.md log entry draft. The lead agent coordinates, reviews drafts, and appends approved entries.

## Task Definition

**Project:** ATHENA — Falsification-driven AI co-scientist. Priority 1 research: Trace Semantics Engine IR design.

**Goal:** Resolve or narrow the remaining smaller open threads in `research/trace-semantics/FINDINGS.md`. These are not foundation-blocking — the IR design, contracts, and uncertainty schema are done. These are refinement items that affect production readiness or specific adapter gaps.

**Success criteria:** Each investigated WDK produces a FINDINGS.md log entry with Scope, Method, Findings ([PROVEN]/[CONJECTURE] tags), Implications, Open Threads. Living synthesis updated accordingly. No prototype code changes expected.

**Constraints:** Per CLAUDE.md — append-only FINDINGS.md, read before writing, cite evidence, steel-man then stress-test. Use beads workflow for tracking.

## Key Decisions & Rationale

1. **Steps 12-14 narrowed the cross-track contracts**
   - Step 12: ComparisonProfileV1 (multi-metric divergence profile) is the trace-semantics → adversarial-reward interface
   - Step 13: Hybrid Option D (raw trajectory + derived summary) for convergence representation
   - Step 14: Candidate C (layered point uncertainty + optional tagged distribution) for UncertaintySummary
   - These are NARROWED, not RESOLVED — remaining work is field-level canonicalization, not architectural direction

2. **Parallel investigation is appropriate because the remaining WDKs are independent**
   - Each item concerns a different subsystem (type extensions, adapter gaps, allocation strategy)
   - No item blocks another
   - Each can be investigated by reading FINDINGS.md + relevant source files

3. **WDK#41-44 are NOT in scope for this parallel effort**
   - WDK#41 (reward aggregation) is the bridge to Priority 2 — too large for a sub-agent
   - WDK#42-44 (convergence taxonomy) are interdependent and need coordinated investigation
   - These four items are the natural "next major step," not parallel cleanup

## Current State

### Completed (Steps 1-14)
- 3 adapters (OpenMM mock, GROMACS, VASP) on shared EventKind types
- CausalOverlay with R14/R17/R18 queries, hidden confounder litmus validated
- ComparisonProfileV1 contract (Step 12), convergence representation (Step 13), UncertaintySummary schema (Step 14)
- 92/92 tests, strict clippy clean, 251ms overlay at 10^6 events
- Latest commit: `121711c` (Step 14 — UncertaintySummary schema)

### Candidate Items for Parallel Investigation

| WDK# | Topic | Scope | Key Files |
|-------|-------|-------|-----------|
| 25 | VASP closed-source ceiling impact | How often does correct fault classification require info not in vasprun.xml + OUTCAR + stdout? | `FINDINGS.md`, VASP adapter, cross-framework synthesis |
| 26 | INCAR classification table completeness | Validate ~200-300 INCAR parameter classifications (theory/implementation/ambiguous) | `FINDINGS.md`, `dsl-evaluation/cross-framework-synthesis.md` |
| 35 | ContractTerm needs `value: Option<Value>` | Machine-readable precondition checking for VASP Stage 3 | `common.rs:94-99`, VASP adapter |
| 36 | Value enum needs KnownMatrix variant | Spectral data (band structure over k-points) representation | `common.rs:102-108` |
| 39 | prediction_id type harmonization | String vs SpecElementId mismatch in ComparisonResult | `overlay.rs`, `common.rs` |

Also potentially investigable (lower priority, already narrowed):
- **WDK#27** — Streaming/buffering trade-off for Stage 3
- **WDK#38** — Arena allocation (already validated Vec-first, very low priority)

### Not In Scope
- WDK#41 (reward aggregation scalar) — major investigation, not a parallel cleanup item
- WDK#42-44 (convergence taxonomy) — interdependent cluster
- WDK#1-8 (DSL-specific unknowns) — lower priority, adapter-specific
- WDK#9 (statistical refutation chains) — connects to adversarial-reward track

## Key Code Context

**`common.rs:94-99`** — Current ContractTerm (WDK#35 target):
```rust
pub struct ContractTerm {
    pub description: String,
    pub layer: SemanticLayer,
    pub category: ContractCategory,
}
```

**`common.rs:102-108`** — Current Value enum (WDK#36 target):
```rust
pub enum Value {
    Scalar(f64),
    Text(String),
    Boolean(bool),
    KnownUnit { value: f64, unit: String },
}
```

**`common.rs:134-150`** — ComparisonOutcome + DivergenceMeasure (context for all WDKs):
```rust
pub struct ComparisonOutcome {
    pub agreement: AgreementLevel,
    pub divergence: Option<DivergenceMeasure>,
    pub detail: String,
}
```

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master investigation log (1527 lines) | Target for new entries |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/common.rs` | Shared IR types | Reference (WDK#35, 36, 39) |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/overlay.rs` | CausalOverlay + queries | Reference (WDK#39) |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/vasp_adapter.rs` | VASP adapter | Reference (WDK#25, 26) |
| `research/trace-semantics/dsl-evaluation/cross-framework-synthesis.md` | Cross-framework analysis | Reference (WDK#25, 26) |
| `.claude/prompts/prompt_009_...wdk40-uncertainty-schema.md` | Step 14 prompt (RISEN) | Created this session |

## Loop State

N/A — this session was a single prompt generation + Codex execution + verification cycle, not an iterative review loop. Step 14 was executed by Codex and verified in one pass.

## Next Steps

1. **Create a team** with TeamCreate for parallel WDK investigation
2. **Spawn sub-agents** (one per WDK item) — each reads FINDINGS.md and relevant source files, investigates, and produces a draft FINDINGS.md entry
3. **Recommended agent assignments:**
   - Agent 1: WDK#35 + WDK#36 (type extension pair — both are small, related Value/ContractTerm changes)
   - Agent 2: WDK#25 (VASP closed-source ceiling — needs domain reasoning about DFT failure modes)
   - Agent 3: WDK#26 (INCAR classification completeness — needs parameter-level analysis)
   - Agent 4: WDK#39 (prediction_id harmonization — needs overlay.rs analysis)
4. **Lead agent reviews** all draft entries for methodology compliance (evidence citations, [PROVEN]/[CONJECTURE] tags, mechanism + conditions)
5. **Append approved entries** to FINDINGS.md in reverse chronological order, update living synthesis
6. **Session close:** `bd close` all items, `bd sync`, commit, merge to main

## Session Artifacts

- Prompt: `.claude/prompts/prompt_009_2026-02-21_wdk40-uncertainty-schema.md` (RISEN, WDK#40)
- Commit: `121711c` (Step 14 — UncertaintySummary schema)
- Beads: `athena-6s4` (Step 14, closed)
- Previous handoff: `.claude/handoffs/handoff_010_2026-02-21_trace-semantics-open-threads.md`

## Documentation Updated

No documentation updates — all project docs were current.
