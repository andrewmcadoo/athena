# Prompt: Session 2 Aggregation Structural Fixes & Sweep

> Generated: 2026-02-21 | Framework: RISEN

---

## Session Goal

Implement Session 2 of the adversarial reward aggregation research — applying structural fixes (SE-dampening, multiplicity bonus, soft aggregation, SE-aware reliability) to three aggregation candidates, running a parameter sweep across ~603 configurations, executing calibration simulations and correlation robustness analysis, and updating FINDINGS.md with results.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation process with clear sequential methodology (structural fixes -> sweep -> calibration -> analysis -> documentation), step dependencies, and well-defined success criteria.
- **Alternatives considered:** TIDD-EC (good for precision constraints but the dominant characteristic is sequential methodology, not boundary enforcement)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Each step has a clear "what" and "why"; code locations are pinpointed |
| Specificity | 10/10 | Exact line numbers, function signatures, parameter ranges, pass criteria |
| Context | 9/10 | Full codebase structure provided; Session 1 results referenced with data |
| Completeness | 9/10 | 9 steps covering code changes, sweep, calibration, correlation, docs, backward compat |
| Structure | 9/10 | RISEN framework with clear dependencies and priority ordering |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

```
ROLE:
You are a research prototype engineer working on ATHENA's adversarial reward aggregation system. You have expertise in scientific computing, statistical aggregation methods (inverse-variance weighting, Fisher's method, hypothesis-testing gating), and signal-to-noise analysis. You are working in a research codebase where prototypes are throwaway artifacts — correctness and scientific rigor matter, production polish does not.

INSTRUCTIONS:
1. All work happens in `research/adversarial-reward/prototypes/aggregation-candidates/`.
2. Structural fixes must be gated by boolean config flags defaulting to `False` so that `evaluate.py` with default configs reproduces Session 1 results identically (backward compatibility).
3. The existing codebase uses frozen dataclasses, pure functions, and no external dependencies beyond Python stdlib. Maintain this style.
4. Every new config parameter must have a sensible default that preserves existing behavior when the feature is disabled.
5. Read the existing source files before modifying them — understand the normalization pipeline (`normalize_component` → kind-specific CDF → direction adjustment → bounded clipping) and each candidate's aggregation logic.
6. The sweep must test ALL 7 scenarios for every config combination. Use the existing `build_scenario_fixtures()` and `evaluate_fixture()` infrastructure from `evaluate.py`.
7. Update FINDINGS.md per append-only protocol: new investigation log entry at top (reverse chronological), update Accumulated Findings with evidence citations.

PRIORITY ORDERING:
- **Mandatory** (session fails without these): Steps 1-5, 8, 9
- **Stretch goals** (defer to Session 2b if sweep takes longer than expected): Steps 6, 7
- If the Stage 1 normalization sweep does not find a clear winner, document the top-3 normalization configs and run Stage 2 with each. Note this in FINDINGS.md.

STEPS:
1. **SE-dampening in normalization layer** — Modify `normalization.py`:
   - Add three fields to `NormalizationConfig` (after line 22): `se_dampen_enabled: bool = False`, `se_dampen_k: float = 5.0`, `se_dampen_x0: float = 2.0`
   - Add a module-level function after the existing `sigmoid` function (line 34):
     ```python
     def se_dampen(raw_score: float, value: float, se: float, config: NormalizationConfig) -> float:
         snr = abs(value) / se
         return raw_score * sigmoid(snr, config.se_dampen_k, config.se_dampen_x0)
     ```
   - Wire into `normalize_component()` as the final step before the return on line 167. Add explicit code:
     ```python
     if config.se_dampen_enabled:
         snapshot = extract_uncertainty_snapshot(component)
         if snapshot.standard_error is not None and snapshot.standard_error > 0:
             final_score = se_dampen(final_score, component.value, snapshot.standard_error, config)
     ```
     Note: `value` is `component.value` (the raw input value), NOT `transformed_value`. This ensures SNR is computed from the original measurement scale. For S1 Noisy TV: base SNR = |0.0012|/0.30 = 0.004, doubled SNR = |0.0024|/0.60 = 0.004 — identical dampening, test passes.
   - Edge case: if SE is 0 or None, skip dampening (pass through unchanged — handled by the guard above).

2. **Multiplicity bonus for IVW-CDF** — Modify `candidates.py` (IVWCDFConfig ~line 20, aggregate_ivw_cdf ~line 57):
   - Add three fields to `IVWCDFConfig`: `multiplicity_bonus_enabled: bool = False`, `multiplicity_threshold: float = 0.1`, `multiplicity_scale: float = 0.5`
   - After computing `aggregate` on line 91 and before bounding on line 92: if `multiplicity_bonus_enabled` and `len(staged) > 1`:
     ```python
     concordant = sum(1 for entry in staged if float(entry["score"]) > cfg.multiplicity_threshold)
     concordance = concordant / len(staged)
     bonus = 1.0 + cfg.multiplicity_scale * concordance * math.log(len(staged))
     aggregate = aggregate * bonus
     ```
     The existing `bounded_unit_interval` call on line 92 handles clipping to [0, 1].
   - Why this is feasible for S2: with 8 concordant metrics, bonus = 1 + scale * 1.0 * ln(8). At scale=1.5, bonus=4.12. Current S2 IVW-CDF aggregate ≈ 0.27, so 0.27 * 4.12 = 1.11, clipped to ~1.0, which exceeds the 1.5 * max_single threshold of 0.87.

3. **Soft aggregation mode for HTG-Max** — Modify `candidates.py` (HTGMaxConfig ~line 27, aggregate_htg_max ~line 128):
   - Add field to `HTGMaxConfig`: `soft_sum_boost: float = 2.0`
   - Insert an `elif cfg.mode == "soft_sum":` branch between the end of the `lse_rebound` block (line 190, after `winner_weight_map` assignment) and the `else:` clause (line 191):
     ```python
     elif cfg.mode == "soft_sum":
         raw_sum = sum(float(entry["gated_score"]) for entry in winners)
         aggregate = raw_sum / len(winners) * cfg.soft_sum_boost
         winner_weight_map = {int(entry["idx"]): 1.0 / len(winners) for entry in winners}
     ```
   - The existing `bounded_unit_interval` call on line 197 handles clipping.
   - `soft_sum_boost` is fixed at 2.0 for this session; sweeping it is deferred to Session 3.

4. **SE-aware reliability for Fisher-UP** — Modify `candidates.py` (FisherUPConfig ~line 38, aggregate_fisher_up ~line 238):
   - Add three fields to `FisherUPConfig`: `se_reliability_enabled: bool = False`, `se_reliability_k: float = 3.0`, `se_reliability_x0: float = 2.0`
   - In aggregate_fisher_up, after computing `reliability` on lines 255-259:
     ```python
     if cfg.se_reliability_enabled and snapshot.standard_error is not None and snapshot.standard_error > 0:
         snr = abs(component.value) / snapshot.standard_error
         se_factor = sigmoid(snr, cfg.se_reliability_k, cfg.se_reliability_x0)
         reliability = reliability * se_factor
     ```
   - This overlaps with Step 1's SE-dampening. To test "in isolation": include a separate Stage 2 run with `se_dampen_enabled=False` in the normalization config combined with `se_reliability_enabled=True` in Fisher config. See Step 5 for details.

5. **Create `sweep.py`** — New file with two-stage parameter sweep:
   - Import from `candidates`, `normalization`, `scenarios`, `evaluate` (reuse `build_scenario_fixtures`, `evaluate_fixture`, `ScenarioCellResult`)

   **Stage 1 (Normalization sweep) — 81 configs × 3 candidates = 243 candidate-configs:**
   - All with `se_dampen_enabled=True`, sweeping:
     - `abs_diff_k` (maps to `NormalizationConfig.absolute_difference_sigmoid.k`): [800, 1200, 2000]
     - `abs_diff_x0` (maps to `NormalizationConfig.absolute_difference_sigmoid.x0`): [5e-4, 7e-4, 1e-3]
     - `se_dampen_k`: [3.0, 5.0, 8.0]
     - `se_dampen_x0`: [1.0, 2.0, 3.0]
   - Run each normalization config × 3 candidates (default candidate params) × 7 scenarios = 1701 evaluations
   - Select best normalization config: most total scenario passes across all 3 candidates (tiebreak by highest average aggregate score across passing scenarios)

   **Stage 2 (Candidate-specific sweeps) — fix best normalization from Stage 1:**

   IVW-CDF (~60 configs):
   - `multiplicity_bonus_enabled=True`
   - `multiplicity_scale`: [0.3, 0.5, 0.8, 1.0, 1.5]
   - `multiplicity_threshold`: [0.05, 0.1, 0.2]
   - `w_default`: [0.1, 0.5, 1.0, 2.0]

   HTG-Max (~180 configs):
   - `alpha`: [1.0, 1.5, 2.0]
   - `tau`: [5.0, 7.8, 12.0]
   - `c_floor`: [0.15, 0.3, 0.5, 0.7]
   - `mode`: ["hard_max", "lse_rebound", "soft_sum"]
   - `lse_beta`: [0.5, 2.0, 8.0] (for `lse_rebound` mode only; for `hard_max` and `soft_sum`, use default)
   - `soft_sum_boost`: fixed at 2.0 (not swept this session)
   - Config count: hard_max 36 + lse_rebound 108 + soft_sum 36 = 180

   Fisher-UP (~120 configs):
   - `n_ref`: [50, 100, 200]
   - `r_floor`: [0.1, 0.3, 0.5, 0.7]
   - `se_reliability_enabled`: [True, False]
   - When `se_reliability_enabled=True`: `se_reliability_k` in [2.0, 3.0, 5.0], `se_reliability_x0` in [1.5, 2.0, 3.0] (9 combos)
   - When `se_reliability_enabled=False`: k and x0 are irrelevant (1 combo)
   - Config count: 12 × (1 + 9) = 120
   - **Isolation test:** Also run 120 Fisher configs with `se_dampen_enabled=False` in the normalization config (to test SE-reliability without SE-dampening). Compare against the main sweep results.

   Total: ~243 (Stage 1) + ~60 + ~180 + ~120 + ~120 (isolation) = ~723 candidate-configs. All pure math, completes in seconds.

   **S2 criterion sensitivity:** For configs that pass 6/7 (failing only S2), sweep the S2 target multiplier from 1.0→2.0 in 0.1 increments. Report the feasibility frontier: what minimum multiplier threshold each candidate's best config can achieve.

   **Output files:**
   - `sweep_results.json`: Full results (every config, every scenario, pass/fail, scores)
   - `sweep_summary.md`: Top-5 configs per candidate by scenario passes (tiebreak by avg score), any 7/7 configs highlighted, S2 feasibility frontier table

6. **Create `calibration_sim.py`** — 50-cycle extended calibration *(stretch goal)*:
   - All calibration simulations are deterministic (no random seeds). Statistical thresholds are compared against exact values, not sampling distributions.
   - Use S6's calibration fixture as the base metric set (6 diverse metrics, well-characterized).
   - Three drift patterns, 50 cycles each, using the best configs from the sweep:

   | Pattern | Description | Implementation | Pass metric |
   |---------|-------------|----------------|-------------|
   | A: Gradual convergence | Values decrease toward 0 | value_t = initial * (1 - t/50), SE constant | Spearman rho < -0.9 |
   | B: Sudden regime change | Weak then strong | value × 0.1 for t=0-24, one metric jumps to value × 5.0 at t=25 | step_ratio > 3.0 |
   | C: Oscillating uncertainty | SE oscillates | Values constant, SE = base * (0.55 + 0.45 * sin(2π*t/50)) | Pearson r(SE, score) < -0.5 |

   - Smoothness check for all patterns: max |score[t+1] - score[t]| < 0.3
   - Implement Spearman rho using ranks (stdlib only — no scipy). Pearson r from the standard formula.
   - **Output:** `calibration_results.json` (raw cycle-by-cycle data), `calibration_summary.md` (pass/fail per pattern per candidate, metric values, smoothness check)

7. **Fisher-UP correlation robustness** *(stretch goal)* — Create `correlation_test.py`:
   - Generate S2-like fixtures with 8 weak metrics at correlation levels rho = [0.0, 0.3, 0.5, 0.7, 0.9]
   - For each rho: build an 8×8 correlation matrix (uniform pairwise correlation = rho), apply Cholesky decomposition to generate correlated metric values, run Fisher-UP aggregate
   - Use `random.seed(42)` for reproducibility. Represent matrices as `list[list[float]]`. The Cholesky decomposition is a textbook O(n³) implementation over an 8×8 matrix — numerical stability is not a concern at this scale.
   - Implement Brown's correction: for correlated p-values, effective df = 2k² / var(T) where T = sum(-2*log(p_i)). Compare corrected vs. uncorrected aggregates.
   - Report inflation ratio = uncorrected_aggregate / corrected_aggregate at each rho level
   - Flag if inflation ratio > 1.5 at rho=0.5
   - **Output:** Results appended to `sweep_summary.md` or written to `correlation_results.json`

8. **Update FINDINGS.md** — `research/adversarial-reward/FINDINGS.md`:
   - Add new investigation log entry at the TOP (reverse chronological order) with:
     - **Scope:** Session 2 — structural fixes + parameter sweep + calibration + Fisher correlation analysis
     - **Method:** SE-dampening, multiplicity bonus, soft aggregation, SE-aware reliability, two-stage sweep (~603+ configs), 3-pattern calibration simulation, Brown's correction analysis
     - **Findings:** Sweep results matrix (best config per candidate, scenario pass counts), calibration metrics (Spearman/step ratio/Pearson per pattern), correlation inflation ratios
     - **Implications:** Whether any candidate achieves 7/7, calibration stability assessment, Fisher correlation sensitivity
     - **Open Threads:** Items for Session 3
   - Update Accumulated Findings sections (What We Know / What We Suspect / What We Don't Know) — move items between categories based on new evidence, cite log entry numbers
   - Register new prototypes in Prototype Index table: `sweep.py`, `calibration_sim.py`, `correlation_test.py` (if completed), output artifacts
   - If Steps 6-7 are deferred, note this in Open Threads with "Deferred from Session 2 — see plan"

9. **Verify backward compatibility** — Run `python evaluate.py` with default configs (all boolean flags default to False). Confirm output matches Session 1 results exactly:
   - IVW-CDF: 5/7 (S1 FAIL, S2 FAIL)
   - HTG-Max: 5/7 (S2 FAIL, S4 FAIL)
   - Fisher-UP: 3/7 (S1 FAIL, S2 FAIL, S4 FAIL, S7 FAIL)

END GOAL:
1. `python evaluate.py` with default configs reproduces Session 1 results exactly (5/7, 5/7, 3/7)
2. `python sweep.py` completes without error; writes `sweep_results.json` and `sweep_summary.md`
3. At least one candidate config achieves 7/7, OR the best config per candidate is documented with clear failure reasons and the minimum S2 multiplier threshold each can achieve
4. (Stretch) `python calibration_sim.py` completes; reports Spearman rho, step ratio, Pearson r, and smoothness metrics for all 3 patterns × best configs
5. (Stretch) Fisher correlation analysis runs; inflation ratios reported at 5 rho levels; Brown's correction implemented
6. FINDINGS.md updated per append-only protocol with new log entry and updated accumulated findings
7. All new prototypes registered in the Prototype Index

NARROWING:
- Do NOT modify `models.py` or `scenarios.py` — these are read-only for this session
- Do NOT add external dependencies — stdlib only (math, json, dataclasses, statistics, pathlib, itertools, datetime, random)
- Do NOT change default parameter values on existing config dataclasses — backward compatibility requires defaults to match Session 1
- Do NOT edit or delete previous FINDINGS.md log entries — append-only protocol
- Do NOT write production code — these are throwaway research prototypes
- Do NOT over-engineer output formatting — plain markdown tables and JSON are sufficient
- Avoid breaking the frozen dataclass pattern — all new config fields must have defaults
- Do NOT use `numpy`, `scipy`, or `pandas` — implement Cholesky decomposition, Spearman rho, and Pearson r from scratch
- The S2 scenario pass criterion is `aggregate >= 1.5 * max(single_metric_scores)` — do NOT change this threshold in `evaluate.py`. The sweep's S2 sensitivity analysis tests alternative thresholds separately.
- Out of scope: hybrid candidate design (combining elements of IVW + HTG + Fisher). That is Session 3 work.
- Out of scope: sweeping `soft_sum_boost` for HTG-Max. Fixed at 2.0 this session.
```

