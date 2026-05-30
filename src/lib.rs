#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod circom;
pub mod circuit;
pub mod proof;
pub mod proving_key;
mod utils;
pub mod verifying_key;

#[cfg(all(target_arch = "wasm32", feature = "parallel"))]
compile_error!("The 'parallel' feature is not supported in WASM builds.");
