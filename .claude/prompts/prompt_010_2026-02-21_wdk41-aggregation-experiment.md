# Prompt: WDK#41 Multi-Metric Profile Aggregation Experiment

> Generated: 2026-02-21 | Framework: RISEN

---

## Session Goal

Define and stress-test three candidate aggregation functions (`Vec<MetricComponent>` -> `AggregateScore`) for ATHENA's adversarial-reward track, evaluating them against 7 synthetic scenarios to select one that is bounded, monotonic, calibratable, and Noisy-TV resistant.

## Framework Selection

- **Chosen:** RISEN
- **Rationale:** Complex multi-step research process with clear methodology (implement -> test -> analyze -> recommend), explicit constraints (bounded, monotonic, calibratable, Noisy-TV resistant), and important narrowing (ATHENA's three architectural constraints). RISEN's Steps + Narrowing components map directly to the experiment protocol and architectural constraints.
- **Alternatives considered:** RISE-IE (input->output transformation) -- less ideal because the task is primarily about methodology/evaluation, not a single data transformation. Chain of Thought -- useful for reasoning within each step, but RISEN provides better overall structure.

## Evaluation Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Goal is unambiguous; each candidate and scenario has precise definition |
| Specificity | 9/10 | Mathematical formulas, exact pass criteria, concrete file paths |
| Context | 8/10 | Input contracts included verbatim; architecture references cited |
| Completeness | 9/10 | Covers what/why/how, output format specified, scope boundaries clear |
| Structure | 9/10 | RISEN components well-separated; tables for scenarios; numbered steps |
| **Overall** | **9/10** | |

---

## Structured Prompt

> Copy-paste ready. This is the primary deliverable.

ROLE:
You are a computational research scientist with expertise in Bayesian experimental design, information-theoretic metrics, and meta-analysis. You are working within ATHENA, a falsification-driven AI co-scientist that uses structured failure analysis over causal DAGs. You have deep familiarity with:
- Inverse-variance weighting and Fisher's method for combining evidence
- CDF normalization of heterogeneous divergence metrics (Z-scores, Bayes factors, KL divergence, effect sizes)
- Noisy TV degeneration in surprise-maximizing agents
- Calibration feedback loops comparing predicted vs. actual surprise

INSTRUCTIONS:
Design, implement, and stress-test three candidate aggregation functions that map `Vec<MetricComponent>` -> `AggregateScore` (a bounded [0,1] scalar representing contradiction evidence strength). Each candidate must satisfy four properties simultaneously:
1. **Bounded:** Output strictly in [0,1] on all inputs
2. **Monotonic:** Higher values = stronger contradiction evidence (convention G2)
3. **Calibratable:** Decomposable into per-component `(score, weight)` pairs so the section 5.4 calibration feedback loop can diagnose systematic miscalibration
4. **Noisy-TV resistant:** High-uncertainty metrics must not inflate the aggregate -- the function must degrade gracefully when value and uncertainty both increase

Follow the steel-man-then-stress-test methodology: build the strongest version of each candidate, then systematically attack it with adversarial scenarios. Evaluation priority: Noisy TV resistance > Calibratability > Signal compounding > Robustness to missing data.

**Input contracts** (consume as-is, do not modify):

```
MetricComponent {
  kind: DivergenceKind   // AbsoluteDifference|ZScore|BayesFactor|KLDivergence|EffectSize|Custom
  value: f64
  direction: Option<EffectDirection>  // Contradiction|Agreement|None
  uncertainty: Option<UncertaintySummary>
  sample_size: Option<u32>
  units: Option<Unit>
  method_ref: String
}

UncertaintySummary {
  point: PointUncertainty
  distribution: Option<DistributionPayload>
}

PointUncertainty =
  | Summary { sample_size: Option<u32>, standard_error: Option<f64>,
              interval: Option<IntervalEstimate>, method_ref: String }
  | NoUncertainty { reason: UncertaintyUnavailableReason }
```

**EffectDirection handling:** During CDF normalization, if `direction == Agreement`, invert the normalized score: `score = 1 - cdf_score`. If `direction` is absent, treat the metric as unsigned contradiction evidence (take absolute value before CDF transform). This ensures agreement metrics reduce -- not inflate -- the aggregate.

**Custom DivergenceKind handling:** Apply a configurable sigmoid `sigma(x) = 1 / (1 + exp(-k*(x - x0)))` with `k` and `x0` as required parameters on Custom metrics. If parameters are absent, exclude the metric from aggregation and log a warning. Do not silently default.

STEPS:

1. **Create prototype directory** at `research/adversarial-reward/prototypes/aggregation-candidates/` with Python dataclasses mirroring `MetricComponent` and `UncertaintySummary` from the contracts above. Include `EffectDirection` as an enum with `Contradiction`, `Agreement`, and `None` variants.

2. **Implement Candidate 1 -- Inverse-Variance Weighted CDF-Rank (IVW-CDF):**
   - Normalize each metric to [0,1] via kind-specific CDF transforms: ZScore -> `2*Phi(|z|)-1`, BayesFactor -> `1 - 1/(1+BF)`, KLDivergence -> `1 - exp(-kl)`, AbsoluteDifference -> domain-calibrated sigmoid, EffectSize -> `2*Phi(|d|)-1`, Custom -> configurable sigmoid (see above)
   - Apply EffectDirection inversion after normalization
   - Weight by inverse variance: `w_i = n / (se^2 + eps)`, with `w_default` fallback when uncertainty is absent. Use `MetricComponent.sample_size` as primary source; fall back to `PointUncertainty.sample_size` if the former is absent.
   - Aggregate: `score = sum(w_i * u_i) / sum(w_i)` -> [0,1]

3. **Implement Candidate 2 -- Hierarchical Type-Grouped Maximum with Confidence Gating (HTG-Max):**
   - Same CDF normalization and EffectDirection handling as C1
   - Confidence gate: `c_i = sigmoid(alpha * (precision_i - tau))`, with `c_floor` for missing uncertainty
   - Gated score: `g_i = u_i * c_i`
   - Group by `DivergenceKind`, take max gated score within each group
   - Across groups: use hard max (not LogSumExp) to guarantee [0,1] boundedness. If LogSumExp is explored as an alternative, apply a normalizing transform `1 - exp(-LSE)` to re-bound to [0,1], and document both variants.

4. **Implement Candidate 3 -- Fisher-Style CDF-Rank with Uncertainty Penalty (Fisher-UP):**
   - Map each metric to a "contradiction p-value" under a null (no-contradiction) distribution
   - Apply EffectDirection: agreement metrics contribute p-values near 1.0 (non-significant)
   - Reliability exponent: `r_i = min(1, n_i / n_ref) * indicator(uncertainty != NoUncertainty)`, with `r_floor` fallback (e.g., 0.1) when uncertainty is absent. `n_ref` is a configurable reference sample size (default: 100).
   - Adjusted p-value: `p_adj = p^(r_i)` -- uncertain metrics get `r_i` close to 0, pushing `p_adj` toward 1 (non-significant)
   - Fisher combination: `X = -2 * sum(ln(p_adj))`, aggregate = `chi2_cdf(X, 2N)` -> [0,1]

5. **Build 7 scenario fixtures** as parameterized test data:

   | # | Name | What it tests | Pass criterion |
   |---|------|---------------|----------------|
   | 1 | Noisy TV | One metric: high divergence + high uncertainty | Aggregate does NOT increase when value and se both double; aggregate should decrease or remain stable (document direction) |
   | 2 | Unanimous weak signal | 8 metrics with small consistent contradiction | Aggregate >= 1.5x any single metric's score |
   | 3 | Mixed signal | 3 contradiction + 3 agreement metrics (using EffectDirection) | Aggregate between all-contradiction and all-agreement extremes |
   | 4 | Missing data | Metrics with partial/absent uncertainty (NoUncertainty variants) | No NaN/crash; within 20% of full-uncertainty baseline |
   | 5 | Scale heterogeneity | ZScore(2.0) vs BayesFactor(100) vs AbsDiff(0.001 eV) | CDF-normalized scores all land in [0.3, 0.99] (no degenerate collapse to 0 or 1); relative ranking is stable across all 3 candidates |
   | 6 | Calibration decomposability | Single invocation with 6 diverse metrics | Per-component (score, weight) decomposition sums to aggregate within floating-point tolerance; decomposition is sufficient to identify which component dominates |
   | 7 | Boundary-seeking | High contradiction at domain parameter bounds with inflated uncertainty | Aggregate lower than equivalent non-boundary case (same metric values, lower uncertainty) |

6. **Run all 3 candidates on all 7 scenarios.** Record raw scores, per-component decompositions `(u_i, w_i)`, and pass/fail for each cell. Format results as a 3x7 matrix with scores and pass/fail verdicts.

7. **Analyze results and write investigation log entry** in `research/adversarial-reward/FINDINGS.md` following the append-only protocol:
   - Update Status from "NOT STARTED" to "IN PROGRESS"
   - Entry format: Scope, Method, Findings (with the 3x7 matrix), Implications, Open Threads
   - Update Accumulated Findings sections (What We Know / What We Suspect / What We Don't Know) with evidence-backed claims referencing the log entry
   - Register prototype in the Prototype Index table with filename, purpose, status, and what was demonstrated
   - Do NOT edit previous entries or reorder the existing 5 items in the Next Steps section

END GOAL:
A completed Session 1 of the WDK#41 experiment with:
- All 3 candidates producing bounded [0,1] output on all 7 scenarios (no NaN, no out-of-range)
- A 3x7 pass/fail matrix identifying which candidates pass which scenarios
- At least one candidate passing all 7 scenarios, or clear documentation of which fail and why
- Per-component decompositions recorded for calibration analysis in Session 2
- Investigation log entry written in `research/adversarial-reward/FINDINGS.md` with evidence-backed findings
- Prototype files registered in the Prototype Index table with filename, purpose, status, and what was demonstrated

NARROWING:
- Do NOT write production code. Prototypes are throwaway research artifacts in `prototypes/` only.
- Do NOT modify the input contracts (`MetricComponent`, `UncertaintySummary`). Consume them as-is.
- Do NOT replace or reorder the existing 5 items in the Next Steps section of `research/adversarial-reward/FINDINGS.md`. WDK#41 is "Step 0" inserted before them.
- Do NOT weaken ATHENA's three architectural constraints (DSL-only environments, warm-started causal priors, bounded adversarial design).
- Avoid unbounded surprise-maximization in any candidate -- this causes Noisy TV degeneration (VISION.md Section 6.2).
- Do NOT fabricate domain-expert evidential weights. Use the documented CDF transforms and note calibration tuning as a Session 2 concern.
- Do NOT edit previous entries in the Investigation Log. New entries go at the top (reverse chronological).
- Out of scope: Session 2 (parameter sensitivity sweeps) and Session 3 (recommendation and AggregateScore type definition). Record open threads for these but do not execute them.
- Stay within Python + standard scientific libraries (numpy, scipy) for prototype. No new ADR required for prototype-scoped technology choices.
- `IntervalEstimate` fields on `UncertaintySummary` can be treated as opaque for Session 1 prototyping. Consume `standard_error` and `sample_size` only.

---

## Review Findings

### Issues Addressed
1. **W1 (Scenario 6 scope):** Reframed from temporal drift detection (which tests the calibration loop, not the aggregation function) to "calibration decomposability" -- a structural property of a single invocation verifiable in Session 1.
2. **W2 (Scenario 5 pass criterion):** Replaced subjective "matches domain-expert evidential weight" with verifiable criterion: CDF-normalized scores in [0.3, 0.99] with stable ranking across candidates.
3. **W3 (Fisher-UP r_i specification):** Added explicit derivation: `r_i = min(1, n_i / n_ref) * indicator(uncertainty != NoUncertainty)` with `r_floor` fallback, paralleling w_default in Candidate 1.
4. **S1 (Custom DivergenceKind):** Added configurable sigmoid with required parameters; exclusion + warning if absent.
5. **S3 (LogSumExp bounding):** Changed Candidate 2 default to hard max for guaranteed [0,1]; LogSumExp documented as alternative requiring `1 - exp(-LSE)` normalization.
6. **S5 (EffectDirection):** Added explicit handling: agreement metrics invert the normalized score; absent direction treated as unsigned contradiction.
7. **S6 (Status update):** Added instruction to update FINDINGS.md status from "NOT STARTED" to "IN PROGRESS".
8. **S4 (sample_size precedence):** Added clarification: MetricComponent.sample_size is primary, PointUncertainty.sample_size is fallback.

### Remaining Suggestions
- **S2 (IntervalEstimate definition):** Addressed by adding narrowing note that `IntervalEstimate` is opaque for Session 1; only `standard_error` and `sample_size` consumed.
- **S7 (Scenario 1 direction):** Incorporated into pass criterion -- aggregate should decrease or remain stable, direction documented.
- **S8 (Narrowing clarification):** Sharpened to "existing 5 items in the Next Steps section."

## Usage Notes

- **Best used with:** Claude Opus or Sonnet for implementation; the mathematical specifications are precise enough for direct code generation
- **Adjust for:** The `w_default`, `eps`, `alpha`, `tau`, `c_floor`, `r_floor`, `n_ref` parameters are intentionally left as configurables -- Session 2 will sweep these
