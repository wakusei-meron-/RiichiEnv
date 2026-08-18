use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

create_exception!(_riichienv, CalcError, PyValueError);

mod env;

#[pyfunction]
#[pyo3(name = "calculate_score", signature = (han, fu, is_oya, is_tsumo, honba, num_players=4))]
fn calculate_score_py(
    han: u8,
    fu: u8,
    is_oya: bool,
    is_tsumo: bool,
    honba: u32,
    num_players: u8,
) -> riichienv_core::score::Score {
    riichienv_core::score::calculate_score(han, fu, is_oya, is_tsumo, honba, num_players)
}

#[pyfunction]
#[pyo3(name = "parse_hand")]
fn parse_hand_py(text: &str) -> PyResult<(Vec<u32>, Vec<riichienv_core::types::Meld>)> {
    riichienv_core::parser::parse_hand(text).map_err(Into::into)
}

#[pyfunction]
#[pyo3(name = "parse_tile")]
fn parse_tile_py(text: &str) -> PyResult<u8> {
    riichienv_core::parser::parse_tile(text).map_err(Into::into)
}

#[pyfunction]
#[pyo3(name = "check_riichi_candidates")]
fn check_riichi_candidates_py(tiles_136: Vec<u8>) -> Vec<u32> {
    riichienv_core::check_riichi_candidates(tiles_136)
}

#[pyfunction]
#[pyo3(name = "calculate_shanten")]
fn calculate_shanten_py(hand_tiles: Vec<u32>) -> i32 {
    riichienv_core::shanten::calculate_shanten(&hand_tiles)
}

#[pyfunction]
#[pyo3(name = "calculate_shanten_3p")]
fn calculate_shanten_3p_py(hand_tiles: Vec<u32>) -> i32 {
    riichienv_core::shanten::calculate_shanten_3p(&hand_tiles)
}

fn py_calc_error(error: riichienv_calc::CalcError) -> PyErr {
    let message = error.to_string();
    let code = error.code();
    let details = serde_json::to_string(&error).unwrap_or_else(|serialization_error| {
        format!("{{\"serialization_error\":\"{serialization_error}\"}}")
    });
    let py_error = CalcError::new_err(message);
    Python::attach(|py| {
        let value = py_error.value(py);
        value
            .setattr("code", code)
            .expect("CalcError must allow a code attribute");
        value
            .setattr("details", details)
            .expect("CalcError must allow a details attribute");
    });
    py_error
}

fn calc_input(
    counts34: Vec<u8>,
    meld_count: u8,
    unavailable_counts34: Vec<u8>,
    variant: &str,
    contract_version: String,
) -> PyResult<riichienv_calc::CalculationInput> {
    let variant = match variant {
        "yonma" => riichienv_calc::Variant::Yonma,
        "sanma" => riichienv_calc::Variant::Sanma,
        _ => return Err(py_calc_error(riichienv_calc::CalcError::UnsupportedVariant)),
    };
    Ok(riichienv_calc::CalculationInput {
        variant,
        counts34,
        meld_count,
        unavailable_counts34,
        contract_version,
    })
}

fn calc_json<T: serde::Serialize>(
    result: Result<T, riichienv_calc::CalcError>,
) -> PyResult<String> {
    let value = result.map_err(py_calc_error)?;
    serde_json::to_string(&value).map_err(|error| {
        py_calc_error(riichienv_calc::CalcError::SerializationError {
            message: error.to_string(),
        })
    })
}

/// Strict four-player calculation contract.  Output is canonical JSON so the
/// Python and WASM bindings expose byte-equivalent payloads.
#[pyfunction]
#[pyo3(signature = (counts34, meld_count, unavailable_counts34, variant, contract_version))]
fn calculate_shanten34(
    counts34: Vec<u8>,
    meld_count: u8,
    unavailable_counts34: Vec<u8>,
    variant: &str,
    contract_version: String,
) -> PyResult<String> {
    calc_json(riichienv_calc::calculate_shanten34(&calc_input(
        counts34,
        meld_count,
        unavailable_counts34,
        variant,
        contract_version,
    )?))
}

