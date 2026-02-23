import json, sys
from pathlib import Path

from models import DivergenceKind, SigmoidParams
from normalization import BF_NORM_LOG_SCALED_C, NormalizationConfig
from scenarios import DEFAULT_CUSTOM_SIGMOIDS, build_scenario_fixtures


CONTRACT_PATH = Path(__file__).resolve().parent / "aggregate_score_recommendation.json"
CORRELATION_RESULTS_PATH = Path(__file__).resolve().parent / "correlation_results.json"
TRIGGER_THRESHOLD_CORRELATION_INFLATION = 1.5


def _load_contract_metadata() -> dict[str, object]:
    with CONTRACT_PATH.open("r", encoding="utf-8") as handle:
        payload = json.load(handle)

    recommendation = payload["recommendation"]
    parameters = payload["parameters"]
    normalization_params = parameters["normalization_config"]["absolute_difference_sigmoid"]

    return {
        "recommendation_version": recommendation["version"],
        "recommendation_status": recommendation["status"],
        "recommendation_date": recommendation["date"],
        "bf_normalization_c": parameters["bf_normalization"]["c"],
        "hybrid_n_terms": parameters["hybrid_config"]["n_terms"],
        "abs_diff_sigmoid_k": normalization_params["k"],
        "abs_diff_sigmoid_x0": normalization_params["x0"],
        "guardrails": payload["guardrails"],
        "operating_envelope": payload["operating_envelope"],
        "accepted_limitations": payload["accepted_limitations"],
        "revisit_triggers": payload["revisit_triggers"],
    }


CONTRACT = _load_contract_metadata()
RECOMMENDATION_VERSION = str(CONTRACT["recommendation_version"])
RECOMMENDATION_STATUS = str(CONTRACT["recommendation_status"])
RECOMMENDATION_DATE = str(CONTRACT["recommendation_date"])
CONTRACT_BF_C = float(CONTRACT["bf_normalization_c"])
CONTRACT_HYBRID_N_TERMS = int(CONTRACT["hybrid_n_terms"])
CONTRACT_ABS_DIFF_K = float(CONTRACT["abs_diff_sigmoid_k"])
CONTRACT_ABS_DIFF_X0 = float(CONTRACT["abs_diff_sigmoid_x0"])
GUARDRAILS = list(CONTRACT["guardrails"])
OPERATING_ENVELOPE = dict(CONTRACT["operating_envelope"])
ACCEPTED_LIMITATIONS = list(CONTRACT["accepted_limitations"])
REVISIT_TRIGGERS = list(CONTRACT["revisit_triggers"])
REVISIT_TRIGGER_IDS = {
    str(entry["id"]) for entry in REVISIT_TRIGGERS if isinstance(entry, dict) and "id" in entry
}


def _format_float(value: float) -> str:
    return f"{value:.6f}"


def _format_items(items: list[str]) -> str:
    if not items:
        return "none"
    return ", ".join(items)


def _get_guardrail_entry(guardrail_id: str) -> dict[str, object] | None:
    for guardrail in GUARDRAILS:
        if isinstance(guardrail, dict) and guardrail.get("id") == guardrail_id:
            return guardrail
    return None


def _extract_inflation_ratios(payload: object) -> list[float]:
    ratios: list[float] = []
    entries: list[object] = []
    if isinstance(payload, dict) and isinstance(payload.get("results"), list):
        entries = list(payload["results"])
    elif isinstance(payload, list):
        entries = list(payload)

    for entry in entries:
        if isinstance(entry, dict) and "inflation_ratio" in entry:
            ratios.append(float(entry["inflation_ratio"]))

    if not ratios and isinstance(payload, dict) and "inflation_ratio" in payload:
        ratios.append(float(payload["inflation_ratio"]))
    return ratios


