# Regime Validity Analysis (Session 4.2)

## Scope

Question: do remaining failures occur in realistic operating regimes for ATHENA's DSL targets (OpenMM, GROMACS, CESM, VASP), or only in stress-test extremes?

Evidence inputs:
- `perturbation_summary.md`
- `stretch_summary.md`
- `ceiling_analysis.md`
- `research/adversarial-reward/FINDINGS.md` (Sessions 4, 4.1, 5)
- `normalization.py`
- `scenarios.py`

Confidence labels:
- HIGH: well-established behavior in standard computational-science practice.
- MEDIUM: reasonable domain inference, but not tied to a single canonical benchmark.
- LOW: uncertain; requires domain-expert validation.

## Realistic Parameter Ranges (DSL-Informed)

These are training-knowledge-informed operating ranges for typical, quality-controlled DSL workflows, not hard physical limits.

| parameter | realistic_min | realistic_max | confidence | source/rationale |
| :--- | ---: | ---: | :---: | :--- |
| `|z|` (Z-score / standardized effect axis) | 0.0 | 6.0 | HIGH | In OpenMM/GROMACS, CESM, and VASP post-processing, most accepted runs cluster in low-to-moderate standardized residuals; `|z|>6` is usually an outlier or run-quality issue, not nominal behavior. |
| Bayes factor (`BF`) | 1 | 1000 (common band 1-300) | MEDIUM | Bayesian model/hypothesis comparisons in simulation workflows commonly produce weak-to-strong evidence in low hundreds; values in high hundreds/low thousands can occur with strong cumulative evidence. |
| Custom sigmoid midpoint (`x0`) for `DivergenceKind.Custom` | 0.0 | 0.5 | MEDIUM | In `normalization.py`, custom metrics map via `sigmoid(value, k, x0)`. For contradiction-oriented nonnegative custom inputs (as in `scenarios.py` defaults), negative `x0` implies 0.5 activation before zero evidence and over-amplifies weak signal. |
| Custom sigmoid steepness (`k`) for `DivergenceKind.Custom` | 0.8 | 3.0 | MEDIUM | Moderate slopes preserve gradation for weak-to-moderate contradiction evidence; very high slopes act like hard thresholds and are usually calibration/stress settings. |
| Standard-error multiplier vs baseline (`SE_mult`) | 0.5 | 3.0 | MEDIUM | In stable simulation campaigns, uncertainty inflation by 2-3x can happen due sampling/replicate variation; 5-10x typically indicates under-sampling, non-equilibrated windows, or run-quality failure rather than normal operation. |
| Missing uncertainty count (per 4-metric bundle) | 0 | 2 | MEDIUM | DSL pipelines can drop uncertainty fields for some diagnostics, but fully missing uncertainty payloads across most metrics are usually QA/reporting failures, not routine production state. |
| Abrupt single-metric regime jump factor (Pattern B style) | 1.0x | 5.0x | MEDIUM | Physically meaningful regime shifts generally move multiple coupled observables; isolated one-metric jumps at extreme factors are usually artifacts or invalid runs. |

## Domain Check for Pattern B 50x Single-Metric Jump

Pattern B in `calibration_sim.py` scales one metric from `0.1` to `5.0` at cycle 25 while all others remain at `0.1` (a 50x isolated jump).

| domain | known abrupt phenomena | 50x isolated one-metric jump in valid runs? | confidence | rationale |
| :--- | :--- | :---: | :---: | :--- |
| OpenMM / GROMACS | Conformational transitions, rare events, phase transitions | No (out-of-range) | MEDIUM | Large transitions typically co-move multiple observables (energy, structure, contacts); isolated 50x one-metric jump is more consistent with instrumentation or setup failure. |
| CESM | Regime shifts (e.g., circulation/oscillation changes), extreme anomalies | No (out-of-range) | MEDIUM | Climate regime changes are coupled and temporally distributed; instantaneous 50x jump in a single diagnostic while peers stay flat is not standard physically valid behavior. |
| VASP | Electronic/structural transitions, SCF instability modes | No (out-of-range for accepted outputs) | MEDIUM | Extreme spikes can occur in non-converged SCF iterations, but those are run failures and normally excluded from validated scientific outputs. |

## Failure Boundary Overlay

| failure_mode | boundary_value | realistic_range | classification | rationale |
| :--- | :--- | :--- | :---: | :--- |
| S2 sigmoid fragility | `x0=-0.2` with `k>=2.0` fails (`perturbation_summary.md`) | `x0 in [0.0, 0.5]`, `k in [0.8, 3.0]` | Out-of-range | Negative `x0` sits outside the realistic custom-contradiction midpoint range; all sampled `x0>=0` points pass in Session 4 grid. |
| Pattern B under-response | 50x isolated jump (`0.1 -> 5.0` on one metric), observed `step_ratio=1.029` vs threshold `>3.0` (`stretch_summary.md`) | isolated abrupt jump typically `<=5x` in valid DSL outputs | Out-of-range | The stress pattern is intentionally extreme and not representative of physically valid single-metric behavior across target DSLs. |
| S1 SE multiplier fragility | fail at `SE_mult=5.0` and `10.0` (`perturbation_summary.md`) | `SE_mult in [0.5, 3.0]` | Out-of-range | Failures appear only under severe uncertainty inflation beyond normal quality-controlled operating bands. |
| S5 BF ceiling | old mapping failed for `BF>=120` (`perturbation_summary.md`) | `BF` commonly spans into this region | Resolved | Session 4.1 adopted log-scaled BF normalization (`bf_max_target=10000`), restoring positive S5 margins through BF=1000 (`ceiling_analysis.md`, `stretch_summary.md`). |
| S6 joint compression | old mapping failed at high `d_mid` with high `bf_strong` (`perturbation_summary.md`) | overlap with realistic high-evidence corners possible | Resolved | Same Session 4.1 BF normalization recovered all five prior failing S6 cells (`ceiling_analysis.md` Section 4). |

## Per-Failure Narrative

### S2 custom sigmoid (`x0=-0.2`, `k>=2.0`)

`normalization.py` uses `config.custom_sigmoids[method_ref]` only for `DivergenceKind.Custom`. In `scenarios.py`, default custom entries are nonnegative-midpoint (`s2.custom.1: x0=0.0`, `s6.custom.1: x0=0.3`). The failing locus is a negative midpoint stress setting, not a representative DSL operating point. This is still guardrail-worthy because a misconfigured config can re-enter this region.

### Pattern B sudden regime change

Pattern B is a deliberately adversarial calibration probe. Its 50x isolated jump is a strong stressor, but not a typical physically valid event in OpenMM/GROMACS/CESM/VASP production analyses. Classification: accepted out-of-range boundary, not a blocking architectural defect.

### S1 SE multiplier fragility

Hybrid behavior is stable through `SE_mult=3.0` and fails only at `5.0` and `10.0`. These multipliers exceed realistic uncertainty inflation in standard DSL campaigns and map better to low-quality/invalid runs.

### S5/S6 BF-related failures

These were real in-range concerns under old BF normalization and are now addressed by Session 4.1's log-scaled mapping (`bf_max_target=10000`). No additional guardrail is required for S5/S6 in this session.

## Verdict

- Remaining unresolved failures are in stress-test extremes, not realistic DSL operating ranges.
- One architectural guardrail is still required to prevent accidental entry into the known S2 failure region: enforce `x0 >= 0` for all custom sigmoid parameters.
- Pattern B and S1 extreme-SE behavior are accepted limitations with explicit boundaries, not blockers for Session 6 recommendation work.

Session outcome for `athena-17c`: complete.
