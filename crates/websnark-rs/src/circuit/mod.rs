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
pub use value::Value;
pub use witness::Witness;
