use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::circuit::{CircuitError, value::Value};

/// CircomV1-compatible circuit definition
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Circuit {
    pub n_pub_inputs: u64,
    pub n_prv_inputs: u64,
    pub n_inputs: u64,
    pub n_outputs: u64,
    pub n_vars: u64,
    pub n_signals: u64,
    pub n_constants: u64,

    pub signal_name2_idx: FxHashMap<String, u64>,
    pub component_name2_idx: FxHashMap<String, u64>,

    pub signals: Vec<Signal>,
    pub components: Vec<Component>,

    pub templates: FxHashMap<String, String>,
    pub functions: FxHashMap<String, Function>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Signal {
    pub names: Vec<String>,
    #[serde(rename = "triggerComponents")]
    pub trigger_components: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Component {
    pub name: String,
    pub template: String,
    pub params: FxHashMap<String, Value>,
    #[serde(rename = "inputSignals")]
    pub input_signals: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Function {
    pub params: Vec<String>,
    pub func: String,
}

impl Circuit {
    /// Returns the index of the i-th input
    pub fn input_idx(&self, i: u64) -> Result<u64, CircuitError> {
        if i >= self.n_inputs {
            return Err(CircuitError::InputIndexOutOfBounds(i, self.n_inputs));
        }
        Ok(self.n_outputs + 1 + i)
    }

    pub fn signal_names(&self, i: u64) -> Result<String, CircuitError> {
        let idx = self.input_idx(i)?;
        Ok(self.signals[idx as usize].names.join(","))
    }
}
