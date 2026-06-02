#![doc = include_str!("../../../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod circom;
pub mod circuit;
pub mod proof;
pub mod proving_key;
mod serde;
pub mod verifying_key;

#[cfg(all(feature = "parallel", target_arch = "wasm32"))]
pub use wasm_bindgen_rayon::init_thread_pool;
