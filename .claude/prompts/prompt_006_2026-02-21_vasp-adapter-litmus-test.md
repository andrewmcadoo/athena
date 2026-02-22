# Prompt: VASP Adapter + Hidden Confounder Litmus Test

> Generated: 2026-02-21 | Prompt #1 | Framework: RISEN

---

## Session Goal

Create a structured prompt that instructs an AI coding agent to implement a VASP adapter for the ATHENA trace-semantics IR prototype (parsing INCAR/OSZICAR/OUTCAR into LayeredEventLog), along with a hidden confounder litmus test proving end-to-end confounder detection, plus documentation updates — following a detailed implementation plan already written.

## Framework Selection

- **Chosen:** RISEN (Role, Instructions, Steps, End Goal, Narrowing)
- **Rationale:** Complex multi-step implementation task with clear sequential phases (adapter → tests → litmus test → docs), a specific role (AI coding agent), detailed methodology, explicit constraints, and a well-defined end goal. RISEN maps directly to these components.
- **Alternatives considered:** TIDD-EC (good for dos/don'ts but weaker at encoding sequential steps), Chain of Thought (useful for reasoning but this task is more execution than reasoning)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal is unambiguous: implement VASP adapter + litmus test + docs |
| Specificity | 10/10 | Plan specifies exact types, function signatures, test names, line counts |
| Context | 9/10 | Full source code patterns provided; existing adapter as reference |
| Completeness | 9/10 | Covers what/why/how, output format (passing tests), verification criteria |
| Structure | 9/10 | RISEN maps naturally to this sequential multi-step implementation |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

ROLE:
You are an expert Rust systems programmer implementing a VASP (Vienna Ab initio Simulation Package) adapter for the ATHENA trace-semantics IR prototype. You have deep knowledge of:
- Rust idioms (pattern matching, Result/Option chaining, iterators)
- The existing codebase: `LayeredEventLog`, `TraceEventBuilder`, `CausalOverlay`, `EventKind` variants, `BoundaryClassification`, `DslAdapter` trait
- The GROMACS adapter pattern (`gromacs_adapter.rs`) which you will mirror structurally
- VASP output file formats (INCAR, OSZICAR, OUTCAR)
- Causal graph construction and confounder detection via common-cause ancestry

INSTRUCTIONS:
- Mirror the GROMACS adapter (`src/gromacs_adapter.rs`) pattern exactly: public struct, classify function, per-file parsers, `DslAdapter` trait implementation, causal wiring, `LayeredEventLogBuilder` assembly.
- Use existing types only. Do NOT modify `common.rs`, `event_kinds.rs`, `lel.rs`, `adapter.rs`, `overlay.rs`, `gromacs_adapter.rs`, or `Cargo.toml`. The VASP adapter must prove the IR handles DFT without type changes.
- Exercise two previously-unused `EventKind` variants: `ConvergencePoint` (from OSZICAR SCF data) and `StateSnapshot` (from OUTCAR force blocks).
- All parsing is line-based text processing. VASP uses `!` and `#` for comments (not `;` like GROMACS). VASP parameter keys are uppercase by convention.
- **Normalization convention (deviation from GROMACS):** In the VASP adapter, both `ParameterRecord.name` AND `dag_node_ref` use the uppercase-normalized key (via `key.trim().to_ascii_uppercase()`). This differs from `parse_mdp` which uses the raw trimmed key. The reason: VASP convention is uppercase keys, and the litmus test depends on `by_variable` index (keyed on `name`) and `dag_node_ref` being consistent.
- Every test function name must start with `test_vasp_`. Use the existing test helpers (`setup()`, `test_provenance()`, `test_spec()`, `test_experiment_ref()`) and the existing import pattern from `src/tests/mod.rs`.
- The hidden confounder litmus test must prove that `CausalOverlay::detect_confounders` returns a non-empty result containing the planted confounder node. This is the key validation.
- All parser functions (`parse_incar`, `parse_oszicar`, `parse_outcar`) must be `pub fn` — they are imported directly in tests.
- This is prototype research code (not production). Favor clarity and correctness over abstraction.
- **Priority:** Steps 1-4 are core deliverables. Steps 5-7 are documentation updates that can be deferred if needed.

STEPS:

1. **Create `src/vasp_adapter.rs`** (~300-500 lines) with these components:

   a. **Imports and struct:**
   ```rust
   use crate::adapter::{AdapterError, DslAdapter};
   use crate::common::*;
   use crate::event_kinds::EventKind;
   use crate::lel::*;
   pub struct VaspAdapter;
   ```
   Section markers: `const INCAR_MARKER: &str = "--- INCAR ---";`, `const OSZICAR_MARKER: &str = "--- OSZICAR ---";`, `const OUTCAR_MARKER: &str = "--- OUTCAR ---";`

   b. **`pub fn classify_incar_parameter(key: &str, _value: &str) -> (Layer, BoundaryClassification, Option<&'static str>)`**
   Normalize key via `key.trim().to_ascii_uppercase()`. Match table:

   | Key(s) | Layer | Boundary | Unit |
   |--------|-------|----------|------|
   | `GGA`, `METAGGA`, `ISMEAR` | Theory | PrimaryLayer | None |
   | `ENCUT` | Theory | DualAnnotated { secondary_layer: Implementation, rationale: "cutoff determines both physics accuracy and memory/compute cost" } | `"eV"` |
   | `PREC` | Theory | DualAnnotated { secondary_layer: Implementation, rationale: "precision affects both physical accuracy and FFT grid resources" } | None |
   | `SIGMA` | Theory | DualAnnotated { secondary_layer: Methodology, rationale: "smearing width affects both electronic structure accuracy and BZ integration convergence" } | `"eV"` |
   | `IBRION`, `NSW`, `ISIF`, `POTIM` | Methodology | PrimaryLayer | None |
   | `EDIFF` | Methodology | PrimaryLayer | `"eV"` |
   | `EDIFFG` | Methodology | PrimaryLayer | `"eV/Ang"` |
   | `NCORE`, `KPAR`, `NPAR`, `NSIM`, `NELM` | Implementation | PrimaryLayer | None |
   | `ALGO` | Implementation | DualAnnotated { secondary_layer: Methodology, rationale: "algorithm can affect which SCF minimum is found" } | None |
   | `LREAL` | Implementation | DualAnnotated { secondary_layer: Theory, rationale: "real-space projection trades accuracy for speed" } | None |
   | Unknown `_` | Implementation | ContextDependent { default_layer: Layer::Implementation, context_note: "VASP parameter not in classification table" } | None |

   c. **`pub fn parse_incar(content: &str) -> Result<Vec<TraceEvent>, AdapterError>`**
   Mirror `parse_mdp` structurally, with these specific differences:
   - Comments: strip via `raw_value.split(['!', '#']).next()` (VASP uses `!` and `#`, not `;`)
   - Provenance `source_file`: `"INCAR"`
   - Both `ParameterRecord.name` AND `dag_node_ref`: use `key.trim().to_ascii_uppercase()` (the uppercase-normalized key)
   Everything else identical to `parse_mdp`: line iteration, skip empty lines, `split_once('=')`, classify, numeric parse with fallback to `KnownCat`, build `ParameterRecord` events with `TraceEventBuilder`.

   d. **`pub fn parse_oszicar(content: &str, seq_offset: u64) -> Result<Vec<TraceEvent>, AdapterError>`**
   Parse OSZICAR format. OSZICAR lines look like:
   ```
   DAV:   1    0.328E+03    0.328E+03   -0.994E+03   288   0.461E+02
   DAV:   2   -0.159E+02   -0.344E+03    0.286E+03   432   0.131E+02
      1 F= -.11401725E+03 E0= -.11400000E+03  d E =-.11401725E+03
   ```

   Algorithm:
   - `logical_sequence` starts at `seq_offset + 1`
   - Track `current_ionic_step` (starts at 0, incremented from `F=` summary lines)
   - For each line starting with `DAV:` or `RMM:` (after trim) → emit `ConvergencePoint` at `Layer::Methodology`
     - Parse iteration number from token[1] (after split_whitespace)
     - Parse dE from token[3]
     - `metric_name`: `"dE"`, `metric_value`: `Value::Known(dE, "eV".to_string())`, `converged`: `None`
     - Provenance `source_file`: `"OSZICAR"`, line range: current line number
   - For each line containing `F=` → emit `EnergyRecord` at `Layer::Implementation`
     - Extract `current_ionic_step` from first whitespace-delimited integer token
     - Extract energy: split on `F=`, take second half, first whitespace token, parse as f64
     - Extract E0: split on `E0=`, take second half, first whitespace token
     - Extract dE: split on `d E =` or `dE =`, first whitespace token after
     - Components: vec of `("E0", Value::Known(e0, "eV"))` and `("dE", Value::Known(de, "eV"))` where parseable
     - Before emitting the EnergyRecord: walk backward through the events collected so far to find the last `ConvergencePoint` event, and set its `converged` field to `Some(true)`
   - Lines matching neither pattern are skipped.

   e. **`pub fn parse_outcar(content: &str, seq_offset: u64) -> Result<Vec<TraceEvent>, AdapterError>`**
   Parse OUTCAR format. `logical_sequence` starts at `seq_offset + 1`. Scan lines for these patterns:
   - Line containing `"vasp."` or `"VASP"` → extract version string for `ResourceStatus`. `platform_type`: extract version portion. `device_ids`: `vec![version_string]`. `memory_allocated`: None. `memory_peak`: None. `parallelization`: None initially (may be updated). `warnings`: empty vec. Layer: Implementation.
   - Line containing `"running on"` and `"total cores"` → extract core count integer, set `parallelization` field on the ResourceStatus event (e.g., `Some("N cores".to_string())`). If ResourceStatus already emitted, update it; otherwise store for later.
   - Line containing `"free  energy   TOTEN"` → extract energy value after `=`, emit `EnergyRecord` at `Layer::Implementation`. `total`: `Value::Known(energy, "eV".to_string())`. `components`: empty vec. Provenance `source_file`: `"OUTCAR"`.
   - Line containing `"POSITION"` and `"TOTAL-FORCE"` → emit `StateSnapshot { snapshot_type: SnapshotType::Forces, data_ref: "OUTCAR:forces".to_string() }` at `Layer::Implementation`.
   - Line containing `"General timing and accounting"` → emit `ExecutionStatus { status: ExecutionOutcome::Success, framework_error_id: None }`.
   - Line containing `"EDDDAV"` or `"VERY BAD NEWS"` → emit `ExecutionStatus { status: ExecutionOutcome::CrashDivergent, framework_error_id: None }`.
   - If no completion/error marker found after scanning all lines → emit `ExecutionStatus { status: ExecutionOutcome::Timeout, framework_error_id: None }` with `PartiallyInferred { inference_method: "no completion marker in OUTCAR" }` confidence and `field_coverage: 0.5`.
   - Lines matching none of the above patterns are skipped.

   f. **`impl DslAdapter for VaspAdapter`**
   The `parse_trace` method:
   1. Find positions of 3 section markers (order-agnostic). Each section extends from its marker to the next marker or EOF.
   2. Fallback: if no markers found and lines contain `=`, treat entire input as INCAR.
   3. Parse each section: `parse_incar` → `parse_oszicar(incar_count)` → `parse_outcar(incar_count + oszicar_count)`
   4. Causal wiring (mutate events after parsing):
      - Collect `incar_event_ids: Vec<EventId>` from INCAR events
      - For OSZICAR `ConvergencePoint` events: set `causal_refs = incar_event_ids.clone()`
      - For OSZICAR `EnergyRecord` events: set `causal_refs` to vec containing the last ConvergencePoint event's ID
      - For OUTCAR `EnergyRecord` and `StateSnapshot` events: set `causal_refs = incar_event_ids.clone()`
      - For OUTCAR `ExecutionStatus`: set `causal_refs` to last energy event ID (from either OSZICAR or OUTCAR, whichever is last)
   5. Controlled variables: empty vec (DFT has no temperature/pressure thermostats like MD)
   6. ExperimentRef: `experiment_id: "vasp-trace"`, `cycle_id: 0`, `hypothesis_id: "H0-vasp-adapter"`
   7. Spec provenance: `source_file: "INCAR"`, `source_location: SourceLocation::ExternalInput`, `raw_hash: 0`
   8. Assemble via `LayeredEventLogBuilder::new(experiment_ref, spec)`, add all events in order (INCAR first, then OSZICAR, then OUTCAR), `.build()`

2. **Register module in `src/lib.rs`:**
   Add `pub mod vasp_adapter;` after the `pub mod gromacs_adapter;` line.

3. **Add tests to `src/tests/mod.rs`:**

   First, add imports at the top of the file (after existing GROMACS imports):
   ```rust
   use crate::vasp_adapter::{classify_incar_parameter, parse_incar, parse_oszicar, parse_outcar, VaspAdapter};
   ```

   Then add 6 sample data constants:
   - `VASP_INCAR_SAMPLE`: 13 representative INCAR parameters covering all 3 layers, including inline comments with `!` and `#`. Example:
     ```
     GGA = PE
     ENCUT = 520 ! cutoff energy
     PREC = Accurate
     SIGMA = 0.05 # smearing width
     ISMEAR = 0
     IBRION = 2
     NSW = 50
     ISIF = 3
     EDIFF = 1E-6
     EDIFFG = -0.01
     NCORE = 4
     KPAR = 2
     ALGO = Fast
     ```
   - `VASP_OSZICAR_SAMPLE`: 2 ionic steps, each with 3-4 SCF iterations (DAV: lines) plus `F=` summary line
   - `VASP_OUTCAR_SAMPLE`: version line (containing "vasp."), core count line ("running on N total cores"), TOTEN energy line, POSITION/TOTAL-FORCE block header, "General timing and accounting" completion
   - `VASP_OUTCAR_TRUNCATED`: version line + TOTEN energy but NO completion marker (for Timeout test)
   - `VASP_OUTCAR_ERROR`: version line + "VERY BAD NEWS" error marker
   - `VASP_COMBINED_SAMPLE`: all 3 sections joined with `--- INCAR ---\n`, `--- OSZICAR ---\n`, `--- OUTCAR ---\n` markers

   Then add 25 test functions:

   **A. Classification (5 tests):**
   - `test_vasp_classify_theory_params` — `GGA`, `ISMEAR` → (Theory, PrimaryLayer, None)
   - `test_vasp_classify_methodology_params` — `IBRION`, `EDIFF`, `NSW` → Methodology/PrimaryLayer
   - `test_vasp_classify_implementation_params` — `NCORE`, `KPAR` → Implementation/PrimaryLayer
   - `test_vasp_classify_dual_annotated` — `ENCUT` (→Implementation), `ALGO` (→Methodology), `LREAL` (→Theory), `SIGMA` (→Methodology), `PREC` (→Implementation) — all return DualAnnotated with correct secondary layers and rationale strings
   - `test_vasp_classify_unknown_param` — `MYSTERY` → (Implementation, ContextDependent, None)

   **B. INCAR parsing (4 tests):**
   - `test_vasp_parse_incar_basic` — Parse `VASP_INCAR_SAMPLE`, assert 13 ParameterRecord events, spot-check names and values
   - `test_vasp_parse_incar_comments_stripped` — Verify `!` and `#` inline comments are removed from values (e.g., ENCUT value is `520`, not `520 ! cutoff energy`)
   - `test_vasp_parse_incar_empty` — Empty string → `Ok(vec![])`
   - `test_vasp_parse_incar_layer_distribution` — Assert events span all 3 layers (Theory, Methodology, Implementation)

   **C. OSZICAR parsing (4 tests):**
   - `test_vasp_parse_oszicar_convergence_points` — Assert correct count of `ConvergencePoint` events from `VASP_OSZICAR_SAMPLE` (should match total DAV:/RMM: lines)
   - `test_vasp_parse_oszicar_energy_records` — Assert 2 `EnergyRecord` events with correct total energies matching the `F=` values
   - `test_vasp_parse_oszicar_convergence_flagged` — Assert last SCF ConvergencePoint before each ionic summary has `converged == Some(true)`
   - `test_vasp_parse_oszicar_single_step` — Minimal 1-step input (2 DAV: lines + 1 F= line) → 2 ConvergencePoint + 1 EnergyRecord

   **D. OUTCAR parsing (4 tests):**
   - `test_vasp_parse_outcar_resource_status` — Version string and parallelization core count extracted correctly
   - `test_vasp_parse_outcar_energy_and_forces` — `EnergyRecord` (TOTEN) + `StateSnapshot { Forces }` both present
   - `test_vasp_parse_outcar_success` — `ExecutionOutcome::Success` from "General timing and accounting" marker
   - `test_vasp_parse_outcar_truncated` — `ExecutionOutcome::Timeout` + `Completeness::PartiallyInferred` confidence

   **E. Integration (5 tests):**
   - `test_vasp_adapter_combined` — Full 3-section `VASP_COMBINED_SAMPLE` trace parsed via `VaspAdapter.parse_trace()`, assert all EventKind variants present (ParameterRecord, ConvergencePoint, EnergyRecord, StateSnapshot, ResourceStatus, ExecutionStatus)
   - `test_vasp_adapter_incar_only` — INCAR-only trace (no markers, lines contain `=`) → all events are ParameterRecord
   - `test_vasp_adapter_controlled_vars_empty` — `log.spec.controlled_variables.is_empty()` (DFT has no thermostats)
   - `test_vasp_overlay_construction` — `CausalOverlay::from_log(&log)` builds without panic, entity count matches event count
   - `test_vasp_overlay_layer_span` — All 3 layers present in `log.indexes.by_layer` keys

   **F. Litmus + error (3 tests):**
   - `test_vasp_hidden_confounder_litmus` (~60 lines):
     1. Parse `VASP_COMBINED_SAMPLE` through `VaspAdapter.parse_trace()` → get `log`
     2. Find event positions using two-step lookup: `by_variable` gives `Vec<EventId>`, then `by_id` resolves each `EventId` to a `usize` position in `log.events`. Get positions for `"PREC"`, `"SIGMA"`, and `"IBRION"`.
     3. Get the `EventId` of the PREC event: `log.events[prec_pos].id`
     4. Plant confounder by mutation: `log.events[sigma_pos].causal_refs = vec![prec_event_id]` and `log.events[ibrion_pos].causal_refs = vec![prec_event_id]`
     5. Rebuild indexes (or rebuild the log) to reflect the new causal structure. Build `CausalOverlay::from_log(&log)`.
     6. Call `overlay.detect_confounders(&log, "SIGMA", "IBRION")`
     7. Assert: result is non-empty AND contains at least one candidate with `dag_node == "PREC"`
   - `test_vasp_hidden_confounder_controlled_excluded` (~40 lines):
     Same planted-confounder setup as above, but additionally add a `ControlledVariable` to `log.spec.controlled_variables` with `parameter: "PREC".to_string()` (must match the `dag_node_ref` value exactly). Assert `detect_confounders(&log, "SIGMA", "IBRION")` returns empty vec (controlled params are excluded per R14 semantics).
   - `test_vasp_adapter_error_execution` — Parse `VASP_OUTCAR_ERROR` through `parse_outcar`, find the `ExecutionStatus` event, assert `status == ExecutionOutcome::CrashDivergent`.

4. **Verify build:**
   Run `cargo test` in `research/trace-semantics/prototypes/lel-ir-prototype/` — target 92 tests passing.
   Run `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings.

5. **Update `evaluation/hidden-confounder/README.md`:**
   In the Current Status section, update from "NOT STARTED" to note that a prototype-level validation exists in the LEL IR prototype test suite (`test_vasp_hidden_confounder_litmus`), while the full 50-cycle evaluation environment still depends on adversarial-reward research.

6. **Update `research/trace-semantics/FINDINGS.md`:**
   Append two investigation log entries at the top of the Investigation Log section (reverse chronological, newest first):

   **Entry: Step 11 — Hidden Confounder Prototype Litmus Test**
   - Scope: Validate that the R14 confounder detection mechanism works end-to-end on VASP-derived LEL data
   - Method: Planted PREC as common ancestor of SIGMA and IBRION via causal_refs mutation, then ran detect_confounders
   - Findings: detect_confounders correctly identifies PREC as confounder candidate; controlled variable exclusion works correctly
   - Implications: LEL IR has diagnostic value beyond parsing — it supports causal queries that detect hidden confounders
   - Open Threads: Full 50-cycle evaluation environment still needs adversarial-reward formalization (depends on research/adversarial-reward)

   **Entry: Step 10 — VASP Adapter Implementation**
   - Scope: Parse VASP INCAR/OSZICAR/OUTCAR into LEL using existing types only, answering "What We Don't Know" item #12: "Whether a single IR schema can accommodate both DFT codes (VASP) and MD codes (OpenMM, GROMACS)"
   - Key findings: (1) No EventKind changes needed — affirmatively answers WDK#12, (2) ConvergencePoint and StateSnapshot exercised successfully for the first time by any adapter, (3) Multi-file section-marker composition works for 3 input files
   - Implications: Single IR handles both MD (GROMACS/OpenMM) and DFT (VASP); DSL-only architectural constraint holds across simulation paradigms
   - Open Threads: Real VASP output validation with actual calculation results, POTCAR pseudopotential parsing

   Update Accumulated Findings section:
   - Move WDK#12 ("Can one IR handle both DFT and MD?") from "What We Don't Know" to "What We Know" with affirmative answer citing Step 10 evidence
   - Update ConvergencePoint/StateSnapshot status notes to "exercised by VASP adapter (Step 10)"

   Add `vasp_adapter.rs` to the Prototype Index table: purpose "VASP INCAR/OSZICAR/OUTCAR → LEL adapter", status "Complete", demonstrated "DFT compatibility, ConvergencePoint + StateSnapshot coverage"

   Update the Status line at the top to reflect Steps 10-11 and the new test count (92).

7. **Beads workflow:**
   Close beads issues (skip if already closed): `bd close athena-7ob athena-9cp`
   Sync: `bd sync --from-main`
   Stage and commit all changes.

END GOAL:
- `cargo test` passes with 92 tests (67 existing + 25 new) — zero failures
- `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings
- `test_vasp_hidden_confounder_litmus` asserts that `detect_confounders` returns non-empty candidates containing `dag_node == "PREC"`
- `test_vasp_hidden_confounder_controlled_excluded` asserts controlled params are excluded from confounder results
- No existing tests break — zero modifications to existing source files (common.rs, event_kinds.rs, lel.rs, adapter.rs, overlay.rs, gromacs_adapter.rs, Cargo.toml)
- FINDINGS.md has two new log entries and updated synthesis sections
- evaluation/hidden-confounder/README.md status updated from "NOT STARTED"

NARROWING:
- Do NOT modify any existing source files: `common.rs`, `event_kinds.rs`, `lel.rs`, `adapter.rs`, `overlay.rs`, `gromacs_adapter.rs`, `Cargo.toml`
- Do NOT add new dependencies or new EventKind variants
- Do NOT write production code — this is prototype research code in `prototypes/`
- Do NOT over-engineer: no generics, no trait abstractions beyond `DslAdapter`, no feature flags
- Do NOT change or break existing tests — all 67 must still pass unchanged
- Do NOT use `bd edit` (opens interactive editor) — use `bd update` and `bd close` instead
- Stay within `research/trace-semantics/prototypes/lel-ir-prototype/` for Rust code changes
- Prototype-scoped technology choices are fine; no ADR needed
- Keep FINDINGS.md investigation log append-only (new entries at top, never edit previous entries; the Accumulated Findings and Prototype Index sections ARE updated as evidence accumulates)
- Avoid grant-proposal rhetoric ("groundbreaking", "revolutionary") in documentation

---

## Review Findings

### Issues Addressed
1. **Critical: dag_node_ref/name consistency** — Explicitly stated that both `ParameterRecord.name` AND `dag_node_ref` use uppercase-normalized key, deviating from GROMACS `parse_mdp` pattern. Rationale provided.
2. **Critical: ContextDependent missing struct fields** — Added `default_layer: Layer::Implementation, context_note: "VASP parameter not in classification table"` to classification table.
3. **Critical: "Open Question #12" reference** — Changed to "What We Don't Know item #12" with exact question quoted.
4. **Warning: Litmus test mutation mechanics** — Added two-step lookup pattern (by_variable → EventId → by_id → position) and explicit mutation steps.
5. **Warning: framework_error_id omission** — Added `framework_error_id: None` to Timeout ExecutionStatus.
6. **Warning: pub visibility** — Explicitly stated all parser functions are `pub fn`.
7. **Warning: Priority ordering** — Added priority note: Steps 1-4 core, Steps 5-7 deferrable.

### Remaining Suggestions
- OSZICAR sample data format could benefit from inline example (mitigated by the example already in Step 1d)
- OUTCAR skip-line behavior could be more explicit (noted "Lines matching none of the above patterns are skipped")
- Line count estimate widened to ~300-500 to avoid false precision
- Controlled variable `parameter` field must match `dag_node_ref` exactly (noted in test description)

## Usage Notes

- **Best used with:** Claude Opus 4.6 or Sonnet 4.6 in Claude Code CLI with full codebase access
- **Adjust for:** If the test count differs from 67 existing (check with `cargo test` first), adjust the 92 target accordingly. The 25 new tests are the fixed addition.
