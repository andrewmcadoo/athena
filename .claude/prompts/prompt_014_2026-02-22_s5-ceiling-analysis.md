# Prompt: S5 Ceiling Analysis Implementation

> Generated: 2026-02-22T00:00:00Z | Prompt #1 | Framework: RISEN

---

## Session Goal

Create a structured prompt that instructs an AI to implement a ceiling analysis for ATHENA's hybrid aggregation candidate, specifically: analyzing the BF/(BF+1) normalization saturation at BF=111, testing 3 alternative normalization functions (log-scaled, power-law, exp-decay), validating S6 decomposition independence, and running full 7-scenario evaluation suites — all as a new Python analysis script with JSON/markdown outputs and a FINDINGS.md log entry.

## Framework Selection

- **Chosen:** RISEN (Role, Instructions, Steps, End Goal, Narrowing)
- **Rationale:** Complex multi-step implementation task with 4 sequential phases, specific technical constraints, and well-defined deliverables. RISEN excels at encoding sequential processes with role context and explicit boundaries.
- **Alternatives considered:** TIDD-EC (good for precision constraints but weaker at sequential flow), Chain of Thought (useful for reasoning tasks but steps are pre-designed here)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Precise technical goal with exact formulas, thresholds, and file paths |
| Specificity | 9/10 | Exact BF values, normalization formulas, calibration strategy, pass criteria all specified |
| Context | 9/10 | Full codebase structure, existing module APIs, and failure analysis provided |
| Completeness | 9/10 | 4-phase analysis design with sanity gates, pre-filters, output format, and verification |
| Structure | 9/10 | RISEN framework with clear sequential phases and narrowing constraints |
| **Overall** | **9/10** | |

---

## Structured Prompt

ROLE:
You are an expert computational scientist implementing a targeted analysis script within an existing research prototype codebase. You have deep knowledge of: Bayesian model comparison (Bayes factors), Fisher's method for combining p-values, normalization theory for bounded score functions, and Python scientific computing. You understand the ATHENA adversarial-reward aggregation pipeline — its `normalize_component` -> gating -> Fisher product architecture — and can write code that precisely reuses existing module APIs without duplication.

INSTRUCTIONS:

Implement `ceiling_analysis.py` as a new file at `research/adversarial-reward/prototypes/aggregation-candidates/ceiling_analysis.py`. This script performs a 4-phase analysis of the BF/(BF+1) normalization ceiling that limits the hybrid aggregation candidate's S5 scenario at BF=111. The analysis tests 3 alternative normalization functions, validates S6 failure independence, runs full 7-scenario evaluation suites, and produces structured JSON + markdown outputs.

Governing principles:
- **Reuse over rewrite.** Import and call existing functions from `candidates.py`, `evaluate.py`, `scenarios.py`, `perturbation_test.py`, `normalization.py`, and `models.py`. Never duplicate logic that exists, except where noted in Step 4 for the BF-branch post-processing (where `normalize_component` does not expose a hook point).
- **Sanity-gate-first.** Before testing any alternative normalization, verify that the patched hybrid with the current BF norm (`1 - 1/(1+BF)`) produces results matching the baseline `aggregate_hybrid()` within `1e-12` tolerance across all 7 scenarios. Abort with a clear error message if this fails.
- **Calibration-driven parameter selection.** For each normalization family, set the free parameter so that `f(BF_max) = 0.991` for each target ceiling. This directly answers "how far does each normalization extend the operating range?"
- **Pre-filter before expensive evaluation.** Only advance candidates whose raw normalization output at BF=100 is >= 0.3 to Phase 3 (this is a loose sanity check on the raw normalization output, not a proxy for the full S5 criterion).
- **Deterministic, reproducible.** No randomness. All outputs timestamped UTC. JSON output is machine-parseable; markdown output is human-readable.

STEPS:

