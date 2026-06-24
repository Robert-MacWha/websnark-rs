//! Proof generation and verification logic.

mod fr_snarkjs;
mod generate_proof;
#[allow(clippy::module_inception)]
mod proof;

pub use generate_proof::{generate_proof, generate_random_proof};
pub use proof::Proof;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProofError {
    #[error("Invalid witness: {0}")]
    InvalidWitness(ark_bn254::Fr),
    #[error("Invalid coefficient")]
    InvalidCoefficient,
    #[error("Invalid domain")]
    InvalidDomain,
    #[error("Invalid coset")]
    InvalidCoset,
}
