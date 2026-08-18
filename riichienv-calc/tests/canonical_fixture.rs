use riichienv_calc::{CalculationInput, analyze_discards34, analyze_draws34, calculate_shanten34};
use serde_json::Value;

#[test]
fn canonical_fixture_executes_every_scalar_case() {
    let fixture: Value = serde_json::from_str(include_str!("../../fixtures/calc-m0-v1.json"))
        .expect("canonical fixture must be valid JSON");
    for case in fixture["cases"].as_array().expect("cases must be an array") {
        let name = case["name"].as_str().expect("case name");
        let input: CalculationInput =
            serde_json::from_value(case["input"].clone()).expect("calculation input");
        let result = match case["operation"].as_str().expect("operation") {
            "calculate_shanten34" => calculate_shanten34(&input)
                .and_then(|value| serde_json::to_value(value).map_err(serialization_error)),
            "analyze_draws34" => analyze_draws34(&input)
                .and_then(|value| serde_json::to_value(value).map_err(serialization_error)),
            "analyze_discards34" => analyze_discards34(&input)
                .and_then(|value| serde_json::to_value(value).map_err(serialization_error)),
            operation => panic!("unknown operation {operation} in {name}"),
        };
        if let Some(expected_code) = case.get("expect_error").and_then(Value::as_str) {
            assert_eq!(
                result.expect_err(name).code(),
                expected_code,
                "error code for {name}"
            );
            continue;
        }
        let result = result.unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_partial_expectations(name, &result, case.get("expect"));
    }
}

fn serialization_error(error: serde_json::Error) -> riichienv_calc::CalcError {
    riichienv_calc::CalcError::SerializationError {
        message: error.to_string(),
    }
}

fn assert_partial_expectations(name: &str, result: &Value, expected: Option<&Value>) {
    let Some(expected) = expected.and_then(Value::as_object) else {
        return;
    };
    for (field, value) in expected {
        let actual = match field.as_str() {
            "minimum_shanten" => result
                .pointer("/shanten/minimum")
                .or_else(|| result.get("minimum"))
                .cloned(),
            "improving_tiles34" => Some(Value::Array(tile_ids(result, "improving_tiles"))),
            "agari_tiles34" => Some(Value::Array(tile_ids(result, "agari_tiles"))),
            field => result.get(field).cloned(),
        };
        assert_eq!(
            actual.as_ref(),
            Some(value),
            "{name}: expectation for {field}"
        );
    }
}

fn tile_ids(result: &Value, field: &str) -> Vec<Value> {
    result[field]
        .as_array()
        .expect("tile analysis array")
        .iter()
        .map(|tile| tile["tile34"].clone())
        .collect()
}