def check_t1_operating_envelope() -> list[tuple[str, bool, str]]:
    results: list[tuple[str, bool, str]] = []

    bf_delta = abs(BF_NORM_LOG_SCALED_C - CONTRACT_BF_C)
    bf_passed = bf_delta <= 1e-8
    results.append(
        (
            "t1.bf_norm_c_matches_contract",
            bf_passed,
            f"code={_format_float(BF_NORM_LOG_SCALED_C)}, contract={_format_float(CONTRACT_BF_C)}, delta={bf_delta:.3e}",
        )
    )

    default_cfg = NormalizationConfig()
    observed_k = float(default_cfg.absolute_difference_sigmoid.k)
    observed_x0 = float(default_cfg.absolute_difference_sigmoid.x0)

    k_delta = abs(observed_k - CONTRACT_ABS_DIFF_K)
    x0_delta = abs(observed_x0 - CONTRACT_ABS_DIFF_X0)
    default_sigmoid_passed = k_delta <= 1e-10 and x0_delta <= 1e-10
    results.append(
        (
            "t1.abs_diff_default_sigmoid_matches_contract",
            default_sigmoid_passed,
            (
                f"k_observed={observed_k:.10f}, k_contract={CONTRACT_ABS_DIFF_K:.10f}, k_delta={k_delta:.3e}; "
                f"x0_observed={observed_x0:.10f}, x0_contract={CONTRACT_ABS_DIFF_X0:.10f}, x0_delta={x0_delta:.3e}"
            ),
        )
    )

    x0_min = float(OPERATING_ENVELOPE["custom_sigmoid_x0"]["validated_min"])
    x0_max = float(OPERATING_ENVELOPE["custom_sigmoid_x0"]["validated_max"])
    k_min = float(OPERATING_ENVELOPE["custom_sigmoid_k"]["validated_min"])
    k_max = float(OPERATING_ENVELOPE["custom_sigmoid_k"]["validated_max"])

    x0_out_of_range: list[str] = []
    k_out_of_range: list[str] = []
    for method_ref in sorted(DEFAULT_CUSTOM_SIGMOIDS):
        params: SigmoidParams = DEFAULT_CUSTOM_SIGMOIDS[method_ref]
        if not (x0_min <= params.x0 <= x0_max):
            x0_out_of_range.append(f"{method_ref}={params.x0}")
        if not (k_min <= params.k <= k_max):
            k_out_of_range.append(f"{method_ref}={params.k}")

    x0_envelope_passed = len(x0_out_of_range) == 0
    results.append(
        (
            "t1.custom_sigmoid_x0_within_envelope",
            x0_envelope_passed,
            (
                f"range=[{_format_float(x0_min)}, {_format_float(x0_max)}], "
                f"violations={_format_items(x0_out_of_range)}"
            ),
        )
    )

    k_envelope_passed = len(k_out_of_range) == 0
    results.append(
        (
            "t1.custom_sigmoid_k_within_envelope",
            k_envelope_passed,
            (
                f"range=[{_format_float(k_min)}, {_format_float(k_max)}], "
                f"violations={_format_items(k_out_of_range)}"
            ),
        )
    )

    return results


def check_t2_divergence_coverage() -> list[tuple[str, bool, str]]:
    covered_kinds = {
        "AbsoluteDifference",
        "ZScore",
        "BayesFactor",
        "KLDivergence",
        "EffectSize",
        "Custom",
    }
    results: list[tuple[str, bool, str]] = []
    for kind in DivergenceKind:
        kind_name = kind.name
        covered = kind_name in covered_kinds
        detail = (
            "covered by normalize_component dispatch"
            if covered
            else "missing normalize_component branch for this kind"
        )
        results.append((f"t2.kind.{kind_name}", covered, detail))
    return results


def check_t3_pattern_b_metadata() -> tuple[str, bool, str]:
    entry_l1 = next(
        (
            entry
            for entry in ACCEPTED_LIMITATIONS
            if isinstance(entry, dict) and entry.get("id") == "L1"
        ),
        None,
    )
    if entry_l1 is None:
        return ("t3.pattern_b_l1_metadata", False, "accepted_limitations entry L1 is missing")

    classification = str(entry_l1.get("classification"))
    try:
        observed = float(entry_l1["observed_value"])
        threshold = float(entry_l1["threshold"])
    except (KeyError, TypeError, ValueError):
        return (
            "t3.pattern_b_l1_metadata",
            False,
            "L1 metadata is missing numeric observed_value/threshold",
        )

    passed = classification == "out_of_range" and observed < threshold
    detail = (
        f"classification={classification}, observed_value={observed:.6f}, "
        f"threshold={threshold:.6f}, observed_lt_threshold={observed < threshold}"
    )
    return ("t3.pattern_b_l1_metadata", passed, detail)


