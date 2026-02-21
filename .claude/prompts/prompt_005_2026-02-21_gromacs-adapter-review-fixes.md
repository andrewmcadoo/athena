# Prompt: GROMACS Adapter Review Followup Fixes

> Generated: 2026-02-21 | Prompt #5 | Framework: RTF

---

## Session Goal

Create a prompt for Codex to address the 3 warnings and the coverage gap (S4) from the GROMACS adapter code review — tightening the Step matcher, handling missing "Total Energy", adding missing multi-word headers to the known list, and adding a CrashDivergent test.

## Framework Selection

- **Chosen:** RTF
- **Rationale:** Small, well-defined set of 4 surgical fixes with clear acceptance criteria. RTF keeps it concise without the overhead of RISEN's step sequencing or TIDD-EC's elaborate constraint structure.
- **Alternatives considered:** TIDD-EC (overkill for 4 targeted fixes), RISEN (too heavyweight for non-sequential patches)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Four fixes with code excerpts anchoring each location |
| Specificity | 9/10 | Before/after code, builder patterns, test skeleton provided |
| Context | 8/10 | Sufficient — self-contained with code anchors, no exploration needed |
| Completeness | 9/10 | All 4 items covered with verification criteria |
| Structure | 9/10 | Clean RTF with numbered fixes and mapping table |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
Rust developer patching the GROMACS adapter in the ATHENA LEL IR prototype. Working directory: research/trace-semantics/prototypes/lel-ir-prototype/

TASK:
Apply 4 targeted fixes to `src/gromacs_adapter.rs` and 1 new test to `src/tests/mod.rs` based on code review feedback. No new files. No type changes. No new dependencies. Minimal diffs.

| Review Item | Fix | File |
|---|---|---|
| W1: Step matcher over-matches | Fix 1 | gromacs_adapter.rs |
| W2: Missing Total Energy silently skipped | Fix 2 | gromacs_adapter.rs |
| W3: Multi-word headers not in known list | Fix 3 | gromacs_adapter.rs |
| S4: No test for CrashDivergent path | Fix 4 | tests/mod.rs |

**Fix 1 (W1): Tighten Step line matcher**
Location: the `while idx < lines.len()` loop, find this line:
```rust
if line.contains("Step") {
```
Change to:
```rust
if line.trim_start().starts_with("Step") {
```
This prevents false matches on lines where "Step" appears mid-text. The `parse_step_from_line` helper is fine and needs no changes — it uses `split_whitespace().find_map()` which only extracts the first parseable u64.

**Fix 2 (W2): Warn on missing "Total Energy" instead of silently skipping**
Location: find this block:
```rust
let Some(total) = total_energy else {
    idx = row_idx;
    continue;
};
```
Replace with code that emits a diagnostic NumericalStatus event before continuing. Use the same TraceEventBuilder pattern as the NaN/Inf detection nearby:
```rust
let Some(total) = total_energy else {
    let warning_event = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::NumericalStatus {
            event_type: NumericalEventType::ConvergenceFailure,
            affected_quantity: "Total Energy".to_string(),
            severity: Severity::Warning,
            detail: Value::KnownCat(
                "Energy block parsed but Total Energy header not found".to_string(),
            ),
        })
        .temporal(TemporalCoord {
            simulation_step: current_step,
            wall_clock_ns: None,
            logical_sequence: seq,
        })
        .provenance(ProvenanceAnchor {
            source_file: "simulation.log".to_string(),
            source_location: SourceLocation::LineRange {
                start: (idx + 1) as u32,
                end: (idx + 1) as u32,
            },
            raw_hash: 0,
        })
        .build();
    seq += 1;
    events.push(warning_event);
    idx = row_idx;
    continue;
};
```

**Fix 3 (W3): Add missing multi-word headers to known list**
Location: find the `known` array in `tokenize_energy_headers`:
```rust
let known = [
    ("Kinetic En.", "Kinetic_En."),
    ("Total Energy", "Total_Energy"),
    ("Pressure (bar)", "Pressure_(bar)"),
    ("Coulomb (SR)", "Coulomb_(SR)"),
    ("Coul. recip.", "Coul._recip."),
];
```
Add these 6 entries:
```rust
let known = [
    ("Kinetic En.", "Kinetic_En."),
    ("Total Energy", "Total_Energy"),
    ("Pressure (bar)", "Pressure_(bar)"),
    ("Coulomb (SR)", "Coulomb_(SR)"),
    ("Coul. recip.", "Coul._recip."),
    ("LJ (SR)", "LJ_(SR)"),
    ("Proper Dih.", "Proper_Dih."),
    ("Improper Dih.", "Improper_Dih."),
    ("LJ (LR)", "LJ_(LR)"),
    ("Coulomb (LR)", "Coulomb_(LR)"),
    ("Disper. corr.", "Disper._corr."),
];
```

**Fix 4 (S4): Add test for CrashDivergent path**
Append to `src/tests/mod.rs`. Create a new test constant and test function following the pattern of the existing `test_parse_log_success` and `test_parse_log_truncated` tests:
```rust
const GROMACS_LOG_FATAL_ERROR: &str = "\
             :-) GROMACS - gmx mdrun, 2023.3 (-:

Using 1 GPU

   Step           Time
      0        0.00000

Energies (kJ/mol)
   Kinetic En.   Total Energy
      1234.56       -5678.90

Fatal error: Step 100: The total potential energy is -1e+14
";

#[test]
fn test_parse_log_fatal_error() {
    setup();
    let events = crate::gromacs_adapter::parse_log(GROMACS_LOG_FATAL_ERROR, 0).unwrap();
    let last = events.last().expect("Expected at least one event");
    match &last.kind {
        EventKind::ExecutionStatus {
            status: ExecutionOutcome::CrashDivergent,
            ..
        } => {}
        other => panic!("Expected CrashDivergent, got {:?}", other),
    }
}
```

FORMAT:
- Edit only `src/gromacs_adapter.rs` and `src/tests/mod.rs`. No other files.
- Expected test count after all fixes: 67 (66 existing + 1 new CrashDivergent test). The 3 code fixes do not add tests.
- Verify: `cargo test` (67 pass, 0 fail), `cargo clippy --all-targets --all-features -- -D warnings` (0 warnings).
- All paths relative to working directory: research/trace-semantics/prototypes/lel-ir-prototype/
```

---

## Review Findings

### Issues Addressed
- Added mapping table linking review items to fixes (reviewer W: contradictory mapping)
- Anchored all fixes with code excerpts instead of fragile line numbers (reviewer W: line number drift)
- Expanded Fix 2 with full TraceEventBuilder pattern including temporal/provenance (reviewer W: incomplete construction spec)
- Specified test constant name and full test skeleton for Fix 4 (reviewer W: vague test quality)
- Added expected test count (67) to FORMAT section (reviewer W: missing quality gate)

### Remaining Suggestions
- Could add before/after for Fix 1 in fuller context (single-line change, low ambiguity)
- Working directory repeated in ROLE and FORMAT for redundancy (reviewer S: thin component)

## Usage Notes

- **Best used with:** Codex or Claude Code with codebase access
- **Adjust for:** If test_parse_log_fatal_error triggers a "Step" false match (due to "Fatal error: Step 100" in the test constant), Fix 1 must land first — the `starts_with("Step")` tightening prevents this. Apply fixes in order 1-4.