1. **Set up imports and constants.**
   Import from existing modules:
   - `evaluate_fixture`, `ScenarioCellResult` from `evaluate.py`
   - `build_scenario_fixtures`, `_metric`, `DEFAULT_CUSTOM_SIGMOIDS` from `scenarios.py`
   - `build_s5_bf_variant`, `build_s6_compress_variant`, `BASELINE_HYBRID_CONFIG`, `S5_BF_VALUES`, `S6_D_MID`, `S6_BF_STRONG` from `perturbation_test.py`
   - `bounded_unit_interval`, `gate_precision`, `sigmoid`, `direction_is_agreement`, `normalize_component`, `extract_uncertainty_snapshot`, `se_dampen`, `direction_is_none_variant`, `_direction_value`, `NormalizationConfig` from `normalization.py`
   - `chi_square_cdf_even_df`, `HybridConfig`, `aggregate_hybrid` from `candidates.py`
   - `DivergenceKind`, `AggregateResult`, `ComponentContribution`, `MetricComponent` from `models.py`

   Note: `BASELINE_HYBRID_CONFIG` from `perturbation_test.py` already encapsulates `DEFAULT_CUSTOM_SIGMOIDS` in its normalization config.

   Define constants:
   - `BF_MAX_TARGETS = [200, 500, 1000, 5000, 10000]`
   - `BF_CURVE_RANGE = range(1, 10001)` (for curve computation)
   - `S5_SWEEP_BF = [80.0, 100.0, 120.0, 200.0, 500.0, 1000.0]` (floats, for Phase 3 S5 sweep)
   - `S6_FAILING_CELLS = [(3.0, 500.0), (3.0, 1000.0), (4.0, 100.0), (4.0, 500.0), (4.0, 1000.0)]`
   - `SCORE_AT_100_FLOOR = 0.3` (pre-filter threshold)
   - `SANITY_TOL = 1e-12` (sanity gate tolerance)

2. **Define dataclasses for structured results.**
   Create frozen dataclasses:
   - `BFNormCandidate`: `name: str`, `family: str`, `bf_max_target: int`, `free_param_name: str`, `free_param_value: float`, `norm_fn: Callable[[float], float]`
   - `CeilingProbeResult`: `candidate_name: str`, `bf_max_target: int`, `bf_ceiling: float`, `score_at_100: float`, `score_at_500: float`, `score_at_1000: float`, `passes_prefilter: bool`
   - `FullSuiteResult`: `candidate_name: str`, `bf_max_target: int`, `baseline_7_of_7: bool`, `per_scenario: list[dict]`, `s5_sweep: list[dict]`
   - `S6DecompositionCheck`: `d_mid: float`, `bf_strong: float`, `recon_error: float`, `dominant_share: float`, `failure_is_dominant_share: bool`, `failure_is_recon_error: bool`

3. **Implement BF normalization functions and calibration.**

   Reference normalization (must match `normalization.py` line 138 exactly):
   ```python
   def bf_norm_current(bf: float) -> float:
       return 1.0 - 1.0 / (1.0 + bf)
   ```

   Three alternative families:
   ```python
   def bf_norm_log_scaled(bf: float, c: float) -> float:
       log_term = math.log1p(bf)
       return log_term / (log_term + c)

   def bf_norm_power_law(bf: float, alpha: float) -> float:
       return 1.0 - 1.0 / (1.0 + bf) ** alpha  # alpha < 1 for slower saturation

   def bf_norm_exp_decay(bf: float, k: float) -> float:
       return 1.0 - math.exp(-bf / k)
   ```

   Calibration: For each family, solve for the free parameter such that `f(BF_max) = 0.991`:
   - **Log-scaled**: `c = log(1+BF_max) * (1 - 0.991) / 0.991` -> `c = log(1+BF_max) * 0.009 / 0.991`
   - **Power-law**: `alpha = log(1 - 0.991) / log(1/(1+BF_max))` -> `alpha = log(0.009) / log(1/(1+BF_max))`
   - **Exp-decay**: `k = -BF_max / log(1 - 0.991)` -> `k = -BF_max / log(0.009)`

   Build a `list[BFNormCandidate]` for each `(family, bf_max_target)` combination using `functools.partial` to bind the calibrated parameter.

