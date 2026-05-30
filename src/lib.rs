#![doc = include_str!("../README.md")]

mod circom;
pub mod circuit;
pub mod proof;
pub mod proving_key;
mod utils;
pub mod verifying_key;

#[cfg(all(target_arch = "wasm32", feature = "parallel"))]
compile_error!("The 'parallel' feature is not supported in WASM builds.");
