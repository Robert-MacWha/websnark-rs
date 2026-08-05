use rustc_hash::FxHashMap;

use crate::circuit::{CircuitError, value::Value};

/// CircomV1-compatible circuit definition
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Circuit {
    pub n_pub_inputs: usize,
    pub n_prv_inputs: usize,
    pub n_inputs: usize,
    pub n_outputs: usize,
    pub n_vars: usize,
    pub n_signals: usize,
    pub n_constants: usize,

    pub signal_name2_idx: FxHashMap<String, usize>,
    pub component_name2_idx: FxHashMap<String, usize>,

    pub signals: Vec<Signal>,
    pub components: Vec<Component>,

    pub templates: FxHashMap<String, String>,
    pub functions: FxHashMap<String, Function>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Signal {
    pub names: Vec<String>,
    #[cfg_attr(feature = "serde", serde(rename = "triggerComponents"))]
    pub trigger_components: Vec<usize>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Component {
    pub name: String,
    pub template: String,
    pub params: FxHashMap<String, Value>,
    #[cfg_attr(feature = "serde", serde(rename = "inputSignals"))]
    pub input_signals: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Function {
    pub params: Vec<String>,
    pub func: String,
}

impl Circuit {
    /// Returns the index of the i-th input
    ///
    /// # Errors
    /// Returns an error if the input index is out of bounds
    pub fn input_idx(&self, i: usize) -> Result<usize, CircuitError> {
        if i >= self.n_inputs {
            return Err(CircuitError::InputIndexOutOfBounds(i, self.n_inputs));
        }
        Ok(self.n_outputs + 1 + i)
    }

    /// Returns the name of the i-th input signal
    ///
    /// # Errors
    /// Returns an error if the input index is out of bounds
    pub fn signal_name(&self, i: usize) -> Result<String, CircuitError> {
        let idx = self.input_idx(i)?;
        Ok(self.signals[idx].names.join(","))
    }
}