def check_t4_scenario_coverage() -> list[tuple[str, bool, str]]:
    fixtures = build_scenario_fixtures()
    observed_indices = {fixture.idx for fixture in fixtures}
    baseline_indices = {1, 2, 3, 4, 5, 6, 7}

    new_indices = sorted(observed_indices - baseline_indices)
    missing_indices = sorted(baseline_indices - observed_indices)

    return [
        (
            "t4.no_new_fixture_indices",
            len(new_indices) == 0,
            f"new_indices={new_indices if new_indices else 'none'}",
        ),
        (
            "t4.no_missing_baseline_indices",
            len(missing_indices) == 0,
            f"missing_indices={missing_indices if missing_indices else 'none'}",
        ),
    ]


def check_t5_correlation_envelope() -> list[tuple[str, bool, str]]:
    results: list[tuple[str, bool, str]] = []

    correlation_envelope = OPERATING_ENVELOPE["correlation_rho"]
    max_inflation = float(correlation_envelope["max_inflation"])
    envelope_passed = max_inflation < TRIGGER_THRESHOLD_CORRELATION_INFLATION
    results.append(
        (
            "t5.contract_correlation_max_inflation_below_threshold",
            envelope_passed,
            (
                f"contract_max_inflation={max_inflation:.6f}, "
                f"threshold={TRIGGER_THRESHOLD_CORRELATION_INFLATION:.1f}"
            ),
        )
    )

    if not CORRELATION_RESULTS_PATH.exists():
        results.append(
            (
                "t5.runtime_inflation_ratio_file_check",
                True,
                "correlation_results.json not present; runtime inflation check skipped",
            )
        )
        return results

    try:
        with CORRELATION_RESULTS_PATH.open("r", encoding="utf-8") as handle:
            correlation_payload = json.load(handle)
    except json.JSONDecodeError as exc:
        results.append(
            (
                "t5.runtime_inflation_ratio_file_check",
                False,
                f"invalid JSON in correlation_results.json: {exc.msg}",
            )
        )
        return results

    try:
        ratios = _extract_inflation_ratios(correlation_payload)
    except (TypeError, ValueError):
        results.append(
            (
                "t5.runtime_inflation_ratio_file_check",
                False,
                "correlation_results.json contains non-numeric inflation_ratio values",
            )
        )
        return results

    if not ratios:
        results.append(
            (
                "t5.runtime_inflation_ratio_file_check",
                False,
                "no inflation_ratio values found in correlation_results.json",
            )
        )
        return results

    max_ratio = max(ratios)
    ratios_passed = max_ratio <= TRIGGER_THRESHOLD_CORRELATION_INFLATION
    results.append(
        (
            "t5.runtime_inflation_ratio_file_check",
            ratios_passed,
            (
                f"max_observed_inflation_ratio={max_ratio:.6f}, "
                f"threshold={TRIGGER_THRESHOLD_CORRELATION_INFLATION:.1f}, "
                f"samples={len(ratios)}"
            ),
        )
    )
    return results


