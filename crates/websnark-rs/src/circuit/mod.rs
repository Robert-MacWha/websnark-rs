//! Circuit definition and witness generation.

#[allow(clippy::module_inception)]
mod circuit;
mod generate_witness;
mod interpreter;
mod rt_ctx;
pub(crate) mod value;
mod witness;

pub use circuit::{Circuit, Component, Function, Signal};
pub use generate_witness::generate_witness;
pub use value::Value;
pub use witness::Witness;