4. **Implement `normalize_component_with_alt_bf`.**

   This function replaces only the BayesFactor branch of `normalize_component`. For non-BF kinds, it delegates entirely to the existing `normalize_component`. For BF kinds, it computes `raw_score = bf_norm_fn(bf)` then applies the same post-processing chain (direction handling, bounding, agreement inversion, SE dampening).

   **Acknowledged tradeoff**: The BF-branch post-processing (~15 lines of direction/bounding/SE logic) is duplicated from `normalize_component` because that function does not expose a hook point for substituting only the raw score computation. This is accepted as a pragmatic concession — the duplication is confined to one function and documented.

   ```python
   def normalize_component_with_alt_bf(component, config, bf_norm_fn):
       if component.kind is not DivergenceKind.BayesFactor:
           return normalize_component(component, config)  # delegate unchanged

       # BF-specific branch (mirrors normalization.py lines 126-184 for BF path only)
       direction_value = _direction_value(component.direction)
       if component.direction is None or direction_is_none_variant(component.direction):
           transformed_value = abs(component.value)
           direction_mode = "unsigned"
       else:
           transformed_value = component.value
           direction_mode = direction_value or "unset"

       bf = max(transformed_value, 0.0)
       raw_score = bf_norm_fn(bf)  # <-- the substitution point

       bounded_raw = bounded_unit_interval(raw_score, config.clip_eps)
       if direction_is_agreement(component.direction):
           adjusted_score = 1.0 - bounded_raw
       else:
           adjusted_score = bounded_raw

       final_score = bounded_unit_interval(adjusted_score, config.clip_eps)
       if config.se_dampen_enabled:
           snapshot = extract_uncertainty_snapshot(component)
           if snapshot.standard_error is not None and snapshot.standard_error > 0:
               final_score = se_dampen(final_score, component.value, snapshot.standard_error, config)

       return final_score, [], {
           "raw_score": bounded_raw,
           "direction_mode": direction_mode,
           "transformed_value": transformed_value,
       }
   ```

5. **Implement `aggregate_hybrid_patched`.**

   This is a copy of `aggregate_hybrid` from `candidates.py` (lines 383-474) with one change: replace the call to `normalize_component(component, cfg.normalization)` with `normalize_component_with_alt_bf(component, cfg.normalization, bf_norm_fn)`. Accept `bf_norm_fn` as an additional parameter. All other logic (gating, Fisher product with `n_terms=1`, contribution decomposition) remains identical.

6. **Phase 1: `confirm_s6_decomposition()`.**

   For each `(d_mid, bf_strong)` in `S6_FAILING_CELLS`:
   a. Build fixture via `build_s6_compress_variant(d_mid, bf_strong)`
   b. Run `evaluate_fixture(fixture, "Hybrid", lambda comps: aggregate_hybrid(comps, BASELINE_HYBRID_CONFIG))`
   c. Extract from the returned `ScenarioCellResult.raw_scores` dict (keys defined by S6 branch of `evaluate.py` lines 200-218):
      - `recon_error = abs(raw_scores["reconstructed"] - raw_scores["aggregate"])`
      - `dominant_share = raw_scores["dominant_share"]`
   d. Record `S6DecompositionCheck` with `failure_is_dominant_share = (dominant_share < 0.35)` and `failure_is_recon_error = (recon_error > 1e-8)`
   e. Assert all 5 have `failure_is_dominant_share = True` and `failure_is_recon_error = False`

   Return list of `S6DecompositionCheck` results.

7. **Phase 2: `compute_bf_curves()` and `find_exact_bf_ceiling()`.**

   For each `BFNormCandidate`:
   a. Compute the BF->score curve across `BF_CURVE_RANGE`
   b. Find `bf_ceiling`: the maximum BF value where `score < 0.991` (binary search or linear scan)
   c. Record `score_at_100 = norm_fn(100)`, `score_at_500 = norm_fn(500)`, `score_at_1000 = norm_fn(1000)`
   d. Apply pre-filter: `passes_prefilter = (score_at_100 >= SCORE_AT_100_FLOOR)`

   Also include the current normalization `bf_norm_current` as the reference row (bf_ceiling should be ~111).

   Return list of `CeilingProbeResult`.