def check_contract_metadata() -> list[tuple[str, bool, str]]:
    results: list[tuple[str, bool, str]] = []

    results.append(
        (
            "meta.recommendation_version_locked",
            RECOMMENDATION_VERSION == "1.0",
            f"observed={RECOMMENDATION_VERSION}, expected=1.0",
        )
    )
    results.append(
        (
            "meta.recommendation_status_locked",
            RECOMMENDATION_STATUS == "LOCKED",
            f"observed={RECOMMENDATION_STATUS}, expected=LOCKED",
        )
    )

    c_delta = abs(CONTRACT_BF_C - BF_NORM_LOG_SCALED_C)
    results.append(
        (
            "meta.bf_norm_c_code_json_parity",
            c_delta <= 1e-8,
            f"code={BF_NORM_LOG_SCALED_C:.6f}, contract={CONTRACT_BF_C:.6f}, delta={c_delta:.3e}",
        )
    )

    results.append(
        (
            "meta.hybrid_n_terms_locked",
            CONTRACT_HYBRID_N_TERMS == 1,
            f"observed={CONTRACT_HYBRID_N_TERMS}, expected=1",
        )
    )

    guardrail_id = "GR-S2-CUSTOM-SIGMOID-X0-NONNEG"
    guardrail_entry = _get_guardrail_entry(guardrail_id)
    guardrail_passed = (
        guardrail_entry is not None
        and guardrail_entry.get("enforcement") == "reject_at_config_construction"
    )
    enforcement_value = (
        str(guardrail_entry.get("enforcement"))
        if guardrail_entry is not None
        else "missing"
    )
    results.append(
        (
            "meta.guardrail_enforcement_mode",
            guardrail_passed,
            f"id={guardrail_id}, enforcement={enforcement_value}",
        )
    )
    return results


def main() -> None:
    section_checks: list[tuple[str, str, list[tuple[str, bool, str]]]] = [
        ("T1", "Operating Envelope", check_t1_operating_envelope()),
        ("T2", "Divergence Coverage", check_t2_divergence_coverage()),
        ("T3", "Pattern B Metadata", [check_t3_pattern_b_metadata()]),
        ("T4", "Scenario Coverage", check_t4_scenario_coverage()),
        ("T5", "Correlation Envelope", check_t5_correlation_envelope()),
        ("META", "Contract Metadata", check_contract_metadata()),
    ]

    all_results: list[tuple[str, str, bool, str]] = []
    passed_count = 0

    for trigger_id, section_name, checks in section_checks:
        print(f"--- {trigger_id}: {section_name} ---")
        for check_name, passed, detail in checks:
            status = "PASS" if passed else "FAIL"
            print(f"{status} {check_name}: {detail}")
            all_results.append((trigger_id, check_name, passed, detail))
            if passed:
                passed_count += 1

    total_checks = len(all_results)
    print(f"{passed_count}/{total_checks} checks passed")

    failures = [(trigger_id, check_name) for trigger_id, check_name, passed, _ in all_results if not passed]
    if failures:
        print("--- TRIGGER ALERT ---")
        trigger_actions = {
            "T1": "open bead tagged revisit-T1; rerun regime_validity with empirical distributions",
            "T2": "open bead tagged revisit-T2; implement normalization for new kind and rerun baseline/perturbation/acceptance",
            "T3": "open bead tagged revisit-T3; run targeted Pattern B recovery analysis for calibration requirements",
            "T4": "open bead tagged revisit-T4; rerun full evaluation on expanded scenario suite",
            "T5": "open bead tagged revisit-T5; rerun correlation robustness probes and review correction strategy",
            "META": "open bead tagged revisit-contract-metadata; reconcile lockfile metadata and code constants",
        }

        for trigger_id, check_name in failures:
            if trigger_id in {"T1", "T2", "T3", "T4", "T5"} and trigger_id not in REVISIT_TRIGGER_IDS:
                action = (
                    f"contract revisit_triggers missing {trigger_id}; "
                    "open bead tagged revisit-contract-metadata; repair trigger registry"
                )
                print(f"META tripped ({check_name}): {action}")
                continue

            action = trigger_actions.get(trigger_id, trigger_actions["META"])
            print(f"{trigger_id} tripped ({check_name}): {action}")

    guardrail = _get_guardrail_entry("GR-S2-CUSTOM-SIGMOID-X0-NONNEG")
    guardrail_ok = (
        guardrail is not None and guardrail.get("enforcement") == "reject_at_config_construction"
    )
    print(
        "contract_metadata "
        f"version={RECOMMENDATION_VERSION} "
        f"bf_norm_c={CONTRACT_BF_C:.6f} "
        f"n_terms={CONTRACT_HYBRID_N_TERMS} "
        f"guardrail_status={'ok' if guardrail_ok else 'invalid'} "
        f"date={RECOMMENDATION_DATE}"
    )

    sys.exit(0 if passed_count == total_checks else 1)


if __name__ == "__main__":
    main()
