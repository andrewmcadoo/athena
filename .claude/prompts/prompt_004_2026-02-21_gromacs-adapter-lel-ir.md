# Prompt: GROMACS Adapter for LEL IR Prototype

> Generated: 2026-02-21 | Prompt #1 | Framework: RISEN

---

## Session Goal

Create a structured prompt that guides an AI coding agent to implement a GROMACS adapter module for the LEL IR prototype — parsing real GROMACS `.mdp` and `.log` files into the existing `LayeredEventLog` type system, with ~22 new tests, to validate that the IR generalizes beyond OpenMM.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation task with clear sequential dependencies (scaffold -> classifiers -> parsers -> integration -> tests -> docs), a well-defined end goal (66 passing tests, clippy clean), and important constraints (no new dependencies, hand-written parsers, no type changes). RISEN's Role + Instructions + Steps + End Goal + Narrowing maps directly onto this.
- **Alternatives considered:** TIDD-EC (strong dos/don'ts fit but weaker on sequential methodology); Chain of Thought (too lightweight for this implementation depth).

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal, steps, and deliverables are unambiguous; layer classification table is precise |
| Specificity | 9/10 | Exact parameter classifications, file paths, test names, and verification commands |
| Context | 9/10 | Full type system, existing adapter pattern, and FINDINGS.md protocol embedded |
| Completeness | 9/10 | Covers scaffold through docs; all EventKind variants and builder patterns referenced |
| Structure | 9/10 | RISEN components well-separated; steps sequential with clear substeps |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a Rust systems programmer implementing a DSL trace adapter for the ATHENA research project's LEL IR prototype. You have expertise in: hand-written text parsers (no regex crate), the existing LEL type system (TraceEvent, EventKind, LayeredEventLogBuilder, CausalOverlay), GROMACS molecular dynamics simulation output formats (.mdp parameter files, .log execution logs), and the project's research workflow (FINDINGS.md append-only protocol). You are working inside an existing, well-tested Rust crate (44 passing tests, clippy clean) and must preserve all existing functionality while adding the new adapter.

Working directory: research/trace-semantics/prototypes/lel-ir-prototype/

INSTRUCTIONS:
Follow these governing principles throughout:

1. **Reuse existing types exclusively.** Every GROMACS event must map to an existing EventKind variant (ParameterRecord, EnergyRecord, ResourceStatus, ExecutionStatus, NumericalStatus). If a GROMACS output element cannot be represented by an existing variant, flag it as a finding — do NOT add new variants.

2. **Mirror the MockOpenMmAdapter pattern.** Study `src/adapter.rs` (MockOpenMmAdapter) for the DslAdapter trait implementation pattern: ExperimentRef construction, ExperimentSpec with ControlledVariable entries, TraceEventBuilder fluent API, LayeredEventLogBuilder for event assembly.

3. **Hand-written parsers only.** No new crate dependencies. Parse .mdp key=value lines and .log fixed-format output with string operations (split, trim, parse::<f64>(), starts_with, etc.).

4. **Provenance traceability.** Every TraceEvent must carry a ProvenanceAnchor with SourceLocation::LineRange referencing the actual line numbers (1-indexed) from the input text. Use "input.mdp" for MDP events and "simulation.log" for LOG events as source_file values. Set `raw_hash` to 0 for all ProvenanceAnchors — content hashing is deferred to production scope.

5. **Layer classification must be deterministic.** Each GROMACS parameter maps to exactly one Layer + BoundaryClassification based on the classification table (Step 2). Unknown parameters default to Implementation + ContextDependent { default_layer: Implementation, context_note: "GROMACS parameter not in classification table" }.

6. **Monotonic temporal ordering.** MDP events get simulation_step: 0, wall_clock_ns: None, with monotonically increasing logical_sequence starting at 1. LOG events get simulation_step from the Step line preceding each energy block. logical_sequence continues from where MDP left off (passed as seq_offset to parse_log).

7. **Test-then-verify cadence.** After each implementation step, run `cargo test` to confirm no regressions, then `cargo clippy --all-targets --all-features -- -D warnings` for zero warnings.

8. **DualAnnotated rationale strings.** For parameters classified as DualAnnotated, provide a descriptive rationale explaining why the parameter spans layers. Examples: rcoulomb -> "Cutoff radius affects both force field accuracy and computational performance", dt -> "Timestep affects both sampling methodology and numerical stability".

STEPS:

1. **Scaffold the module.**
   - Create `src/gromacs_adapter.rs` with: `use` imports from `crate::common::*`, `crate::event_kinds::EventKind`, `crate::lel::*`, `crate::adapter::*`; `pub struct GromacsAdapter;`; section marker constants `const MDP_MARKER: &str = "--- MDP ---";` and `const LOG_MARKER: &str = "--- LOG ---";`; skeleton `DslAdapter` impl returning `Err(AdapterError::UnsupportedFormat("not yet implemented".into()))`.
   - Add `pub mod gromacs_adapter;` to `src/lib.rs` (before the `#[cfg(test)]` line).
   - Run `cargo test` — all 44 existing tests must still pass.

2. **Implement the MDP parameter classifier.**
   - Create `pub fn classify_mdp_parameter(key: &str, _value: &str) -> (Layer, BoundaryClassification, Option<&'static str>)`.
   - Implement the classification table:
     - Theory + PrimaryLayer: coulombtype, vdwtype, fourierspacing, pme_order
     - Theory + DualAnnotated { secondary_layer: Methodology, rationale: "Cutoff radius affects both force field accuracy and computational performance" }: rcoulomb (units: "nm"), rvdw (units: "nm")
     - Methodology + PrimaryLayer: integrator, nsteps, tcoupl, ref_t (units: "K"), pcoupl, ref_p (units: "bar"), gen_vel, gen_temp (units: "K")
     - Methodology + DualAnnotated { secondary_layer: Implementation, rationale: "Timestep affects both sampling methodology and numerical stability" }: dt (units: "ps"), tau_t (units: "ps"), tau_p (units: "ps")
     - Methodology + DualAnnotated { secondary_layer: Implementation, rationale: "Constraint algorithm choice affects both sampling validity and computational cost" }: constraints
     - Implementation + PrimaryLayer: nstlog, nstxout, nstenergy, nstlist
     - Unknown parameters: Layer::Implementation, BoundaryClassification::ContextDependent { default_layer: Layer::Implementation, context_note: "GROMACS parameter not in classification table".to_string() }, units: None

3. **Implement the MDP parser.**
   - Create `pub fn parse_mdp(content: &str) -> Result<Vec<TraceEvent>, AdapterError>`.
   - For each line: skip empty lines and lines starting with `;`. Split on the first `=`. Trim both key and value. Strip inline `;` comments from the value (split on `;`, take first part, trim).
   - Classify the parameter. Try `value.parse::<f64>()` — if it succeeds, use `Value::Known(v, unit.unwrap_or("").to_string())`; otherwise use `Value::KnownCat(value.to_string())`.
   - Build a TraceEvent via TraceEventBuilder with: the classified Layer and BoundaryClassification, EventKind::ParameterRecord { name: key, specified_value: None, actual_value: parsed_value, units: from classifier, observation_mode: ObservationMode::Observational }, TemporalCoord { simulation_step: 0, wall_clock_ns: None, logical_sequence: monotonic_counter }, ProvenanceAnchor { source_file: "input.mdp", source_location: SourceLocation::LineRange { start: line_num, end: line_num }, raw_hash: 0 }, dag_node_ref: Some(key.to_string()).
   - Track line numbers (1-indexed) for provenance.

4. **Implement the LOG parser.**
   - Create `pub fn parse_log(content: &str, seq_offset: u64) -> Result<Vec<TraceEvent>, AdapterError>`.
   - Parse three kinds of content by scanning lines sequentially, tracking a mutable `logical_sequence` counter starting at `seq_offset + 1`:

   **Header (first occurrence):**
   - Detect a line containing "GROMACS" — extract version string. Detect lines containing "GPU" or "CPU" for hardware info.
   - Emit one ResourceStatus event: platform_type from hardware detection ("GPU" or "CPU"), device_ids containing the version string, memory_allocated: None, memory_peak: None, parallelization: None, warnings: vec![]. Layer: Implementation. ProvenanceAnchor: LineRange of the header lines.

   **Energy blocks:**
   - Trigger on a line containing `"Energies (kJ/mol)"`.
   - Read the next line as headers, the line after as values. Headers and values are paired positionally — split both on whitespace, but handle multi-word headers by detecting that the value line has fewer tokens (all floats) than the header line. Strategy: count the number of float tokens in the value line, then greedily group header tokens from right to left to match that count. Fallback: pre-process known multi-word headers (e.g., replace "Kinetic En." -> "Kinetic_En.", "Total Energy" -> "Total_Energy") before splitting.
   - Find "Total Energy" (or "Total_Energy") in the parsed pairs — use as EnergyRecord.total (Value::Known(f64, "kJ/mol".to_string())). All other pairs become components (Vec<(String, Value)>).
   - Check all float values for NaN/inf (`value.is_nan() || value.is_infinite()`) — if found, also emit a NumericalStatus event with event_type: NumericalEventType::NaNDetected (or InfDetected), severity: Severity::Warning, affected_quantity: the component name, detail: Value::KnownCat("NaN detected in energy component".to_string()).
   - Get the simulation step from the most recent line matching `"   Step "` or `"Step           Time"` pattern — parse the step number. If no step line found yet, use step 0.
   - EnergyRecord events: Layer Implementation, dag_node_ref: None. Causal refs are wired in Step 5d (not here).

   **Completion:**
   - Scan for `"Finished mdrun"` -> ExecutionStatus { status: ExecutionOutcome::Success, framework_error_id: None }.
   - Scan for `"Fatal error"` -> ExecutionStatus { status: ExecutionOutcome::CrashDivergent, framework_error_id: None }.
   - If neither found -> ExecutionStatus { status: ExecutionOutcome::Timeout, framework_error_id: None }. For this case, call `.confidence(ConfidenceMeta { completeness: Completeness::PartiallyInferred { inference_method: "no completion marker in log".to_string() }, field_coverage: 0.5, notes: vec![] })` on the TraceEventBuilder.
   - The completion event: Layer Implementation. Causal refs wired in Step 5d.

5. **Wire parse_trace integration.**
   - In `GromacsAdapter::parse_trace(&self, raw: &str)`:

     a. Split `raw` on section markers. If `"--- MDP ---"` is found, extract MDP content (text between MDP marker and LOG marker or end). If `"--- LOG ---"` is found, extract LOG content. If no markers, try parsing the whole input as MDP (if it contains `=` on any line), otherwise as LOG.

     b. Parse MDP content -> `Vec<TraceEvent>` via `parse_mdp()`. Collect all MDP event IDs into `mdp_event_ids: Vec<EventId>`.

     c. Parse LOG content -> `Vec<TraceEvent>` via `parse_log(log_content, mdp_events.len() as u64)`. Build all MDP events first so their EventIds are available.

     d. **Wire causal references post-parse.** For each LOG EnergyRecord event, set `causal_refs = mdp_event_ids.clone()`. For each ExecutionStatus event, set `causal_refs = vec![last_energy_event_id]` (the EventId of the last EnergyRecord). For each NumericalStatus event, set `causal_refs = vec![triggering_energy_event_id]`. Mutation: `event.causal_refs = ...` (fields are pub).

     e. Build ExperimentSpec: scan MDP events for `ref_t` -> ControlledVariable { id: SpecElementId(1), parameter: "temperature".to_string(), held_value: <from ref_t value> }, `ref_p` -> ControlledVariable { id: SpecElementId(2), parameter: "pressure".to_string(), held_value: <from ref_p value> }. Set preconditions, postconditions, predictions, interventions, dag_refs to empty Vecs. Set provenance to ProvenanceAnchor { source_file: "input.mdp".to_string(), source_location: SourceLocation::ExternalInput, raw_hash: 0 }.

     f. Build ExperimentRef { experiment_id: "gromacs-trace".to_string(), cycle_id: 0, hypothesis_id: "H0-gromacs-adapter".to_string() }.

     g. Assemble all events (MDP then LOG) into LayeredEventLogBuilder via `.add_event()`, build, and return.

6. **Write 22 new tests.**
   - Add `use crate::gromacs_adapter::GromacsAdapter;` and test constants to `src/tests/mod.rs`.
   - Embed realistic GROMACS text snippets as `const` strings. Example MDP:
     ```
     ; GROMACS mdp file
     integrator = md
     dt = 0.002 ; ps
     nsteps = 500000
     coulombtype = PME
     rcoulomb = 1.0
     ref_t = 300
     ref_p = 1.0
     tcoupl = V-rescale
     pcoupl = Parrinello-Rahman
     nstlog = 1000
     ```
   - Example LOG:
     ```
                  :-) GROMACS - gmx mdrun, 2023.3 (-:

     Using 1 GPU

        Step           Time
           0        0.00000

     Energies (kJ/mol)
        Bond          Angle    Proper Dih.          LJ-14     Coulomb-14
     1234.56       2345.67        345.678       456.789       567.890
        LJ (SR)   Coulomb (SR)   Coul. recip.      Potential    Kinetic En.
     -12345.6      -54321.0        1234.56      -45678.9       12345.6
        Total Energy   Pressure (bar)
     -33333.3            1.013

     Finished mdrun on rank 0
     ```

   **Append these tests at the bottom of `src/tests/mod.rs`:**

   **Unit tests (8):**
   - `test_classify_theory_params` — coulombtype, rcoulomb map to Theory layer
   - `test_classify_methodology_params` — integrator, dt, tcoupl map to Methodology layer
   - `test_classify_implementation_params` — nstlog, nstxout map to Implementation layer
   - `test_classify_dual_annotated` — dt returns DualAnnotated { secondary_layer: Implementation }, rcoulomb returns DualAnnotated { secondary_layer: Methodology }
   - `test_classify_unknown_param` — unknown key returns Implementation + ContextDependent
   - `test_parse_mdp_basic` — parse 5 MDP lines, get 5 events with correct names and values
   - `test_parse_mdp_comments_stripped` — inline `;` comments removed from values
   - `test_parse_mdp_empty` — empty string -> Ok(empty vec)

   **Parser tests (8):**
   - `test_parse_mdp_layer_distribution` — parse ~10 params, verify events span all three layers
   - `test_parse_mdp_provenance_lines` — LineRange line numbers match actual input line positions
   - `test_parse_energy_block` — parse a two-row energy table -> correct (name, f64) pairs
   - `test_parse_log_header` — version/hardware line -> ResourceStatus with "GROMACS" in device_ids
   - `test_parse_log_energy_record` — energy block -> EnergyRecord with total + components
   - `test_parse_log_nan_detection` — NaN in energy values -> NumericalStatus event emitted alongside EnergyRecord
   - `test_parse_log_success` — "Finished mdrun" -> ExecutionStatus::Success
   - `test_parse_log_truncated` — no completion marker -> ExecutionStatus::Timeout

   **Integration tests (6):**
   - `test_gromacs_adapter_combined` — parse_trace with both MDP+LOG sections -> LayeredEventLog with events from both
   - `test_gromacs_adapter_mdp_only` — MDP section only -> valid log with parameter events
   - `test_gromacs_adapter_controlled_vars` — ref_t and ref_p in spec.controlled_variables
   - `test_gromacs_overlay_construction` — Parse -> LEL -> CausalOverlay::from_log -> overlay.len() == event count
   - `test_gromacs_overlay_layer_span` — overlay log has events in all three layers (check indexes.by_layer)
   - `test_gromacs_e2e_confounder_detection` — Build GROMACS log with dag_node_refs, construct CausalOverlay, run detect_confounders with GROMACS variable names -> returns meaningful ConfounderCandidate

7. **Update FINDINGS.md.**
   - Append a new investigation log entry at the top of the Investigation Log section (below the horizontal rule after the most recent entry).
   - Format: `### 2026-02-21: Step 9: GROMACS Adapter for Cross-Framework Validation`
   - Sections: Scope, Method, Findings (numbered: which EventKind variants exercised, any type gaps discovered, layer classification tractability, test counts), Implications (for IR generality), Open Threads.
   - Update the Status line to mention Step 9 completion.
   - Update the Accumulated Findings "What We Know" section with the cross-framework generalization result.
   - Update the Prototype Index table to include `gromacs_adapter.rs` with purpose "GROMACS .mdp/.log parser, DslAdapter impl", status "Complete", demonstrated "Cross-framework IR generalization".

END GOAL:
When complete, the following must all be true:
- `cargo test` passes with ~66 tests (44 existing + 22 new), zero failures
- `cargo clippy --all-targets --all-features -- -D warnings` passes with zero warnings
- GROMACS events use the exact same TraceEvent/EventKind types as OpenMM — no type system changes were needed
- CausalOverlay::from_log works on GROMACS-derived LayeredEventLog instances
- detect_confounders (R14) returns meaningful results with GROMACS variable names
- All three layers (Theory/Methodology/Implementation) are populated from real GROMACS .mdp/.log data
- FINDINGS.md has a new Step 9 investigation log entry following the append-only protocol
- The Prototype Index table in FINDINGS.md includes gromacs_adapter.rs

NARROWING:
- Do NOT add new EventKind variants. If GROMACS output doesn't fit existing variants, document it as a finding.
- Do NOT add new crate dependencies (no regex, no nom, no serde_json additions). Hand-written parsers only.
- Do NOT modify any existing type definitions in common.rs, event_kinds.rs, lel.rs, or overlay.rs. The adapter must work with the type system as-is.
- Do NOT modify existing tests in tests/mod.rs — only append new tests at the bottom.
- Do NOT edit or delete previous Investigation Log entries in FINDINGS.md — append only.
- Avoid over-engineering the energy table parser. If the greedy header-grouping strategy proves too complex, use the fallback: pre-process known multi-word headers into single tokens before splitting.
- Stay within prototype scope — this is a research artifact, not production code. Correctness > polish.
- Do NOT create separate test files. All new tests go in the existing `src/tests/mod.rs`.
- Do NOT use `bd edit` — it opens $EDITOR which blocks agents. Use `bd update` with inline flags instead.
```

---

## Review Findings

### Issues Addressed

**Critical (all 3 fixed):**
1. Added `ExperimentSpec.provenance` field specification in Step 5e with explicit ProvenanceAnchor construction
2. Fully specified `ContextDependent` variant with both `default_layer: Layer::Implementation` and `context_note` in Steps 2 and 5 (Instruction 5)
3. Clarified causal reference wiring mechanism in Step 5d: build MDP events first, collect IDs, mutate LOG events' `causal_refs` fields post-construction

**Warnings (all 4 fixed):**
1. Changed `parse_log` signature to include `seq_offset: u64` parameter (Step 4)
2. Added explicit `.confidence()` builder call for the Timeout case (Step 4, Completion)
3. Added `raw_hash: 0` instruction to INSTRUCTIONS principle 4
4. Added concrete `DualAnnotated` rationale strings in INSTRUCTIONS principle 8 and Step 2 classification table

### Remaining Suggestions
- Energy table parser could benefit from a worked example with real column layout (partially addressed by embedding example LOG snippet in Step 6)
- `NumericalStatus.detail` field specified as `Value::KnownCat(...)` in Step 4 — could be more precise with actual NaN value reference
- FINDINGS.md "What We Don't Know" section update left to agent judgment based on actual findings

## Usage Notes

- **Best used with:** Claude Opus 4.6 or Sonnet 4.6 in Claude Code with full codebase access
- **Adjust for:** If running without the existing codebase, provide the full source of adapter.rs, common.rs, event_kinds.rs, lel.rs, and overlay.rs as context
