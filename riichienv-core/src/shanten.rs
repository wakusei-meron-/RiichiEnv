//! Legacy shanten API compatibility layer.
//!
//! Four-player lookup tables and the canonical implementation live in
//! `riichienv-calc`.  The engine continues to expose the historic module so
//! existing RiichiEnv clients do not need to change imports.

pub use riichienv_calc::legacy::*;