8. **Phase 3: `run_full_suite_evaluation()`.**

   **Sanity gate first**: Build patched hybrid with `bf_norm_current`. Run all 7 scenarios via `evaluate_fixture()`. For each scenario, compare the `raw_scores` dict values against running `aggregate_hybrid` with `BASELINE_HYBRID_CONFIG`. All values must match within `SANITY_TOL` (`1e-12`). If any mismatch, abort with `RuntimeError` detailing the scenario and divergence.

   For each candidate that passed the Phase 2 pre-filter:
   a. Build `patched_fn = lambda comps: aggregate_hybrid_patched(comps, BASELINE_HYBRID_CONFIG, candidate.norm_fn)`
   b. Run all 7 baseline scenarios via `evaluate_fixture()`. Record pass/fail + per-scenario margins.
   c. Run S5 BF sweep at `S5_SWEEP_BF` values (floats) via `build_s5_bf_variant()` + `evaluate_fixture()`. Record pass/fail + margins. Note: `S5_BF_VALUES` (imported from `perturbation_test.py`) is a reference constant; use `S5_SWEEP_BF` (defined in Step 1) for this sweep.

   Return list of `FullSuiteResult`.

9. **Phase 4: `check_s6_side_benefit()`.**

   For each candidate that achieved 7/7 at baseline in Phase 3:
   a. Re-run the 5 failing S6 cells using `aggregate_hybrid_patched` with the candidate's `norm_fn`. Only the BayesFactor component (`s6.bf.strong`) is affected; all other components use the standard normalization path via delegation in `normalize_component_with_alt_bf`.
   b. Check whether `dominant_share >= 0.35` for any previously-failing cell
   c. Record which cells (if any) now pass

   Return results as list of dicts.

10. **Format and write outputs.**

    Generate `ceiling_analysis.json`:
    - Timestamped, contains all phase results as serialized dicts
    - Written to the same directory as `ceiling_analysis.py`

    Generate `ceiling_analysis.md` with these sections:
    1. **S6 Decomposition**: 5-row table confirming all failures are `dominant_share < 0.35`, not reconstruction error
    2. **BF Normalization Comparison**: table with columns (name | bf_ceiling | score@100 | score@500 | score@1000 | pre-filter | 7/7 pass?)
    3. **Best Candidate Detail**: full S5 sweep results + all 7 scenario margins for the best-performing candidate
    4. **S6 Side-Benefit**: whether alternative normalization helps any of the 5 failing S6 cells
    5. **Recommendation**: structured statement indicating whether a candidate resolves the S5 ceiling, with the recommended `bf_max_target` and normalization family for the athena-e2a decision

    Write both files to `Path(__file__).resolve().parent`.

11. **Implement `main()` orchestrator.**

    Sequence: Phase 1 -> Phase 2 -> Phase 3 (with sanity gate) -> Phase 4 -> output formatting.
    Print a summary to stdout after writing files.