#[pyfunction]
#[pyo3(signature = (counts34, meld_count, unavailable_counts34, variant, contract_version))]
fn analyze_draws34(
    counts34: Vec<u8>,
    meld_count: u8,
    unavailable_counts34: Vec<u8>,
    variant: &str,
    contract_version: String,
) -> PyResult<String> {
    calc_json(riichienv_calc::analyze_draws34(&calc_input(
        counts34,
        meld_count,
        unavailable_counts34,
        variant,
        contract_version,
    )?))
}

#[pyfunction]
#[pyo3(signature = (counts34, meld_count, unavailable_counts34, variant, contract_version))]
fn analyze_discards34(
    counts34: Vec<u8>,
    meld_count: u8,
    unavailable_counts34: Vec<u8>,
    variant: &str,
    contract_version: String,
) -> PyResult<String> {
    calc_json(riichienv_calc::analyze_discards34(&calc_input(
        counts34,
        meld_count,
        unavailable_counts34,
        variant,
        contract_version,
    )?))
}

#[pyfunction]
fn calculate_batch34(requests_json: &str) -> PyResult<String> {
    let requests: Vec<riichienv_calc::BatchRequest> =
        serde_json::from_str(requests_json).map_err(|error| {
            py_calc_error(riichienv_calc::CalcError::SerializationError {
                message: error.to_string(),
            })
        })?;
    calc_json(riichienv_calc::calculate_batch34(&requests))
}

#[pyfunction]
fn calc_core_metadata() -> PyResult<String> {
    serde_json::to_string(&riichienv_calc::core_metadata()).map_err(|error| {
        py_calc_error(riichienv_calc::CalcError::SerializationError {
            message: error.to_string(),
        })
    })
}

#[pymodule]
fn _riichienv(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("CalcError", _py.get_type::<CalcError>())?;
    m.add_class::<riichienv_core::types::Meld>()?;
    m.add_class::<riichienv_core::types::MeldType>()?;
    m.add_class::<riichienv_core::types::Wind>()?;
    m.add_class::<riichienv_core::types::Conditions>()?;
    m.add_class::<riichienv_core::types::WinResult>()?;
    m.add_class::<riichienv_core::score::Score>()?;
    m.add_class::<riichienv_core::hand_evaluator::HandEvaluator>()?;
    m.add_class::<riichienv_core::hand_evaluator_3p::HandEvaluator3P>()?;
    m.add_class::<riichienv_core::replay::MjSoulReplay>()?;
    m.add_class::<riichienv_core::replay::MjaiReplay>()?;
    m.add_class::<riichienv_core::replay::LogKyoku>()?;
    m.add_class::<riichienv_core::replay::mjsoul_replay::KyokuIterator>()?;
    m.add_class::<riichienv_core::replay::WinResultContext>()?;
    m.add_class::<riichienv_core::replay::WinResultContextIterator>()?;
    m.add_class::<riichienv_core::rule::GameRule>()?;
    m.add_class::<riichienv_core::yaku::Yaku>()?;

    // Env classes
    m.add_class::<riichienv_core::action::ActionType>()?;
    m.add_class::<riichienv_core::action::Phase>()?;
    m.add_class::<riichienv_core::action::Action>()?;
    m.add_class::<riichienv_core::action::Action3P>()?;
    m.add_class::<riichienv_core::observation::Observation>()?;
    m.add_class::<riichienv_core::observation_3p::Observation3P>()?;
    m.add_class::<env::RiichiEnv>()?;

    m.add_function(wrap_pyfunction!(calculate_score_py, m)?)?;
    m.add_function(wrap_pyfunction!(parse_hand_py, m)?)?;
    m.add_function(wrap_pyfunction!(parse_tile_py, m)?)?;
    m.add_function(wrap_pyfunction!(check_riichi_candidates_py, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_shanten_py, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_shanten_3p_py, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_shanten34, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_draws34, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_discards34, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_batch34, m)?)?;
    m.add_function(wrap_pyfunction!(calc_core_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(
        riichienv_core::yaku::get_yaku_by_id_py,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(riichienv_core::yaku::get_all_yaku_py, m)?)?;
    Ok(())
}
