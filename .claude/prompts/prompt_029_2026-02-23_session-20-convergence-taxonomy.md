# Prompt: Session 20 — Cross-Framework Convergence Taxonomy

> Generated: 2026-02-23 | Framework: RISEN

---

## Session Goal

Create a structured implementation prompt for ATHENA Session 20 that: (1) extracts shared convergence derivation into `convergence.rs` with canonical taxonomy and cross-framework mapping (WDK#42), (2) decides and documents whether derivation lives adapter-inline calling shared utilities vs. Stage 1→2 post-pass, with explicit rationale (WDK#44), (3) extends OpenMM parser for real CSV format, (4) writes cross-framework equivalence tests — all while preserving non-negotiable invariants: minimum-data guard (window >= 4), no synthetic certainty, and derived provenance (`causal_refs` + `Completeness::Derived`).

## Framework Selection

- **Chosen:** RISEN (Role, Instructions, Steps, End Goal, Narrowing)
- **Rationale:** Complex multi-step implementation with 5 sequential phases, hard constraints (provenance invariants), and important boundaries (what NOT to do). RISEN's Steps component maps directly to phased plan; Narrowing captures non-negotiable invariants.
- **Alternatives considered:** TIDD-EC (strong on dos/don'ts but weaker at expressing sequential phases), CO-STAR (audience-oriented, not execution-oriented), Chain of Thought (reasoning over structured execution)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Unambiguous phases, explicit gate criteria, concrete file paths and line numbers |
| Specificity | 9/10 | Exact function signatures, line ranges, mapping table, enum variants |
| Context | 9/10 | Codebase structure, WDK references, existing test count, adapter differences |
| Completeness | 9/10 | All three deliverables, five phases, verification criteria, FINDINGS.md protocol |
| Structure | 10/10 | RISEN components well-separated, phases sequential with gates, narrowing precise |
| **Overall** | **9/10** | |

---

## Structured Prompt

ROLE:

You are a Rust systems engineer working on the ATHENA falsification-driven AI co-scientist prototype. You have deep familiarity with the LEL+DGR Hybrid IR crate at `research/trace-semantics/prototypes/lel-ir-prototype/`, its three framework adapters (VASP, GROMACS, OpenMM), and the `TraceEvent`/`LayeredEventLog` type system. You understand that this is a research prototype (not production code) and that ATHENA's core thesis requires canonical cross-framework failure signals for downstream causal reasoning.

INSTRUCTIONS:

Follow these governing principles throughout all phases:

1. **Pure refactor first.** Phase 1 must produce zero behavior change — identical test output before and after. Extract shared logic, don't redesign it.

2. **Preserve provenance invariants (non-negotiable).** Every derived `ConvergencePoint` must carry:
   - `causal_refs` linking back to the source energy events + any `ExecutionStatus`/`NumericalStatus` events
   - `Completeness::Derived { from_elements }` — never `FullyObserved` for derived signals
   - Minimum-data guard: window >= 4 energy events, return `None` below this threshold (no synthetic certainty)

3. **Taxonomy is a projection layer.** The `CanonicalConvergence` struct is computed *from* existing `TraceEvent` data. It does not modify the `ConvergencePoint` struct or any existing event types. It is a read-only classification.

4. **WDK#44 decision: adapter-inline with shared utility. No post-pass.** Derivation stays inside each adapter's `parse_trace` flow, but the duplicated computation moves to a shared `derive_energy_convergence_summary` function in `convergence.rs`. VASP remains unchanged — its SCF convergence mechanism is fundamentally different. Document the rationale: adapter-inline preserves natural provenance and avoids Stage 1→2 post-pass architecture cost; shared extraction eliminates code duplication.

5. **Backward compatibility is mandatory.** All 100 existing tests must pass unchanged. New code adds tests; it does not modify existing test assertions.

6. **Adhere to ATHENA research workflow.** Update `research/trace-semantics/FINDINGS.md` per the append-only investigation log protocol. Mark WDK#42 and WDK#44 as resolved with evidence citations.

STEPS:

1. **Phase 1 — Extract shared derivation into `convergence.rs`**
   - Create `src/convergence.rs` with:
     - `pub const MIN_CONVERGENCE_WINDOW: usize = 4;`
     - `pub const REL_DELTA_THRESHOLD: f64 = 1e-4;`
     - `pub fn derive_energy_convergence_summary(events: &[TraceEvent], source_file: &str) -> Option<TraceEvent>` — extract the logic currently duplicated verbatim between `gromacs_adapter.rs:544-653` and `adapter.rs:69-180`. Parameterize `source_file` so each adapter can pass its own provenance string.
   - Add `pub mod convergence;` to `src/lib.rs`.
   - In `gromacs_adapter.rs`: delete `GROMACS_MIN_CONVERGENCE_WINDOW`, `GROMACS_REL_DELTA_THRESHOLD`, and `derive_gromacs_convergence_summary`. Replace the call site with `convergence::derive_energy_convergence_summary(events, "simulation.log")`.
   - In `adapter.rs`: delete `OPENMM_MIN_CONVERGENCE_WINDOW`, `OPENMM_REL_DELTA_THRESHOLD`, and `derive_openmm_convergence_summary`. Replace the call site with `convergence::derive_energy_convergence_summary(events, "simulation.log")`.
   - **Gate:** `cargo test` passes 100/100. `cargo clippy -- -D warnings` is clean. This is a pure refactor with zero behavior change.

2. **Phase 2 — Canonical taxonomy types and mapping function**
   - Add to `convergence.rs`:
     ```rust
     pub enum ConvergencePattern { Converged, Oscillating, Stalled, Divergent, InsufficientData }
     pub enum ConvergenceConfidence { Direct, Derived, Absent }
     pub struct CanonicalConvergence {
         pub pattern: ConvergencePattern,
         pub confidence: ConvergenceConfidence,
         pub source_metric: String,
         pub source_framework: String,
     }
     ```
   - Implement `pub fn classify_convergence(event: &TraceEvent, framework: &str, log: &LayeredEventLog) -> CanonicalConvergence`:
     - Priority 1: Check log for `NumericalStatus{NaNDetected/InfDetected}` or `ExecutionStatus{CrashDivergent}` → `Divergent` (overrides all)
     - Priority 2: Match `ConvergencePoint { metric_name, converged, .. }` per mapping table:

       | metric_name | converged | Framework | → Pattern | → Confidence |
       |---|---|---|---|---|
       | `derived_convergence_rel_delta_max` | `Some(true)` | GROMACS/OpenMM | `Converged` | `Derived` |
       | `derived_oscillation_rel_delta_mean` | `Some(false)` | GROMACS/OpenMM | `Oscillating` | `Derived` |
       | `derived_stall_rel_delta_mean` | `Some(false)` | GROMACS/OpenMM | `Stalled` | `Derived` |
       | `dE` | `Some(true)` | VASP | `Converged` | `Direct` |
       | `dE` | `converged=None` | VASP | `InsufficientData` | `Direct` |
       | (unknown metric_name) | any | any | `InsufficientData` | `Absent` |

     - Confidence from `event.confidence.completeness`: `FullyObserved` → `Direct`, `Derived{..}` → `Derived`
   - Implement `pub fn classify_all_convergence(log: &LayeredEventLog, framework: &str) -> Vec<CanonicalConvergence>`: iterate log events, classify each `ConvergencePoint`.
   - Write ~8 tests: one per mapping table row, plus Divergent override priority, plus unknown metric_name edge case.
   - **Gate:** `cargo test` passes all old + new tests. `cargo clippy` clean.

3. **Phase 3 — OpenMM CSV parser extension**
   - Extend `parse_openmm_energy_series` in `adapter.rs`:
     - If first non-empty line starts with `#"Step"` or `#"` → CSV mode:
       - Parse header to find column index containing "Potential Energy" (real format: `#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)",...`)
       - Parse data rows by splitting on `,`
       - Extract (step, energy) using discovered column indices
     - Else → existing whitespace parsing (backward compatible)
   - Add test constant `OPENMM_CSV_STABLE` with real StateDataReporter CSV format.
   - Write ~4 tests: CSV parse produces correct (step, energy) pairs, CSV convergence classification, CSV oscillation classification, whitespace backward compat.
   - **Gate:** `cargo test` passes all tests. `cargo clippy` clean.

4. **Phase 4 — Cross-framework equivalence tests**
   - Six synthetic scenarios, each run through all applicable adapters → `classify_all_convergence` → assert same canonical pattern:

     | Scenario | VASP | GROMACS | OpenMM | Expected |
     |---|---|---|---|---|
     | A: Steady-state converged | OSZICAR with F= | Stable log | Stable CSV | `Converged` |
     | B: Oscillating | N/A (SCF-level) | Oscillating log | Oscillating CSV | `Oscillating` |
     | C: Stalled | N/A | Drifting log | Drifting data | `Stalled` |
     | D: Divergent (NaN) | OUTCAR error | NaN in log | NaN energy | `Divergent` |
     | E: Insufficient data | No F= line | <4 energies | <4 energies | `InsufficientData` |
     | F: Threshold boundary | N/A | Delta = 1e-4 exactly | Delta = 1e-4 exactly | Boundary test |

   - Document known asymmetry: VASP oscillation/stall detection operates at SCF level (not ionic) — scenarios B/C are N/A for VASP. This is a semantic distinction, not a taxonomy failure.
   - **Gate:** `cargo test` passes all tests (~120 total). `cargo clippy -- -D warnings` clean.

5. **Phase 5 — FINDINGS.md update**
   - Add investigation log entry at the top (reverse chronological):
     - **Date:** 2026-02-23
     - **Identifier:** Session 20
     - **Scope:** WDK#42 canonical convergence taxonomy, WDK#44 placement decision, OpenMM CSV validation
     - **Method:** Taxonomy design, shared derivation extraction, CSV parser extension, cross-framework equivalence tests
     - **Findings:** Enumerate each finding with code citations (e.g., `convergence.rs::derive_energy_convergence_summary`, `test_classify_convergence_converged_derived`, session log entry date)
     - **Open threads:** Any remaining items
   - Update Accumulated Findings:
     - WDK#42 → resolved. Evidence: `CanonicalConvergence` enum + `classify_convergence` implemented, all 6 equivalence scenarios pass, mapping table is code-backed.
     - WDK#44 → resolved. Evidence: shared extraction complete (`convergence::derive_energy_convergence_summary`), both adapters call it, A/B tradeoff analysis documented (adapter-inline preserves provenance, avoids post-pass architecture cost; shared extraction eliminates duplication).
     - Add new "What We Know" items for canonical taxonomy and placement decision
   - Update Prototype Index: add `convergence.rs` entry with purpose, status, and what it demonstrated.

END GOAL:

Session 20 is complete when ALL of the following are true:
1. `cargo test` passes ~120 tests (100 existing unchanged + ~20 new)
2. `cargo clippy -- -D warnings` produces zero warnings
3. The duplicated derivation logic between `gromacs_adapter.rs` and `adapter.rs` is eliminated — both call `convergence::derive_energy_convergence_summary`
4. `CanonicalConvergence` taxonomy correctly maps all three frameworks' convergence signals to 5 canonical patterns
5. Cross-framework equivalence tests prove: same physical scenario → same canonical label regardless of framework
6. OpenMM CSV parser handles real `StateDataReporter` output format while preserving whitespace backward compat
7. Every derived `ConvergencePoint` still carries `causal_refs`, `Completeness::Derived`, and respects the window >= 4 minimum-data guard
8. WDK#42 and WDK#44 are marked resolved in FINDINGS.md with evidence citations
9. WDK#44 rationale is documented: adapter-inline chosen over post-pass because it preserves natural provenance and avoids architecture cost; shared extraction eliminates the duplication con
10. All code changes are committed and pushed

NARROWING:

- **Do NOT** modify the `ConvergencePoint` struct or any existing event types. The taxonomy is a projection, not a schema change.
- **Do NOT** create a Stage 1→2 post-pass architecture. WDK#44 decision is adapter-inline with shared utility.
- **Do NOT** modify VASP adapter convergence logic. VASP SCF convergence is fundamentally different and correct as-is.
- **Do NOT** emit `ConvergencePoint` events for windows < 4. Return `None` — silence is correct for insufficient data.
- **Do NOT** use `Completeness::FullyObserved` for GROMACS/OpenMM derived signals. These are always `Derived`.
- **Do NOT** edit or delete previous FINDINGS.md investigation log entries. The log is append-only.
- **Do NOT** modify existing test assertions. New tests are additive.
- **Do NOT** write production code. This is a research prototype.
- **Avoid** grant-proposal rhetoric ("groundbreaking", "revolutionary") in FINDINGS.md entries.
- **Stay within** the `research/trace-semantics/prototypes/lel-ir-prototype/` directory for all code changes.
- **Constraint:** All paths in the plan are relative to `research/trace-semantics/prototypes/lel-ir-prototype/`.
- **Known asymmetry (not a bug):** VASP oscillation/stall detection is SCF-level, not ionic-level. Scenarios B/C in equivalence tests are N/A for VASP.

---

## Review Findings

### Issues Addressed
1. **CRITICAL-3 (resolution criteria):** Added explicit evidence threshold — WDK#42 resolved = enum + classify + 6 scenarios pass; WDK#44 resolved = shared extraction + both adapters call it + A/B rationale documented.
2. **WARNING-3 (CSV format):** Added specific StateDataReporter column header: `#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)",...`
3. **SUGGESTION-2 (citation format):** Added citation format guidance: `convergence.rs::<fn>`, `test_<name>`, session log date.

### Remaining Suggestions
- Consider renaming `adapter.rs` → `openmm_adapter.rs` for naming consistency (out of scope for Session 20)
- Consider adding gate failure handling expectations (reasonable default: fix and re-run)
- Consider consolidating scenario table into preamble section (stylistic preference)
- Consider adding provenance code example to Governing Principle #2 (would add clarity but not blocking)

## Usage Notes

- **Best used with:** Claude Code in a single session with crate access at `research/trace-semantics/prototypes/lel-ir-prototype/`
- **Adjust for:** If test count diverges from ~120 estimate, the important criterion is that all 100 existing tests pass unchanged and equivalence scenarios A-F all pass
