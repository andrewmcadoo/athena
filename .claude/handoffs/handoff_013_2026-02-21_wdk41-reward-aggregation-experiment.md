# Session Handoff: WDK#41 Reward Aggregation Experiment Design

> Generated: 2026-02-21 | Handoff #13 | Previous: handoff_012_2026-02-21_organize-findings-doc.md

---

## Continuation Directive

Plan an experiment to answer WDK#41: **How do you collapse a multi-metric comparison into one reward number?** Specifically, design a research experiment that determines how to standardize profile aggregation (from `ComparisonProfileV1.metrics: Vec<MetricComponent>`) into a bounded, monotonic reward scalar that remains calibratable under ARCHITECTURE.md §5.4 feedback and robust against Noisy-TV style reward hacking. This bridges the trace-semantics IR (Priority 1, largely complete) to the adversarial-reward track (Priority 2, not yet started).

## Task Definition

**Project:** ATHENA — falsification-driven AI co-scientist for structured simulation frameworks (OpenMM, GROMACS, VASP).

**Goal:** Formalize the aggregation function that collapses a vector of typed divergence metrics (each with uncertainty, sample size, provenance) into one scalar the adversarial agent can optimize. The scalar must be: (1) bounded, (2) monotonic (higher = more contradiction evidence), (3) calibratable against predicted-vs-actual surprise under §5.4, (4) resistant to Noisy-TV degeneration.

**Why it matters:** This is the handoff point between two research tracks. Trace-semantics produces `ComparisonProfileV1` with multi-component metrics. The adversarial-reward track needs a single scalar to rank candidate experiments. Without this aggregation, the adversarial agent has no objective function.

**Constraints (non-negotiable):** DSL-only environments, warm-started causal priors, bounded adversarial design (ARCHITECTURE.md §3.1). See `CLAUDE.md` for full constraints.

## Key Decisions & Rationale

1. **Candidate B (Multi-Metric Divergence Profile) is the recommended comparison formalization**
   - **Rationale:** Scores best across 5 evaluation axes (adversarial reward, BSE compatibility, IR simplicity, Stage 2->3 tractability, adapter burden). Provides scalar optimization + component-level calibration controls.
   - **Alternatives rejected:** Candidate A (Typed Scalar Divergence) — too weak for calibration/BSE; Candidate C (Distribution-Aware Posterior) — too expensive for adapters that can't emit full posteriors.
   - **Evidence:** Step 12 log in `research/trace-semantics/FINDINGS.md` (line ~670)

2. **ComparisonProfileV1 interface contract defined with 5 guarantees**
   - G1: Determinism (identical inputs -> identical outputs)
   - G2: Monotonicity declaration (higher = more contradiction evidence)
   - G3: Validity gating (LFI can invalidate implementation artifacts)
   - G4: Auditability (trace/spec/DAG provenance per metric)
   - G5: Partial distribution support (optional uncertainty, explicit omission)
   - **Evidence:** Step 12 contract block at lines ~732-759

3. **Aggregation function is explicitly novel research** — Step 12 finding #9 states: "Novel research still required: (i) canonical aggregation from profile -> bounded reward scalar under Noisy-TV constraints."

## Current State

### Completed
- **Trace-semantics research track:** All 14 steps and 3 synthesis steps complete. Prototype: 92/92 tests, 3 adapters (OpenMM, GROMACS, VASP), full query surface (R14+R17+R18). IR design question answered: Hybrid LEL+DGR.
- **FINDINGS.md reorganization:** TOC, index table, thematic What We Know (5 groups, 75 items), separated resolved WDK items (17 resolved, 27 open). File is now navigable.
- **Bead `athena-apb` created:** Tracks WDK#41 as P1 feature issue.

### In Progress
- Nothing actively in progress. The adversarial-reward FINDINGS.md is at "NOT STARTED" status.

### Blocked / Open Questions
- WDK#41 is the specific blocker. Adjacent open questions that may inform the experiment:
  - WDK#42: Convergence pattern taxonomy (what patterns to aggregate over)
  - WDK#43: GROMACS/OpenMM convergence summary derivation
  - WDK#44: Where to compute convergence summaries in the pipeline
- The adversarial-reward track has its own next steps (survey active learning/BED, characterize failure spectrum, analyze Noisy TV in DSL contexts) — WDK#41 connects to steps 4 and 5 of that track.

## Key Code Context

**`ComparisonProfileV1` contract** (from Step 12 log, lines ~732-751) — this is the input the aggregation function must consume:
```text
ComparisonProfileV1 {
  comparison_event_id: EventId
  prediction_id: SpecElementId
  dag_node: Option<String>
  metrics: Vec<MetricComponent>
  aggregate: AggregateScore        // <-- THIS IS WDK#41
  reward_validity: RewardValidity
  provenance: ProvenanceAnchor
}

MetricComponent {
  kind: DivergenceKind   // AbsoluteDifference|ZScore|BayesFactor|KLDivergence|EffectSize|Custom
  value: f64
  direction: Option<EffectDirection>
  uncertainty: Option<UncertaintySummary>
  sample_size: Option<u32>
  units: Option<Unit>
  method_ref: String
}
```

The `AggregateScore` field is the undetermined output — WDK#41 is defining what goes here and how it's computed from the `metrics` vector.

## Files Map

| Path | Role/Purpose | Status |
|------|-------------|--------|
| `research/trace-semantics/FINDINGS.md` | Master research log (1939 lines, reorganized) | Modified this session |
| `research/adversarial-reward/FINDINGS.md` | Priority 2 research log — NOT STARTED | Read, not modified |
| `ARCHITECTURE.md` §4.4, §5.4 | Adversarial Experiment Designer + Calibration Feedback | Reference |
| `VISION.md` §4.2, §6.2 | Bounded adversarial design + Noisy TV problem | Reference |
| `research/trace-semantics/prototypes/lel-ir-prototype/` | Working Rust prototype (92/92 tests) | Reference |

## Loop State

N/A — no Claude->Codex loop. This is a research planning task.

## Next Steps

1. **Read `research/adversarial-reward/FINDINGS.md`** — understand the research question and planned next steps for that track
2. **Read Step 12 log** (trace-semantics FINDINGS.md lines ~670-774) — the ComparisonProfileV1 contract and candidate evaluation are the input specification
3. **Read ARCHITECTURE.md §5.4** (Adversarial Calibration Feedback) — the calibration loop constrains what aggregation functions are valid
4. **Read VISION.md §6.2** (Noisy TV Problem) — the failure mode the aggregation must resist
5. **Design the experiment** — define: (a) candidate aggregation functions, (b) synthetic scenarios to stress-test them, (c) evaluation criteria, (d) what "success" looks like
6. **Write findings to `research/adversarial-reward/FINDINGS.md`** as the first investigation log entry for that track

## Session Artifacts

- Bead created: `athena-apb` (WDK#41, P1 feature)
- Previous handoff: `.claude/handoffs/handoff_012_2026-02-21_organize-findings-doc.md`
- FINDINGS.md reorganization: +60 lines (TOC, index table, thematic groups, resolved section, status note)

## Documentation Updated

No documentation updates — all project docs were current.
