//! Deterministic four-player tile34 calculation APIs.
//!
//! This crate deliberately accepts projected state, not game events.  Source
//! adapters own event reconstruction, red-five identity, and the provenance of
//! the unavailable-tile projection.  Invalid projections fail closed.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

mod nyanten;

/// Legacy entry points retained by `riichienv-core`.  They intentionally do
/// not form part of the versioned calculation contract.
pub mod legacy {
    pub use crate::nyanten::{
        calc_shanten_components_from_counts, calc_shanten_from_counts, calc_shanten_from_counts_3p,
        calc_standard_shanten_from_counts, calculate_shanten,
    };
    #[cfg(feature = "legacy-python")]
    pub use crate::nyanten::{
        calculate_best_ukeire, calculate_best_ukeire_3p, calculate_effective_tiles,
        calculate_effective_tiles_3p, calculate_effective_tiles_3p_with_discard,
        calculate_effective_tiles_with_discard, calculate_shanten_3p,
    };
}

pub const TILE_KIND_COUNT: usize = 34;
pub const CALCULATION_CONTRACT_VERSION: &str = "m0-v1";
pub const TILE_ENCODING_VERSION: &str = "tile34-v1";
pub const AVAILABILITY_CONTRACT_VERSION: &str = "unavailable-physical-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Yonma,
    Sanma,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalculationInput {
    pub variant: Variant,
    pub counts34: Vec<u8>,
    pub meld_count: u8,
    pub unavailable_counts34: Vec<u8>,
    pub contract_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreMetadata {
    pub core_version: String,
    pub core_git_sha: String,
    pub calculation_contract_version: String,
    pub tile_encoding_version: String,
    pub availability_contract_version: String,
}

pub fn core_metadata() -> CoreMetadata {
    CoreMetadata {
        core_version: env!("CARGO_PKG_VERSION").to_owned(),
        core_git_sha: option_env!("RIICHIENV_CORE_GIT_SHA")
            .unwrap_or("unknown")
            .to_owned(),
        calculation_contract_version: CALCULATION_CONTRACT_VERSION.to_owned(),
        tile_encoding_version: TILE_ENCODING_VERSION.to_owned(),
        availability_contract_version: AVAILABILITY_CONTRACT_VERSION.to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalcError {
    #[error("COUNT_OUT_OF_RANGE: {field}[{index}] must be between 0 and 4, got {value}")]
    CountOutOfRange {
        field: String,
        index: usize,
        value: u8,
    },
    #[error(
        "UNAVAILABLE_BELOW_HAND: unavailable_counts34[{index}] ({unavailable}) is below counts34 ({hand})"
    )]
    UnavailableBelowHand {
        index: usize,
        unavailable: u8,
        hand: u8,
    },
    #[error("HAND_MELD_INCONSISTENT: expected {expected} logical tiles, got {actual}")]
    HandMeldInconsistent { expected: u8, actual: u8 },
    #[error("MELD_COUNT_OUT_OF_RANGE: meld_count must be between 0 and 4, got {meld_count}")]
    MeldCountOutOfRange { meld_count: u8 },
    #[error("UNSUPPORTED_VARIANT: only yonma is supported by calculation contract m0-v1")]
    UnsupportedVariant,
    #[error("CONTRACT_VERSION_MISMATCH: expected {expected}, got {actual}")]
    ContractVersionMismatch { expected: String, actual: String },
    #[error("DUPLICATE_BATCH_ID: duplicate batch id {id}")]
    DuplicateBatchId { id: String },
    #[error("SERIALIZATION_ERROR: {message}")]
    SerializationError { message: String },
    #[error("CORE_INIT_FAILED: {message}")]
    CoreInitFailed { message: String },
}

impl CalcError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::CountOutOfRange { .. } => "COUNT_OUT_OF_RANGE",
            Self::UnavailableBelowHand { .. } => "UNAVAILABLE_BELOW_HAND",
            Self::HandMeldInconsistent { .. } => "HAND_MELD_INCONSISTENT",
            Self::MeldCountOutOfRange { .. } => "MELD_COUNT_OUT_OF_RANGE",
            Self::UnsupportedVariant => "UNSUPPORTED_VARIANT",
            Self::ContractVersionMismatch { .. } => "CONTRACT_VERSION_MISMATCH",
            Self::DuplicateBatchId { .. } => "DUPLICATE_BATCH_ID",
            Self::SerializationError { .. } => "SERIALIZATION_ERROR",
            Self::CoreInitFailed { .. } => "CORE_INIT_FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShantenResult {
    pub minimum: i8,
    pub standard: i8,
    pub chiitoitsu: Option<i8>,
    pub kokushi: Option<i8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileAnalysis {
    pub tile34: u8,
    pub remaining_count: u8,
    pub next_shanten: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawAnalysis {
    pub shanten: ShantenResult,
    pub improving_tiles: Vec<TileAnalysis>,
    pub agari_tiles: Vec<TileAnalysis>,
    pub improving_tile_kinds: u8,
    pub improving_tile_count: u16,
    pub agari_tile_kinds: u8,
    pub agari_tile_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscardAnalysis {
    pub discard_tile34: u8,
    pub draw_analysis: DrawAnalysis,
}

fn validate(
    input: &CalculationInput,
    expected_total: Option<u8>,
) -> Result<[u8; TILE_KIND_COUNT], CalcError> {
    if input.variant != Variant::Yonma {
        return Err(CalcError::UnsupportedVariant);
    }
    if input.contract_version != CALCULATION_CONTRACT_VERSION {
        return Err(CalcError::ContractVersionMismatch {
            expected: CALCULATION_CONTRACT_VERSION.to_owned(),
            actual: input.contract_version.clone(),
        });
    }
    if input.meld_count > 4 {
        return Err(CalcError::MeldCountOutOfRange {
            meld_count: input.meld_count,
        });
    }
    let counts = input.counts34.as_slice();
    let unavailable = input.unavailable_counts34.as_slice();
    if counts.len() != TILE_KIND_COUNT {
        return Err(CalcError::HandMeldInconsistent {
            expected: TILE_KIND_COUNT as u8,
            actual: counts.len().min(u8::MAX as usize) as u8,
        });
    }
    if unavailable.len() != TILE_KIND_COUNT {
        return Err(CalcError::HandMeldInconsistent {
            expected: TILE_KIND_COUNT as u8,
            actual: unavailable.len().min(u8::MAX as usize) as u8,
        });
    }
    let mut result = [0; TILE_KIND_COUNT];
    for index in 0..TILE_KIND_COUNT {
        if counts[index] > 4 {
            return Err(CalcError::CountOutOfRange {
                field: "counts34".to_owned(),
                index,
                value: counts[index],
            });
        }
        if unavailable[index] > 4 {
            return Err(CalcError::CountOutOfRange {
                field: "unavailable_counts34".to_owned(),
                index,
                value: unavailable[index],
            });
        }
        if unavailable[index] < counts[index] {
            return Err(CalcError::UnavailableBelowHand {
                index,
                unavailable: unavailable[index],
                hand: counts[index],
            });
        }
        result[index] = counts[index];
    }
    let total = result.iter().sum::<u8>() + input.meld_count * 3;
    if let Some(expected) = expected_total {
        if total != expected {
            return Err(CalcError::HandMeldInconsistent {
                expected,
                actual: total,
            });
        }
    } else if total != 13 && total != 14 {
        return Err(CalcError::HandMeldInconsistent {
            expected: 13,
            actual: total,
        });
    }
    Ok(result)
}

fn logical_melds(counts: &[u8; TILE_KIND_COUNT], meld_count: u8) -> u8 {
    (counts.iter().sum::<u8>() / 3) + meld_count
}

fn shanten_from_counts(counts: &[u8; TILE_KIND_COUNT], meld_count: u8) -> ShantenResult {
    let logical_melds = logical_melds(counts, meld_count);
    let (standard, chiitoitsu, kokushi) =
        nyanten::calc_shanten_components_from_counts(counts, logical_melds);
    if meld_count > 0 {
        ShantenResult {
            minimum: standard,
            standard,
            chiitoitsu: None,
            kokushi: None,
        }
    } else {
        ShantenResult {
            minimum: standard.min(chiitoitsu).min(kokushi),
            standard,
            chiitoitsu: Some(chiitoitsu),
            kokushi: Some(kokushi),
        }
    }
}

/// Calculate all four-player shanten forms for a validated 13- or 14-tile state.
pub fn calculate_shanten34(input: &CalculationInput) -> Result<ShantenResult, CalcError> {
    let counts = validate(input, None)?;
    Ok(shanten_from_counts(&counts, input.meld_count))
}

fn analyze_draws_validated(
    input: &CalculationInput,
    counts: &[u8; TILE_KIND_COUNT],
) -> DrawAnalysis {
    let shanten = shanten_from_counts(counts, input.meld_count);
    let mut improving_tiles = Vec::new();
    let mut agari_tiles = Vec::new();
    for index in 0..TILE_KIND_COUNT {
        let remaining_count = 4 - input.unavailable_counts34[index];
        if remaining_count == 0 {
            continue;
        }
        let mut next_counts = *counts;
        next_counts[index] += 1;
        let next_shanten = shanten_from_counts(&next_counts, input.meld_count).minimum;
        let analysis = TileAnalysis {
            tile34: index as u8,
            remaining_count,
            next_shanten,
        };
        if next_shanten < shanten.minimum {
            improving_tiles.push(analysis.clone());
        }
        if shanten.minimum == 0 && next_shanten == -1 {
            agari_tiles.push(analysis);
        }
    }
    let improving_tile_kinds = improving_tiles.len() as u8;
    let improving_tile_count = improving_tiles
        .iter()
        .map(|tile| u16::from(tile.remaining_count))
        .sum();
    let agari_tile_kinds = agari_tiles.len() as u8;
    let agari_tile_count = agari_tiles
        .iter()
        .map(|tile| u16::from(tile.remaining_count))
        .sum();
    DrawAnalysis {
        shanten,
        improving_tiles,
        agari_tiles,
        improving_tile_kinds,
        improving_tile_count,
        agari_tile_kinds,
        agari_tile_count,
    }
}

/// Analyse all drawable tile types for a 13-logical-tile state.
pub fn analyze_draws34(input: &CalculationInput) -> Result<DrawAnalysis, CalcError> {
    let counts = validate(input, Some(13))?;
    Ok(analyze_draws_validated(input, &counts))
}

/// Analyse each distinct base tile that may be discarded from a 14-tile state.
pub fn analyze_discards34(input: &CalculationInput) -> Result<Vec<DiscardAnalysis>, CalcError> {
    let counts = validate(input, Some(14))?;
    let mut analyses = Vec::new();
    for index in 0..TILE_KIND_COUNT {
        if counts[index] == 0 {
            continue;
        }
        let mut after_discard = counts;
        after_discard[index] -= 1;
        analyses.push(DiscardAnalysis {
            discard_tile34: index as u8,
            // The discarded physical tile moves from concealed hand to river.
            // It remains unavailable, so the input availability is unchanged.
            draw_analysis: analyze_draws_validated(input, &after_discard),
        });
    }
    Ok(analyses)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "input", rename_all = "snake_case")]
pub enum BatchOperation {
    CalculateShanten34(CalculationInput),
    AnalyzeDraws34(CalculationInput),
    AnalyzeDiscards34(CalculationInput),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchRequest {
    pub id: String,
    #[serde(flatten)]
    pub request: BatchOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CalculationResult {
    Shanten(ShantenResult),
    Draws(DrawAnalysis),
    Discards(Vec<DiscardAnalysis>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub id: String,
    pub result: Option<CalculationResult>,
    pub error: Option<CalcError>,
}

/// Execute scalar-equivalent work in input order.  Duplicate identifiers and
/// contract mismatches fail the whole batch; individual invalid scenes retain
/// their typed errors in the corresponding output item.
pub fn calculate_batch34(requests: &[BatchRequest]) -> Result<Vec<BatchItemResult>, CalcError> {
    let mut ids = HashSet::new();
    for request in requests {
        if !ids.insert(&request.id) {
            return Err(CalcError::DuplicateBatchId {
                id: request.id.clone(),
            });
        }
        let input = match &request.request {
            BatchOperation::CalculateShanten34(input)
            | BatchOperation::AnalyzeDraws34(input)
            | BatchOperation::AnalyzeDiscards34(input) => input,
        };
        if input.contract_version != CALCULATION_CONTRACT_VERSION {
            return Err(CalcError::ContractVersionMismatch {
                expected: CALCULATION_CONTRACT_VERSION.to_owned(),
                actual: input.contract_version.clone(),
            });
        }
    }
    Ok(requests
        .iter()
        .map(|request| {
            let output = match &request.request {
                BatchOperation::CalculateShanten34(input) => {
                    calculate_shanten34(input).map(CalculationResult::Shanten)
                }
                BatchOperation::AnalyzeDraws34(input) => {
                    analyze_draws34(input).map(CalculationResult::Draws)
                }
                BatchOperation::AnalyzeDiscards34(input) => {
                    analyze_discards34(input).map(CalculationResult::Discards)
                }
            };
            match output {
                Ok(result) => BatchItemResult {
                    id: request.id.clone(),
                    result: Some(result),
                    error: None,
                },
                Err(error) => BatchItemResult {
                    id: request.id.clone(),
                    result: None,
                    error: Some(error),
                },
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(counts: &[usize], meld_count: u8) -> CalculationInput {
        let mut counts34 = vec![0; TILE_KIND_COUNT];
        for &tile in counts {
            counts34[tile] += 1;
        }
        CalculationInput {
            variant: Variant::Yonma,
            unavailable_counts34: counts34.clone(),
            counts34,
            meld_count,
            contract_version: CALCULATION_CONTRACT_VERSION.to_owned(),
        }
    }

    #[test]
    fn separates_agari_from_improving_tiles_and_counts_remaining() {
        // 123m 456m 789m 12p 11z: 3p completes the hand.
        let state = input(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27], 0);
        let result = analyze_draws34(&state).unwrap();
        assert_eq!(result.shanten.minimum, 0);
        assert_eq!(result.improving_tiles, result.agari_tiles);
        assert_eq!(result.agari_tiles[0].tile34, 11);
        assert_eq!(result.agari_tiles[0].remaining_count, 4);
        assert_eq!(result.agari_tile_kinds, 1);
        assert_eq!(result.agari_tile_count, 4);
    }

    #[test]
    fn open_hands_exclude_closed_special_forms() {
        let state = input(&[0, 1, 2, 3, 4, 5, 9, 10, 11, 27], 1);
        let result = calculate_shanten34(&state).unwrap();
        assert!(result.chiitoitsu.is_none());
        assert!(result.kokushi.is_none());
    }

    #[test]
    fn validates_without_clamping() {
        let mut state = input(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27], 0);
        state.unavailable_counts34[0] = 0;
        assert!(matches!(
            analyze_draws34(&state),
            Err(CalcError::UnavailableBelowHand { .. })
        ));
        state.unavailable_counts34[0] = 5;
        assert!(matches!(
            analyze_draws34(&state),
            Err(CalcError::CountOutOfRange { .. })
        ));
    }

    #[test]
    fn batch_keeps_scene_errors_but_rejects_duplicate_ids() {
        let valid = input(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27], 0);
        let mut invalid = valid.clone();
        invalid.counts34.pop();
        let output = calculate_batch34(&[
            BatchRequest {
                id: "ok".to_owned(),
                request: BatchOperation::AnalyzeDraws34(valid),
            },
            BatchRequest {
                id: "bad".to_owned(),
                request: BatchOperation::AnalyzeDraws34(invalid),
            },
        ])
        .unwrap();
        assert!(output[0].result.is_some());
        assert!(output[1].error.is_some());
        assert!(matches!(
            calculate_batch34(&[
                BatchRequest {
                    id: "same".to_owned(),
                    request: BatchOperation::CalculateShanten34(input(
                        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27],
                        0
                    ))
                },
                BatchRequest {
                    id: "same".to_owned(),
                    request: BatchOperation::CalculateShanten34(input(
                        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 27, 27],
                        0
                    ))
                },
            ]),
            Err(CalcError::DuplicateBatchId { .. })
        ));
    }

    #[test]
    fn contract_version_is_required_during_deserialization() {
        let json = r#"{"variant":"yonma","counts34":[],"meld_count":0,"unavailable_counts34":[]}"#;
        assert!(serde_json::from_str::<CalculationInput>(json).is_err());
    }

    #[test]
    fn fixed_seed_one_million_valid_states_preserve_tile_invariants() {
        let mut seed = 0x4d30_7631_u64;
        for _ in 0..1_000_000 {
            let mut counts34 = vec![0u8; TILE_KIND_COUNT];
            let mut placed = 0;
            while placed < 13 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let tile = ((seed >> 32) % TILE_KIND_COUNT as u64) as usize;
                if counts34[tile] < 4 {
                    counts34[tile] += 1;
                    placed += 1;
                }
            }
            let input = CalculationInput {
                variant: Variant::Yonma,
                unavailable_counts34: counts34.clone(),
                counts34,
                meld_count: 0,
                contract_version: CALCULATION_CONTRACT_VERSION.to_owned(),
            };
            let result = calculate_shanten34(&input).unwrap();
            assert!(result.minimum <= result.standard);
            assert!(result.chiitoitsu.is_some());
            assert!(result.kokushi.is_some());
        }
    }
}
