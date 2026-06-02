//! Circuit definition and witness generation.

mod calculate_witness;
#[allow(clippy::module_inception)]
mod circuit;
mod interpreter;
mod rt_ctx;
pub(crate) mod value;
mod witness;

pub use calculate_witness::calculate_witness;
pub use circuit::{Circuit, Component, Function, Signal};
pub use value::{Value, ValueError};
pub use witness::Witness;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CircuitError {
    #[error("parse error in function {function}: {source}")]
    ParseError {
        #[source]
        source: crate::circom::ParseError,
        function: String,
    },
    #[error("value error: {0}")]
    ValueError(#[from] crate::circuit::value::ValueError),
    #[error("assertion failed: {0} != {1}: {2}")]
    AssertionFailed(Box<ark_bn254::Fr>, Box<ark_bn254::Fr>, String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
