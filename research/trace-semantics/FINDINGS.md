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

NOT STARTED

## Key Definitions

- **Trace log**: Raw output from DSL framework execution — timestamped events, state transitions, parameter values, errors, and warnings produced by the simulation engine.
- **Semantic IR**: Structured intermediate representation that maps trace log events to a causal narrative distinguishing theory-layer operations (parameter choices, equation evaluations) from implementation-layer operations (memory allocation, data loading, numerical execution).
- **Fault classification boundary**: The minimum IR resolution at which the LFI's three-stage audit (implementation -> methodology -> theory) can produce determinate classifications rather than ambiguous ones.
- **Theory-implementation separation**: The API-enforced structural distinction in DSL frameworks between what the user specifies (theory) and how the framework executes it (implementation).

## Investigation Log

*No entries yet.*

## Accumulated Findings

### What We Know

*No findings yet.*

### What We Suspect

*No findings yet.*

### What We Don't Know

*No findings yet.*

## Prototype Index

| Filename | Purpose | Status | Demonstrated |
| :--- | :--- | :--- | :--- |
| *None yet* | | | |

## Next Steps

1. **Survey DSL trace formats** — Collect and document the actual trace output structure from OpenMM, GROMACS, and VASP. Identify common elements (timestamped events, state dumps, error codes) and framework-specific elements. Scope: 2-3 sessions.

2. **Survey existing IR designs in RCA and formal verification** — Review intermediate representations used in root cause analysis tools, formal verification (SAT solvers, model checkers), and program analysis. Identify which design patterns transfer to DSL trace parsing. Scope: 2-3 sessions.

3. **Map LFI three-stage audit backwards to minimum IR requirements** — Starting from the three audit stages in ARCHITECTURE.md 5.3, derive the minimum set of semantic distinctions the IR must represent. What must Stage 1 (implementation audit) be able to query? Stage 2 (methodology audit)? Stage 3 (theory evaluation)? Scope: 1-2 sessions.

4. **Characterize the 21% baseline and DSL improvement** — Understand what drives the low accuracy of general RCA on unstructured traces, and what structural properties of DSL environments improve it. Identify the specific features of DSL traces that make them more amenable to semantic parsing. Scope: 1-2 sessions.

5. **Draft candidate IR schemas** — Based on the above, propose 2-3 candidate IR designs with explicit tradeoffs (resolution vs. generality vs. implementation cost). Each should be evaluated against the minimum requirements from step 3. Scope: 2-3 sessions.
