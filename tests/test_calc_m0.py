import json

import pytest

from riichienv import CalcError, analyze_draws34, calculate_shanten34


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
