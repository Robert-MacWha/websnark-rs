//! Proof generation and verification logic.

mod fr_snarkjs;
mod generate_proof;
#[allow(clippy::module_inception)]
mod proof;

pub use generate_proof::{generate_proof, generate_random_proof};
pub use proof::Proof;
