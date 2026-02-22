# Prompt: Hybrid Robustness Under Fixture Perturbation

> Generated: 2026-02-22 | Framework: RISEN

---

## Session Goal

Implement a perturbation robustness test for the hybrid aggregation candidate (HTG gating + Fisher product) that systematically varies fixture parameters across all 7 scenarios, producing a robustness map with pass/fail regions, margins, and tipping points — to determine whether Session 3's 7/7 result is structurally robust or parameter-tuned.

## Framework Selection

- **Chosen:** RISEN (Role, Instructions, Steps, End Goal, Narrowing)
- **Rationale:** Complex multi-step implementation task with clear methodology (systematic perturbation), sequential steps, and explicit constraints. RISEN excels at encoding process-heavy tasks with methodology.
- **Alternatives considered:** TIDD-EC (good for dos/don'ts but weaker on sequential process), Chain of Thought (better for reasoning tasks than implementation)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal is unambiguous — robustness map of hybrid aggregator under perturbation |
| Specificity | 10/10 | Exact sweep values, margin formulas, data structures, file paths all specified |
| Context | 9/10 | Session 3 results, architecture references, and fragility hypotheses provided |
| Completeness | 9/10 | Covers implementation, output, analysis, and documentation update |
| Structure | 10/10 | 8 sequential steps with clear dependencies, tables for sweep parameters |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

ROLE:
You are an expert Python developer implementing a perturbation robustness test for the ATHENA adversarial reward research project. You have deep knowledge of: statistical aggregation methods, fixture-based testing patterns, and structured scientific output. You work within an existing prototype codebase at `research/adversarial-reward/prototypes/aggregation-candidates/`.

INSTRUCTIONS:
- Import from existing modules — do not duplicate or rewrite existing logic
- The hybrid aggregation candidate (HTG gating + Fisher product, n_terms=1) achieved 7/7 on all fixed scenarios in Session 3. This test determines whether that result is structurally robust or parameter-tuned.
- Fragility hypotheses: S2 has a +7.28% margin driven by a single custom sigmoid component. S5 has component `s5.bf.1` with normalized score ~0.990 against the 0.991 ceiling (margin 0.001). These are the most likely failure points.
- This is prototype/research code — optimize for clarity and correctness, not production abstractions
- Use private helpers (`_metric()`, `_summary()`, `_no_uncertainty()`) from `scenarios.py` — acceptable for same-directory prototype code
- All sweep runs use the baseline HybridConfig. The only axis that requires constructing a variant HybridConfig is S2 custom sigmoid perturbation, which modifies `NormalizationConfig.custom_sigmoids` within a new HybridConfig instance. All other axes modify the fixture while keeping the config fixed.

STEPS:
1. Create `perturbation_test.py` with skeleton, imports, and baseline sanity check.
   - Import from: `candidates` (HybridConfig, aggregate_hybrid), `evaluate` (evaluate_fixture, ScenarioCellResult), `scenarios` (ScenarioFixture, build_scenario_fixtures, DEFAULT_CUSTOM_SIGMOIDS, _metric, _summary, _no_uncertainty), `normalization` (NormalizationConfig), `models` (MetricComponent, DivergenceKind, EffectDirection, NoUncertainty, Summary, UncertaintySummary, SigmoidParams)
   - Define baseline HybridConfig constant with `normalization=NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)`
   - Run baseline `evaluate_fixture` for S2 to verify +7.28% margin as sanity check

2. Define data structures:
   @dataclass(frozen=True)
   class PerturbationResult:
       scenario_idx: int
       axis: str           # e.g. "s2_custom_sigmoid"
       label: str          # e.g. "k=2.0,x0=-0.2"
       passed: bool
       margin: float       # distance from failure boundary (negative = fail)
       margin_label: str   # e.g. "ratio/1.5-1"
       raw_scores: dict[str, float]
       is_baseline: bool

   Margin definitions per scenario (positive = pass in all cases):
   - S1: `base - doubled` 
   - S2: `(aggregate / max_single) / 1.5 - 1.0` (current +0.0728)
   - S3: `min(mixed - lo, hi - mixed)` where lo/hi are agreement/contradiction bounds
   - S4: `0.20 - relative_delta`
   - S5: `min(min(component_scores) - 0.3, 0.991 - max(component_scores))` (tracks both lower and upper bound; upper is binding at 0.001 via component s5.bf.1)
   - S6: `min(dominant_share - 0.35, 1e-8 - abs(recon - aggregate))` (tracks both dominance and reconstruction sub-criteria)
   - S7: `non_boundary - boundary`

3. Build fixture variant builders. Each constructs a fresh ScenarioFixture with modified parameters:
   - `build_s2_custom_sigmoid_config(k, x0) -> HybridConfig` — modifies custom_sigmoids dict, preserves s6.custom.1
   - `build_s2_se_scaled_fixture(se_mult) -> ScenarioFixture` — scales non-custom component SEs
   - `build_s5_bf_variant(bf_value) -> ScenarioFixture` — replaces BF component value
   - `build_s6_compress_variant(d_mid_value, bf_strong_value) -> ScenarioFixture` — compresses dominant gap
   - `build_s7_boundary_se_variant(boundary_se) -> ScenarioFixture` — varies z-component SE in boundary dataset
   - `build_s4_missing_count_variant(n_missing) -> ScenarioFixture` — varies how many components lack uncertainty
   - `build_s1_se_variant(se_mult) -> ScenarioFixture` — scales both base and doubled SEs
   Key subtlety: S2 custom sigmoid perturbation modifies NormalizationConfig.custom_sigmoids within a new HybridConfig instance. The builder returns a modified HybridConfig rather than a modified fixture. All other axes modify the fixture while keeping the baseline config fixed.

4. Implement sweep driver (~78 evaluation runs, sub-second total). Organized by priority:

   | Axis | Scenario | Sweep points | Expected failures |
   |------|----------|-------------|-------------------|
   | Custom sigmoid (k x x0) | S2 | 24 (6k x 4x0) | ~4 (x0=-0.2 with k>=2.0) |
   | Non-custom SE scale | S2 | 5 | 0 |
   | BayesFactor value | S5 | 9 | ~5 (BF>=120) |
   | Boundary SE | S7 | 7 | 0 (monotone in SE) |
   | Joint compression (d_mid x bf_strong) | S6 | 16 (4x4) | ~4 (joint attack) |
   | Missing count | S4 | 4 | 0 |
   | SE multiplier | S1 | 5 | 0 |

   Sweep values:
   - S2 custom: k in {1.0, 1.5, 2.0, 2.2, 2.5, 3.0}, x0 in {-0.2, 0.0, 0.2, 0.5}
   - S2 SE scale: mult in {0.5, 0.75, 1.0, 1.5, 2.0}
   - S5 BF: {80, 90, 100, 110, 120, 150, 200, 500, 1000}
   - S7 boundary SE: {0.25, 0.30, 0.40, 0.50, 0.70, 0.90, 1.20}
   - S6 d_mid: {0.9, 2.0, 3.0, 4.0}, bf_strong: {12.0, 100, 500, 1000}
   - S4 missing: {1, 2, 3, 4} components without uncertainty
   - S1 SE mult: {1.0, 1.5, 3.0, 5.0, 10.0}

5. Write output files:
   - `perturbation_results.json` — full structured results per axis, including all margins and raw scores
   - `perturbation_summary.md` — compact tables:
     (a) Top-level verdict: scenario x axis -> pass rate
     (b) Per-critical-axis margin grids (S2 sigmoid 6x4 table, S5 BF list)
     (c) Identified tipping points (exact parameter values where PASS flips to FAIL)

6. Execute `python perturbation_test.py` and review output for:
   - Which scenarios have fragile regions
   - How tight the margins are at boundary
   - Whether any "expected robust" scenarios surprise us

7. Update `research/adversarial-reward/FINDINGS.md`:
   - Append Session 4 investigation log entry at the TOP of the Investigation Log (reverse-chronological)
   - Include: Scope, Method, Findings (robustness map), Implications, Open Threads
   - Add perturbation_test.py to the Prototype Index table
   - Update Accumulated Findings (What We Know / Suspect / Don't Know)

8. Close bead and run session protocol:
   - `bd close athena-bop`
   - `bd sync --from-main`
   - `git add` relevant files + `git commit`

END GOAL:
- `perturbation_test.py` runs without errors and produces `perturbation_results.json` + `perturbation_summary.md`
- Baseline points match Session 3 results (S2 margin = +7.28%, S4 delta = 0.072)
- Tipping points identified for S2, S5, S6 with exact parameter boundaries
- Robustness map reveals whether 7/7 is structural or fragile
- FINDINGS.md Session 4 entry properly formatted (reverse-chronological, evidence-cited)

NARROWING:
- Do NOT modify existing modules (candidates.py, evaluate.py, scenarios.py, normalization.py, models.py) — only read/import from them
- Do NOT add production abstractions — this is throwaway prototype code
- Do NOT skip the baseline sanity check — it validates the test harness against known Session 3 values
- Do NOT fabricate expected results — record what the sweep actually produces
- Stay within the existing prototype directory (`research/adversarial-reward/prototypes/aggregation-candidates/`)
- Constraints: S3 direction asymmetry axis is lowest priority and may be dropped if the other axes reveal sufficient signal. If dropped, adjust the ~78 run count accordingly.
- Out of scope: modifying the hybrid config defaults, changing the evaluation logic, or proposing alternative aggregation methods

---

## Review Findings

### Issues Addressed
1. **[WARNING] S6 margin** — Added reconstruction error sub-criterion: `min(dominant_share - 0.35, 1e-8 - abs(recon - aggregate))`
2. **[WARNING] S5 margin** — Added lower bound tracking: `min(min(scores) - 0.3, 0.991 - max(scores))`, named binding component `s5.bf.1`
3. **[WARNING] HybridConfig ambiguity** — Reworded to "baseline HybridConfig" with explicit note that S2 constructs a new instance
4. **[WARNING] S5 binding component unnamed** — Added `s5.bf.1` identification
5. **[WARNING] Sweep values** — Confirmed present in full prompt (reviewer received abbreviated version)
6. **[SUGGESTION] SigmoidParams import** — Moved to `from models import ...`

### Remaining Suggestions
- S4 precision: 0.072 is rounded from 0.0719926...; consider noting tolerance
- S3 axis: deprioritized in NARROWING, not fully specified; run count notes "adjust accordingly"
- Step 8 bead protocol: project-specific workflow step, opaque to external readers but clear to target user

## Usage Notes

- **Best used with:** Claude Opus or Sonnet with access to the ATHENA codebase; requires reading existing modules before implementing
- **Adjust for:** If S3 axis is included, add its builder and sweep points to Steps 3-4