12. **Append investigation log entry to FINDINGS.md.**

    After the script runs successfully, append an investigation log entry at the top of the Investigation Log section in `research/adversarial-reward/FINDINGS.md`. Follow the reverse-chronological, append-only protocol. Use "Session 4.1" as the label (sub-session of Session 4's perturbation sweep). Entry must include: Scope, Method, Findings, Implications, Open Threads. Reference `ceiling_analysis.py`, `ceiling_analysis.json`, and `ceiling_analysis.md` as evidence.

END GOAL:

1. `ceiling_analysis.py` runs successfully from the `aggregation-candidates/` directory with `python ceiling_analysis.py`
2. Sanity gate passes: patched hybrid with current BF norm produces results matching baseline `aggregate_hybrid` within `1e-12` tolerance across all 7 scenarios
3. S6 decomposition confirms: all 5 failing cells have `failure_is_dominant_share = True` and `failure_is_recon_error = False`
4. At least one alternative normalization achieves 7/7 baseline pass (passes all 7 default scenario fixtures when evaluated via `evaluate_fixture` with the patched hybrid) with `bf_ceiling >= 500`
5. `ceiling_analysis.json` is valid JSON containing all 4 phase results
6. `ceiling_analysis.md` has all 5 sections with a decision-ready recommendation
7. FINDINGS.md has a new entry at the top of the Investigation Log following append-only protocol

NARROWING:

- Do NOT modify any existing files except appending to FINDINGS.md. All new code goes in `ceiling_analysis.py`.
- Do NOT duplicate logic from `normalize_component` for non-BayesFactor kinds — delegate to the existing function. The BF-branch post-processing duplication in `normalize_component_with_alt_bf` is the sole accepted exception (documented in Step 4).
- Do NOT use scipy, numpy, or external dependencies beyond the Python standard library. The existing codebase uses only `math`, `dataclasses`, `functools`, `json`, `datetime`, `pathlib`, `statistics`, `enum`, `typing`.
- Do NOT add randomness or stochastic elements. All analysis must be deterministic.
- Do NOT modify `aggregate_hybrid` in `candidates.py`. The patched version lives only in `ceiling_analysis.py`.
- Do NOT change the S5 pass criterion (`0.3 <= score <= 0.991`) or the S6 pass criterion (`recon_error <= 1e-8 AND dominant_share >= 0.35`). These are fixed evaluation rules.
- Do NOT skip the sanity gate. If the patched hybrid with `bf_norm_current` diverges from baseline beyond `1e-12`, the entire analysis is invalid.
- Do NOT create additional files beyond `ceiling_analysis.py`, `ceiling_analysis.json`, and `ceiling_analysis.md`.
- Prototype code rules apply: this is a research artifact, not production code. It will be discarded when the research question is resolved.
- FINDINGS.md is append-only. New entry goes at the TOP of the Investigation Log. Do not edit or delete previous entries.

---

## Review Findings

### Issues Addressed

1. **[Critical] `bf_norm_current` formula mismatch**: Changed from `bf/(bf+1)` to `1.0 - 1.0/(1.0+bf)` to match `normalization.py` line 138 exactly. Removed "bit-identical" language; consistently use `1e-12` tolerance with named constant `SANITY_TOL`.
2. **[Critical] `normalize_component_with_alt_bf` duplication contradiction**: Added explicit "Acknowledged tradeoff" note in Step 4 explaining the duplication is confined and pragmatic. Updated NARROWING to call out this sole accepted exception. Removed contradiction between "reuse over rewrite" and the step-level instruction.
3. **[Warning] Missing imports**: Added `ScenarioCellResult` and `DEFAULT_CUSTOM_SIGMOIDS` to Step 1 imports.
4. **[Warning] Phase 1 key assumptions**: Added source reference to `evaluate.py` lines 200-218.
5. **[Warning] Pre-filter clarity**: Reworded as "loose sanity check on raw normalization output."
6. **[Warning] Line reference**: Corrected `383-480` to `383-474`.
7. **[Warning] Phase 4 function specification**: Added explicit "Use `aggregate_hybrid_patched`; only BF component affected."
8. **[Warning] `S5_SWEEP_BF` type inconsistency**: Changed to floats, added clarifying note in Step 8c.

### Remaining Suggestions

- ~700 line estimate may be tight; implementing agent should prioritize completeness over line count
- `alpha < 1` comment is informational, kept as documentation
- Session 4.1 sub-numbering is novel but self-explanatory from context
- `frozen=True` + `Callable` field is acceptable since no hashing needed

## Usage Notes

- **Best used with:** Claude Opus or Sonnet with full codebase context (all files in `aggregation-candidates/` directory)
- **Adjust for:** If the existing codebase has been modified since this prompt was created, verify import paths and function signatures still match
