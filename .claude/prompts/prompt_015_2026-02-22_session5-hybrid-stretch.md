# Prompt: Session 5 Hybrid Stretch Analyses

> Generated: 2026-02-22 | Prompt #2 | Framework: RISEN

---

## Session Goal

Implement a stretch test script (`stretch_test.py`) for the post-ceiling hybrid aggregation candidate that runs calibration simulation (3 patterns), correlation robustness (5 rho levels with floor-saturation fix), and generates machine-readable + human-readable output — validating readiness for the athena-6ax recommendation (the formal AggregateScore type recommendation bead, blocked by this work).

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step implementation with clear methodology (4 phases), specific technical constraints, and a well-defined end goal. RISEN maps directly to the phased structure.
- **Alternatives considered:** TIDD-EC (good for precision dos/don'ts, but sequential phase structure is dominant); Chain of Thought (overkill — reasoning steps already explicit in plan)

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal, phases, and pass criteria are unambiguous |
| Specificity | 10/10 | Exact thresholds, function signatures, import paths, fixture choices |
| Context | 9/10 | Session history, prior findings, and design decision rationale included |
| Completeness | 9/10 | All phases, verification steps, and output artifacts specified |
| Structure | 9/10 | Clean RISEN mapping to phased implementation |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

ROLE:
You are a scientific computing implementer working on the ATHENA adversarial reward research track. You have expertise in Bayesian hypothesis testing (Bayes factors, Fisher's method), statistical simulation (calibration patterns, correlation robustness), and Python scientific scripting. You are familiar with the existing aggregation-candidates codebase and its module conventions.

INSTRUCTIONS:
1. Create a single new file `research/adversarial-reward/prototypes/aggregation-candidates/stretch_test.py` (~350-450 lines) that stress-tests the post-ceiling hybrid aggregation candidate (HTG gating + Fisher product, log-scaled BF normalization with c=0.083647 from calibrate_log_scaled(bf_max_target=10000), bf_max=10000).
2. Reuse existing modules by importing — do not copy or rewrite functions that already exist.
3. Follow the 4-phase structure exactly: Phase 0 (baseline gate) → Phase 1 (calibration) → Phase 2 (correlation) → Phase 3 (output). Abort early if Phase 0 fails.
4. Use deterministic seed `random.seed(42)` set at the beginning of Phase 2 (not at script start) so Phase 1 changes cannot affect Phase 2 determinism.
5. Write investigation results to both machine-readable JSON and human-readable Markdown.
6. After script runs successfully, append a Session 5 investigation log entry (this is a new top-level session, not 4.2) to `research/adversarial-reward/FINDINGS.md` (reverse-chronological, top of Investigation Log) and add 3 rows to the Prototype Index table.

STEPS:
1. **Construct the hybrid function:**
   - Import `aggregate_hybrid_patched` + `bf_norm_log_scaled` from `ceiling_analysis.py`
   - Import `BASELINE_HYBRID_CONFIG` from `perturbation_test.py`
   - Build hybrid_fn: `lambda comps: aggregate_hybrid_patched(comps, BASELINE_HYBRID_CONFIG, partial(bf_norm_log_scaled, c=0.083647))`
   - Build scorer for calibration use: `lambda comps: hybrid_fn(comps).aggregate_score` (calibration patterns require `Callable[[Sequence[MetricComponent]], float]`)
   - Build log_evidence extractor: for correlation use, sum `contribution.diagnostics["log_evidence"]` across all contributions in the returned `AggregateResult`

2. **Phase 0 — Baseline Re-verification (pre-gate):**
   - Import `evaluate_fixture` from `evaluate.py`, `build_scenario_fixtures` from `scenarios.py`, `margin_from_cell` from `ceiling_analysis.py`
   - Run all 7 scenario fixtures through evaluate_fixture with the hybrid_fn
   - Compute margin for each via margin_from_cell
   - Gate: if ANY scenario fails, raise RuntimeError with details. Do not proceed.

3. **Phase 1 — Calibration Simulation (50 cycles, 3 patterns):**
   - Import `run_pattern_a`, `run_pattern_b`, `run_pattern_c`, `with_value_scale`, `with_uncertainty_scale`, `pearson_r`, `spearman_rho`, `rank_values` from `calibration_sim.py`
   - Use S6 calibration decomposability fixture (6 metrics) as base
   - Run each pattern with the hybrid scorer
   - Pass criteria (unchanged from Session 2 for direct comparison):
     * Pattern A (gradual convergence): spearman_rho < -0.9, max_delta < 0.3
     * Pattern B (sudden regime change): step_ratio > 3.0, max_delta < 0.3
     * Pattern C (oscillating uncertainty): pearson_r < -0.5, max_delta < 0.3
   - Note: Pattern B smoothness failure (step_ratio > 3.0 but max_delta >= 0.3) is diagnostically meaningful — report as "responsive but non-smooth" in narrative analysis
   - Hard-code Session 2 single-family results (from `calibration_summary.md`) for side-by-side comparison table

4. **Phase 2 — Correlation Robustness (5 rho levels):**
   - Import `correlation_matrix`, `cholesky_decompose`, `sample_correlated`, `perturb_component`, `variance` from `correlation_test.py`
   - Import `chi_square_cdf_even_df` from `candidates.py`
   - Use S6 fixture (6 metrics, strong signals — Phase 0 will verify the exact baseline) — NOT S2 (floor-saturated at ~1e-12)
   - Parameters: rho_values = [0.0, 0.3, 0.5, 0.7, 0.9], 400 samples/rho, random.seed(42) at Phase 2 start
   - Per-sample pipeline:
     a. Build equicorrelation matrix, Cholesky decompose
     b. Sample 400 correlated noise vectors
     c. Perturb S6 components, run aggregate_hybrid_patched, extract aggregate_score. For total log_evidence (T), sum contribution.diagnostics["log_evidence"] across all contributions in the AggregateResult
     d. Compute Var(T) across 400 T-values. If Var(T) = 0, use effective_df = 2*k as fallback
     e. Brown correction: effective_df = 2*k^2 / Var(T), corrected_terms = clamp(int(effective_df/2), 1, 1000)
     f. Corrected score: chi_square_cdf_even_df(T, n_terms=corrected_terms)
     g. Inflation ratio: mean(uncorrected) / max(mean(corrected), 1e-12)
   - Note: The hybrid uses n_terms=1 (not n_terms=k as in standard Fisher combination). The Brown correction is applied as a diagnostic comparison against the Session 2 Fisher-UP correlation test, not as a theoretically exact correction for the hybrid. Report effective_df alongside n_terms=1 for interpretive context.
   - Floor-saturation diagnostic: floor_count = count of scores <= 2e-12; floor_saturated = floor_count/total > 0.5
   - Pass criterion: inflation_ratio <= 1.5 at rho=0.5 AND floor_saturated == False

5. **Phase 3 — Output Generation:**
   - Write `stretch_results.json` with: hybrid_config (bf_norm family, c, bf_max_target, base config), phase0 verification (7 per-scenario pass/margin rows), phase1 calibration (per-pattern metrics + cycles + Session 2 comparison), phase2 correlation (per-rho results + floor diagnostics + Session 2 comparison). All outputs except `generated_at_utc` are deterministic given seed=42.
   - Write `stretch_summary.md` with: baseline verification table, calibration results + comparison table, Pattern B narrative analysis (key question: does the hybrid's HTG gating + Fisher product respond correctly to sudden single-metric regime change, where all Session 2 single-family candidates failed?), correlation results + floor-saturation assessment, summary verdict

6. **FINDINGS.md Update:**
   - Append Session 5 log entry (reverse-chronological, at top of Investigation Log) with: Scope, Method, Findings, Implications, Open Threads
   - Add 3 rows to Prototype Index: stretch_test.py, stretch_results.json, stretch_summary.md. Status: `Complete (Session 5 Stretch)` for all three. Demonstrated column should summarize what each artifact proved.

END GOAL:
- `stretch_test.py` runs end-to-end via `python stretch_test.py` producing `stretch_results.json` and `stretch_summary.md` in the same directory
- Phase 0: all 7 scenarios pass with margins matching ceiling_analysis.md baseline
- Phase 1: calibration results for all 3 patterns with direct comparison to Session 2 single-family results; Pattern B result answers whether the hybrid survives sudden regime change (the key scientific question where all single-family candidates failed in Session 2)
- Phase 2: floor_saturated == False for all rho levels (validates the S6 fixture fix); inflation_ratio reported for each rho with pass/fail at rho=0.5
- Determinism: running twice produces identical JSON (timestamp excepted)
- FINDINGS.md updated with Session 5 entry and Prototype Index rows

NARROWING:
- Do NOT modify any existing source files (ceiling_analysis.py, perturbation_test.py, calibration_sim.py, correlation_test.py, candidates.py, etc.) — import only
- Do NOT change pass criteria thresholds from Session 2 — direct comparison requires identical criteria
- Do NOT use S2 fixture for correlation test — it floor-saturates at ~1e-12, making results diagnostically meaningless
- Do NOT write production code — this is a research prototype
- Do NOT edit previous FINDINGS.md log entries — append-only protocol
- Do NOT skip Phase 0 gate — if baseline verification fails, the stretch results are invalid
- Do NOT access `total_log_evidence` as a field on AggregateResult — it does not exist; sum contribution.diagnostics["log_evidence"] instead
- Stay within the existing module conventions and import patterns of the aggregation-candidates directory

---

## Review Findings

### Issues Addressed
1. **[CRITICAL] Scorer extraction ambiguity** — Made explicit: `lambda comps: hybrid_fn(comps).aggregate_score` with type annotation
2. **[CRITICAL] Missing `total_log_evidence` on return type** — Added extraction via `contribution.diagnostics["log_evidence"]` sum, plus NARROWING constraint
3. **[CRITICAL] Session label ambiguity** — Clarified as "Session 5 (new top-level session, not 4.2)"
4. **[WARNING] Magic constant `c=0.083647`** — Now cites `calibrate_log_scaled(bf_max_target=10000)`
5. **[WARNING] Pattern B smoothness** — Acknowledged as diagnostically meaningful, "responsive but non-smooth" narrative guidance added
6. **[WARNING] Seed placement** — Specified at Phase 2 start, not script start
7. **[WARNING] Brown correction mismatch** — Noted as diagnostic comparison for n_terms=1, not theoretically exact
8. **[WARNING] S6 baseline** — Changed to "Phase 0 will verify" instead of hard-coded ~0.96
9. **[WARNING] Prototype Index format** — Added status label and Demonstrated column guidance
10. **[WARNING] Var(T)=0 edge case** — Added fallback: `effective_df = 2*k`

### Remaining Suggestions
- Line-count budget allocation per phase (~40-50/80-100/100-120/100-130/30-40 split)
- Session 2 hard-coded values could include exact source path (`calibration_summary.md`)
- Determinism self-check comment at output section

## Usage Notes

- **Best used with:** Claude Code or any agent with file read/write + bash access to the ATHENA repo
- **Adjust for:** If ceiling_analysis.py API changes (e.g., AggregateResult gains total_log_evidence field), update the extraction pattern in Steps 1/4 and remove the NARROWING constraint
