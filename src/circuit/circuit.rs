use anyhow::{Result, bail};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::circuit::value::{BinValue, Value};

/// CircomV1-compatible circuit definition
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub names: Vec<String>,
    #[serde(rename = "triggerComponents")]
    pub trigger_components: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub template: String,
    pub params: FxHashMap<String, Value>,
    #[serde(rename = "inputSignals")]
    pub input_signals: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub params: Vec<String>,
    pub func: String,
}

impl Circuit {
    /// Returns the index of the i-th input
    pub fn input_idx(&self, i: u64) -> Result<u64> {
        if i >= self.n_inputs {
            bail!("input index out of bounds: {i} >= {}", self.n_inputs);
        }
        Ok(self.n_outputs + 1 + i)
    }

    pub fn signal_names(&self, i: u64) -> Result<String> {
        let idx = self.input_idx(i)?;
        Ok(self.signals[idx as usize].names.join(","))
    }

    /// Deserialize from the snarkjs camelCase JSON format.
    pub fn from_json(s: &str) -> Result<Self> {
        let raw: CircuitJson = serde_json::from_str(s)?;
        raw.try_into()
    }
}

// ── Binary serde (via CircuitBin) ────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct CircuitBin {
    n_pub_inputs: u64,
    n_prv_inputs: u64,
    n_inputs: u64,
    n_outputs: u64,
    n_vars: u64,
    n_signals: u64,
    n_constants: u64,
    signal_name2_idx: FxHashMap<String, u64>,
    component_name2_idx: FxHashMap<String, u64>,
    signals: Vec<Signal>,
    components: Vec<ComponentBin>,
    templates: FxHashMap<String, String>,
    functions: FxHashMap<String, Function>,
}

#[derive(Serialize, Deserialize)]
struct ComponentBin {
    name: String,
    template: String,
    params: FxHashMap<String, BinValue>,
    input_signals: u64,
}

impl Serialize for Circuit {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        CircuitBin::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
}

impl<'de> Deserialize<'de> for Circuit {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        CircuitBin::deserialize(d)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&Circuit> for CircuitBin {
    type Error = anyhow::Error;

    fn try_from(c: &Circuit) -> Result<Self> {
        let components = c
            .components
            .iter()
            .map(|comp| {
                let params = comp
                    .params
                    .iter()
                    .map(|(k, v)| BinValue::try_from(v).map(|bv| (k.clone(), bv)))
                    .collect::<Result<FxHashMap<_, _>>>()?;
                Ok(ComponentBin {
                    name: comp.name.clone(),
                    template: comp.template.clone(),
                    params,
                    input_signals: comp.input_signals,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(CircuitBin {
            n_pub_inputs: c.n_pub_inputs,
            n_prv_inputs: c.n_prv_inputs,
            n_inputs: c.n_inputs,
            n_outputs: c.n_outputs,
            n_vars: c.n_vars,
            n_signals: c.n_signals,
            n_constants: c.n_constants,
            signal_name2_idx: c.signal_name2_idx.clone(),
            component_name2_idx: c.component_name2_idx.clone(),
            signals: c.signals.clone(),
            components,
            templates: c.templates.clone(),
            functions: c.functions.clone(),
        })
    }
}

impl TryFrom<CircuitBin> for Circuit {
    type Error = anyhow::Error;

    fn try_from(b: CircuitBin) -> Result<Self> {
        let components = b
            .components
            .into_iter()
            .map(|comp| {
                let params = comp
                    .params
                    .into_iter()
                    .map(|(k, bv)| Value::try_from(bv).map(|v| (k, v)))
                    .collect::<Result<FxHashMap<_, _>>>()?;
                Ok(Component {
                    name: comp.name,
                    template: comp.template,
                    params,
                    input_signals: comp.input_signals,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Circuit {
            n_pub_inputs: b.n_pub_inputs,
            n_prv_inputs: b.n_prv_inputs,
            n_inputs: b.n_inputs,
            n_outputs: b.n_outputs,
            n_vars: b.n_vars,
            n_signals: b.n_signals,
            n_constants: b.n_constants,
            signal_name2_idx: b.signal_name2_idx,
            component_name2_idx: b.component_name2_idx,
            signals: b.signals,
            components,
            templates: b.templates,
            functions: b.functions,
        })
    }
}

// ── JSON parsing (snarkjs camelCase format) ───────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CircuitJson {
    n_pub_inputs: u64,
    n_prv_inputs: u64,
    n_inputs: u64,
    n_outputs: u64,
    n_vars: u64,
    n_signals: u64,
    n_constants: u64,
    signal_name2_idx: FxHashMap<String, u64>,
    component_name2_idx: FxHashMap<String, u64>,
    signals: Vec<Signal>,
    components: Vec<ComponentJson>,
    templates: FxHashMap<String, String>,
    functions: FxHashMap<String, Function>,
}

#[derive(Deserialize)]
struct ComponentJson {
    name: String,
    template: String,
    params: FxHashMap<String, serde_json::Value>,
    #[serde(rename = "inputSignals")]
    input_signals: u64,
}

impl TryFrom<CircuitJson> for Circuit {
    type Error = anyhow::Error;

    fn try_from(json: CircuitJson) -> Result<Self> {
        let components = json
            .components
            .into_iter()
            .map(|c| {
                let params = c
                    .params
                    .into_iter()
                    .map(|(k, v)| {
                        Value::from_json(&v)
                            .map(|val| (k, val))
                            .map_err(anyhow::Error::msg)
                    })
                    .collect::<Result<FxHashMap<_, _>>>()?;
                Ok(Component {
                    name: c.name,
                    template: c.template,
                    params,
                    input_signals: c.input_signals,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Circuit {
            n_pub_inputs: json.n_pub_inputs,
            n_prv_inputs: json.n_prv_inputs,
            n_inputs: json.n_inputs,
            n_outputs: json.n_outputs,
            n_vars: json.n_vars,
            n_signals: json.n_signals,
            n_constants: json.n_constants,
            signal_name2_idx: json.signal_name2_idx,
            component_name2_idx: json.component_name2_idx,
            signals: json.signals,
            components,
            templates: json.templates,
            functions: json.functions,
        })
    }
}