---

## Review Findings

### Issues Addressed
1. **Critical: Sweep counts corrected** — HTG-Max ~180, Fisher-UP ~120, total ~603+ (including isolation test). Explicit config count breakdowns added to Step 5.
2. **Critical: se_dampen wiring made explicit** — Added exact code showing `extract_uncertainty_snapshot` call, `component.value` usage, and guard conditions. Noted that `value` is the raw input, not `transformed_value`.
3. **Warning: Priority ordering added** — Steps 6-7 marked as stretch goals with deferral guidance.
4. **Warning: Soft_sum insertion point fixed** — Changed from "~line 184" to "between line 190 and line 191 (the else clause)".
5. **Warning: soft_sum_boost scope clarified** — Explicitly noted as fixed at 2.0, sweep deferred to Session 3.
6. **Warning: SE-dampening isolation test added** — Step 5 now includes 120 Fisher configs with `se_dampen_enabled=False` to test SE-reliability in isolation.
7. **Warning: Calibration determinism noted** — Added "All calibration simulations are deterministic" statement.
8. **Warning: se_dampen placement specified** — "module-level function after the existing `sigmoid` function definition".

### Remaining Suggestions
- S2 multiplicity bonus feasibility: a worked example is included in Step 2 showing scale=1.5 achieves the target
- Cholesky implementation: seed and matrix representation specified; O(n³) textbook approach approved
- Output file naming: no conflicts with Session 1 artifacts (different names)

## Usage Notes

- **Best used with:** Claude Opus or Sonnet in Claude Code with file access to the athena repository
- **Adjust for:** If Session 1 results have changed since the prompt was written, update the backward compatibility check in Step 9
