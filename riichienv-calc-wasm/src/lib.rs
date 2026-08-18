use riichienv_calc::{BatchRequest, CalcError, CalculationInput};
use wasm_bindgen::prelude::*;

fn js_error(error: CalcError) -> JsValue {
    let object = js_sys::Object::new();
    js_sys::Reflect::set(
        &object,
        &JsValue::from_str("code"),
        &JsValue::from_str(error.code()),
    )
    .expect("plain JS object must accept code");
    js_sys::Reflect::set(
        &object,
        &JsValue::from_str("message"),
        &JsValue::from_str(&error.to_string()),
    )
    .expect("plain JS object must accept message");
    let details = serde_wasm_bindgen::to_value(&error)
        .unwrap_or_else(|serialization_error| JsValue::from_str(&serialization_error.to_string()));
    js_sys::Reflect::set(&object, &JsValue::from_str("details"), &details)
        .expect("plain JS object must accept details");
    object.into()
}

fn serialization_error(error: impl std::fmt::Display) -> JsValue {
    js_error(CalcError::SerializationError {
        message: error.to_string(),
    })
}

fn parse_input(value: JsValue) -> Result<CalculationInput, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(serialization_error)
}

fn serialize<T: serde::Serialize>(value: T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&value).map_err(serialization_error)
}

/// Works with both wasm-pack `--target web` and `--target nodejs`; initialization
/// is provided by the generated wasm-bindgen entrypoint and there is no JS fallback.
#[wasm_bindgen]
pub fn calculate_shanten34(input: JsValue) -> Result<JsValue, JsValue> {
    serialize(riichienv_calc::calculate_shanten34(&parse_input(input)?).map_err(js_error)?)
}

#[wasm_bindgen]
pub fn analyze_draws34(input: JsValue) -> Result<JsValue, JsValue> {
    serialize(riichienv_calc::analyze_draws34(&parse_input(input)?).map_err(js_error)?)
}

#[wasm_bindgen]
pub fn analyze_discards34(input: JsValue) -> Result<JsValue, JsValue> {
    serialize(riichienv_calc::analyze_discards34(&parse_input(input)?).map_err(js_error)?)
}

#[wasm_bindgen]
pub fn calculate_batch34(input: JsValue) -> Result<JsValue, JsValue> {
    let requests: Vec<BatchRequest> =
        serde_wasm_bindgen::from_value(input).map_err(serialization_error)?;
    serialize(riichienv_calc::calculate_batch34(&requests).map_err(js_error)?)
}

#[wasm_bindgen]
pub fn calc_core_metadata() -> Result<JsValue, JsValue> {
    serialize(riichienv_calc::core_metadata())
}
