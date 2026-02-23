import json, sys
from pathlib import Path

from scenarios import DEFAULT_CUSTOM_SIGMOIDS, build_scenario_fixtures
from normalization import NormalizationConfig
from models import SigmoidParams
from candidates import HybridConfig, aggregate_hybrid
from evaluate import evaluate_fixture, ScenarioCellResult
from ceiling_analysis import margin_from_cell


SCENARIO_KEYS = {
    1: "S1_noisy_tv",
    2: "S2_unanimous_weak_signal",
    3: "S3_mixed_signal",
    4: "S4_missing_data",
    5: "S5_scale_heterogeneity",
    6: "S6_calibration_decomposability",
    7: "S7_boundary_seeking",
}


def test_margin_parity() -> list[tuple[str, bool, str]]:
    results: list[tuple[str, bool, str]] = []
    rec_path = Path(__file__).resolve().parent / "aggregate_score_recommendation.json"
    with rec_path.open("r", encoding="utf-8") as f:
        recommendation = json.load(f)
    baseline_margins = recommendation["baseline_margins"]

    fixtures = build_scenario_fixtures()
    norm_cfg = NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)
    hybrid_cfg = HybridConfig(normalization=norm_cfg)
    fn = lambda comps: aggregate_hybrid(comps, config=hybrid_cfg)

    for fixture in fixtures:
        scenario_key = SCENARIO_KEYS[fixture.idx]
        try:
            cell = evaluate_fixture(fixture, "Hybrid", fn)
            assert isinstance(cell, ScenarioCellResult)
            assert cell.passed is True, f"scenario gate failed: passed={cell.passed}"
            margin, _ = margin_from_cell(cell)
            expected = float(baseline_margins[scenario_key])
            delta = abs(margin - expected)
            assert delta <= 1e-6, f"margin delta {delta:.3e} > 1e-6"
            results.append(
                (
                    scenario_key,
                    True,
                    f"margin={margin:.9f}, expected={expected:.9f}, delta={delta:.3e}",
                )
            )
        except AssertionError as exc:
            results.append((scenario_key, False, str(exc)))

    return results


def test_guardrail_rejection() -> tuple[str, bool, str]:
    test_name = "GR-S2-CUSTOM-SIGMOID-X0-NONNEG"
    try:
        NormalizationConfig(custom_sigmoids={"test": SigmoidParams(k=2.0, x0=-0.2)})
    except ValueError as exc:
        msg = str(exc)
        if "GR-S2-CUSTOM-SIGMOID-X0-NONNEG" in msg:
            return test_name, True, msg
        return test_name, False, f"wrong ValueError message: {msg}"
    return test_name, False, "expected ValueError was not raised"


def test_decomposition_invariant() -> list[tuple[str, bool, str]]:
    results: list[tuple[str, bool, str]] = []
    fixtures = build_scenario_fixtures()
    norm_cfg = NormalizationConfig(custom_sigmoids=DEFAULT_CUSTOM_SIGMOIDS)
    hybrid_cfg = HybridConfig(normalization=norm_cfg)

    for fixture in fixtures:
        scenario_key = SCENARIO_KEYS[fixture.idx]
        ran_datasets: list[str] = []
        failed: RuntimeError | None = None
        failed_dataset = ""

        for dataset_key in fixture.datasets:
            try:
                aggregate_hybrid(fixture.datasets[dataset_key], config=hybrid_cfg)
                ran_datasets.append(dataset_key)
            except RuntimeError as exc:
                failed = exc
                failed_dataset = dataset_key
                break

        if failed is None:
            results.append((scenario_key, True, f"datasets={','.join(ran_datasets)}"))
        else:
            results.append((scenario_key, False, f"dataset={failed_dataset}: {failed}"))

    return results


def main() -> None:
    all_results: list[tuple[str, bool, str]] = []
    all_results.extend(test_margin_parity())
    all_results.append(test_guardrail_rejection())
    all_results.extend(test_decomposition_invariant())

    passed_count = 0
    total = len(all_results)
    for test_name, passed, detail in all_results:
        status = "PASS" if passed else "FAIL"
        print(f"{status} {test_name}: {detail}")
        if passed:
            passed_count += 1

    print(f"{passed_count}/{total} passed")
    print("contract_metadata version=1.0 bf_norm_c=0.083647 n_terms=1 guardrail_enabled=true")
    sys.exit(0 if passed_count == total else 1)


if __name__ == "__main__":
    main()
