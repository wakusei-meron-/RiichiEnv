import json
from pathlib import Path

import pytest

from riichienv import (
    CalcError,
    analyze_discards34,
    analyze_draws34,
    calculate_batch34,
    calculate_shanten34,
)


def _tenpai_input() -> tuple[list[int], list[int]]:
    # 123m 456m 789m 12p 11z, waiting on 3p.
    counts = [0] * 34
    for tile in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27]:
        counts[tile] += 1
    return counts, counts.copy()


def test_m0_draw_analysis_uses_canonical_json_and_separate_agari_totals() -> None:
    counts, unavailable = _tenpai_input()
    result = json.loads(analyze_draws34(counts, 0, unavailable, "yonma", "m0-v1"))

    assert result["shanten"]["minimum"] == 0
    assert result["agari_tiles"] == result["improving_tiles"]
    assert result["agari_tiles"][0]["tile34"] == 11
    assert result["agari_tile_kinds"] == 1
    assert result["agari_tile_count"] == 4


def test_m0_python_error_exposes_machine_readable_code() -> None:
    counts, unavailable = _tenpai_input()
    unavailable[0] = 0

    with pytest.raises(CalcError) as raised:
        calculate_shanten34(counts, 0, unavailable, "yonma", "m0-v1")

    assert raised.value.code == "UNAVAILABLE_BELOW_HAND"
    assert "UNAVAILABLE_BELOW_HAND" in raised.value.details


@pytest.mark.parametrize(
    ("counts_value", "meld_count", "expected_code"),
    [
        (-1, 0, "COUNT_OUT_OF_RANGE"),
        (256, 0, "COUNT_OUT_OF_RANGE"),
        (0, -1, "MELD_COUNT_OUT_OF_RANGE"),
        (0, 256, "MELD_COUNT_OUT_OF_RANGE"),
    ],
)
def test_m0_python_range_errors_are_typed_before_internal_conversion(
    counts_value: int, meld_count: int, expected_code: str
) -> None:
    counts, unavailable = _tenpai_input()
    if counts_value:
        counts[0] = counts_value
        unavailable[0] = counts_value

    with pytest.raises(CalcError) as raised:
        calculate_shanten34(counts, meld_count, unavailable, "yonma", "m0-v1")

    assert raised.value.code == expected_code


def test_m0_python_executes_canonical_scalar_fixture() -> None:
    fixture = json.loads((Path(__file__).parents[1] / "fixtures/calc-m0-v1.json").read_text())
    functions = {
        "calculate_shanten34": calculate_shanten34,
        "analyze_draws34": analyze_draws34,
        "analyze_discards34": analyze_discards34,
    }
    for case in fixture["cases"]:
        input_ = case["input"]
        arguments = (
            input_["counts34"],
            input_["meld_count"],
            input_["unavailable_counts34"],
            input_["variant"],
            input_["contract_version"],
        )
        expected_error = case.get("expect_error")
        if expected_error:
            with pytest.raises(CalcError) as raised:
                functions[case["operation"]](*arguments)
            assert raised.value.code == expected_error, case["name"]
            continue
        result = json.loads(functions[case["operation"]](*arguments))
        _assert_partial_expectations(case["name"], result, case.get("expect", {}))


def _assert_partial_expectations(name: str, result: object, expected: dict[str, object]) -> None:
    assert isinstance(result, dict) or isinstance(result, list), name
    if not isinstance(result, dict):
        return
    for field, expected_value in expected.items():
        if field == "minimum_shanten":
            actual = result.get("shanten", result).get("minimum")
        elif field in {"improving_tiles34", "agari_tiles34"}:
            actual = [tile["tile34"] for tile in result[field.removesuffix("34")]]
        else:
            actual = result.get(field)
        assert actual == expected_value, f"{name}: {field}"


def test_m0_python_batch_preserves_order_and_exposes_item_error_code() -> None:
    counts, unavailable = _tenpai_input()
    input_ = {
        "variant": "yonma",
        "counts34": counts,
        "meld_count": 0,
        "unavailable_counts34": unavailable,
        "contract_version": "m0-v1",
    }
    requests = [
        {"id": "ok", "operation": "analyze_draws34", "input": input_},
        {
            "id": "bad",
            "operation": "analyze_draws34",
            "input": {**input_, "counts34": []},
        },
    ]
    result = json.loads(calculate_batch34(json.dumps(requests)))
    assert [item["id"] for item in result] == ["ok", "bad"]
    assert result[0]["result"]["kind"] == "draws"
    assert result[1]["error"]["code"] == "HAND_MELD_INCONSISTENT"

    with pytest.raises(CalcError) as raised:
        calculate_batch34(json.dumps([requests[0], requests[0]]))
    assert raised.value.code == "DUPLICATE_BATCH_ID"
