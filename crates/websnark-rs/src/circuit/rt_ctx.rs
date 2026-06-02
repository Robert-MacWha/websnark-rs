use std::{fmt::Write, rc::Rc};

use ark_bn254::Fr;
use num_bigint::BigInt;
use rustc_hash::FxHashMap;
use tracing::{debug, trace};

use crate::{
    circom::{ast::Function, parse_function},
    circuit::{
        CircuitError,
        circuit::Circuit,
        interpreter::execute_function,
        value::{Value, ValueError},
    },
};

/// Runtime context for witness generation.
///
/// Manages the circuit, current witness values, variable scopes, and component
/// triggering logic.  Based on snarkjs's `src/calculateWitness.js::RTCtx` class.
pub struct RTCtx {
    pub circuit: Circuit,
    pub witness: Vec<Option<Fr>>,
    pub not_init_signals: Vec<i64>,
    pub current_component: Option<String>,

    scopes: Vec<FxHashMap<String, Value>>,

    templates: FxHashMap<String, Rc<Function>>,
    #[allow(dead_code)]
    functions: FxHashMap<String, Rc<Function>>,
}

impl RTCtx {
    pub fn new(circuit: Circuit) -> Result<Self, CircuitError> {
        let n_signals = circuit.n_signals as usize;
        let not_init_signals = circuit
            .components
            .iter()
            .map(|c| c.input_signals as i64)
            .collect();

        let templates = circuit
            .templates
            .iter()
            .map(|(k, v)| {
                let func = parse_function(v).map_err(|e| CircuitError::ParseError {
                    source: e,
                    function: k.clone(),
                })?;
                Ok((k.clone(), Rc::new(func)))
            })
            .collect::<Result<_, CircuitError>>()?;

        let functions = circuit
            .functions
            .iter()
            .map(|(k, v)| {
                let func = parse_function(&v.func).map_err(|e| CircuitError::ParseError {
                    source: e,
                    function: k.clone(),
                })?;
                Ok((k.clone(), Rc::new(func)))
            })
            .collect::<Result<_, CircuitError>>()?;

        Ok(Self {
            circuit,
            witness: vec![None; n_signals],
            not_init_signals,
            current_component: None,
            scopes: vec![FxHashMap::default()],
            templates,
            functions,
        })
    }

    pub fn set_signal(
        &mut self,
        name: &str,
        sels: Vec<Value>,
        value: Value,
    ) -> Result<(), CircuitError> {
        let sels = into_numbers(sels)?;
        let value = value.into_fr()?;

        let full = self.build_signal_name(name, sels)?;
        self.set_signal_full(&full, value)
    }

    pub fn get_signal(&self, name: &str, sels: Vec<Value>) -> Result<Value, CircuitError> {
        let sels = into_numbers(sels)?;

        let full = self.build_signal_name(name, sels)?;
        self.get_signal_full(&full).map(Into::into)
    }

    pub fn set_pin(
        &mut self,
        component_name: &str,
        component_sels: Vec<Value>,
        signal_name: &str,
        signal_sels: Vec<Value>,
        value: Value,
    ) -> Result<(), CircuitError> {
        let component_sels = into_numbers(component_sels)?;
        let signal_sels = into_numbers(signal_sels)?;
        let value = value.into_fr()?;

        let full = self.build_pin_name(component_name, component_sels, signal_name, signal_sels)?;
        self.set_signal_full(&full, value)
    }

    pub fn get_pin(
        &self,
        component_name: &str,
        component_sels: Vec<Value>,
        signal_name: &str,
        signal_sels: Vec<Value>,
    ) -> Result<Value, CircuitError> {
        let component_sels = into_numbers(component_sels)?;
        let signal_sels = into_numbers(signal_sels)?;

        let full = self.build_pin_name(component_name, component_sels, signal_name, signal_sels)?;
        self.get_signal_full(&full).map(Into::into)
    }

    pub fn set_var(
        &mut self,
        name: &str,
        sels: Vec<Value>,
        value: Value,
    ) -> Result<Value, CircuitError> {
        let sels = into_numbers(sels)?;
        let scope = self
            .scopes
            .last_mut()
            .ok_or_else(|| anyhow::anyhow!("No active scope"))?;

        if sels.is_empty() {
            scope.insert(name.to_string(), value.clone());
            return Ok(value);
        }

        let entry = scope
            .entry(name.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let Value::Array(arr) = entry else {
            return Err(anyhow::anyhow!("Variable is not an array: {}", name).into());
        };
        set_var_array(arr, &sels, value.clone());
        Ok(value)
    }

    pub fn get_var(&self, name: &str, sels: Vec<Value>) -> Result<Value, CircuitError> {
        let sels = into_numbers(sels)?;
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return select(v, &sels).cloned();
            }
        }
        Err(anyhow::anyhow!("Variable not defined: {}", name).into())
    }

    pub fn call_function(&mut self, _name: &str, _args: &[Value]) -> Result<Value, CircuitError> {
        Err(anyhow::anyhow!("call_function is not supported yet").into())
    }

    pub fn trigger_component(&mut self, c: usize) -> Result<(), CircuitError> {
        debug!("component triggered: {}", self.circuit.components[c].name);
        self.not_init_signals[c] -= 1;

        // Push new scope for this component
        let component = &self.circuit.components[c];
        let new_component = Some(component.name.clone());
        let old_component = std::mem::replace(&mut self.current_component, new_component);

        let new_scope = component.params.clone();
        let first_scope = self.scopes.first().cloned().unwrap_or_default();
        let new_scope = vec![first_scope, new_scope];
        let old_scope = std::mem::replace(&mut self.scopes, new_scope);

        let func = self
            .templates
            .get(&component.template)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Template not defined: {} for component {}",
                    component.template,
                    component.name
                )
            })?
            .clone();
        execute_function(self, &func)?;

        // Restore old component and scope
        self.scopes = old_scope;
        self.current_component = old_component;
        debug!("component finished: {}", self.circuit.components[c].name);
        Ok(())
    }

    fn build_signal_name(&self, name: &str, sels: Vec<u32>) -> Result<String, CircuitError> {
        let mut s = if let Some(current) = &self.current_component {
            format!("{}.{}", current, name)
        } else {
            name.to_string()
        };
        append_sels(&mut s, sels)?;
        Ok(s)
    }

    fn build_pin_name(
        &self,
        component_name: &str,
        component_sels: Vec<u32>,
        signal_name: &str,
        signal_sels: Vec<u32>,
    ) -> Result<String, CircuitError> {
        let mut s = if component_name == "one" {
            "one".to_string()
        } else if let Some(current) = &self.current_component {
            format!("{}.{}", current, component_name)
        } else {
            component_name.to_string()
        };
        append_sels(&mut s, component_sels)?;
        s.push('.');
        s.push_str(signal_name);
        append_sels(&mut s, signal_sels)?;
        Ok(s)
    }

    fn signal_idx(&self, full: &str) -> Result<usize, CircuitError> {
        if let Some(&idx) = self.circuit.signal_name2_idx.get(full) {
            return Ok(idx as usize);
        }
        Ok(full
            .parse::<usize>()
            .map_err(|e| anyhow::anyhow!("Invalid signal index: {}", e))?)
    }

    fn get_signal_full(&self, full: &str) -> Result<Fr, CircuitError> {
        trace!("get {full}");
        let s_id = self.signal_idx(full)?;
        Ok(
            self.witness[s_id]
                .ok_or_else(|| anyhow::anyhow!("Signal not initialized: {}", full))?,
        )
    }

    fn set_signal_full(&mut self, full: &str, value: Fr) -> Result<(), CircuitError> {
        trace!("set {full} = {value}");
        let s_id = self.signal_idx(full)?;
        let first_init = self.witness[s_id].is_none();
        self.witness[s_id] = Some(value);

        let mut to_trigger = Vec::new();
        let trigs = self.circuit.signals[s_id].trigger_components.clone();
        for &c in &trigs {
            let c = c as usize;
            if first_init {
                self.not_init_signals[c] -= 1;
            }
            to_trigger.push(c);
        }

        for c in to_trigger {
            if self.not_init_signals[c] == 0 {
                self.trigger_component(c)?;
            }
        }

        Ok(())
    }

    pub fn assert_eq(&self, a: &Fr, b: &Fr, err: &str) -> Result<(), CircuitError> {
        if a != b {
            return Err(CircuitError::AssertionFailed(*a, *b, err.to_string()));
        }
        Ok(())
    }
}

fn into_numbers(vals: Vec<Value>) -> Result<Vec<u32>, CircuitError> {
    Ok(vals
        .into_iter()
        .map(|v| v.into_u32())
        .collect::<Result<Vec<_>, ValueError>>()?)
}

fn set_var_array(a: &mut Vec<Value>, sels: &[u32], value: Value) {
    let idx = sels[0] as usize;
    while a.len() <= idx {
        a.push(Value::Number(BigInt::ZERO));
    }
    if sels.len() == 1 {
        a[idx] = value;
        return;
    }
    if !matches!(a[idx], Value::Array(_)) {
        a[idx] = Value::Array(Vec::new());
    }
    let Value::Array(nested) = &mut a[idx] else {
        unreachable!()
    };
    set_var_array(nested, &sels[1..], value)
}

fn select<'a>(a: &'a Value, sels: &[u32]) -> Result<&'a Value, CircuitError> {
    if sels.is_empty() {
        return Ok(a);
    }
    let Value::Array(arr) = a else {
        return Err(CircuitError::ValueError(ValueError::ExpectedArray));
    };
    let idx = sels[0] as usize;
    let next = arr
        .get(idx)
        .ok_or_else(|| CircuitError::ValueError(ValueError::ValueOutOfRange))?;
    select(next, &sels[1..])
}

fn append_sels(out: &mut String, sels: Vec<u32>) -> Result<(), CircuitError> {
    for s in sels {
        write!(out, "[{}]", s)
            .map_err(|e| anyhow::anyhow!("Failed to build signal name: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> RTCtx {
        let data = include_str!("../testdata/withdraw.json");
        let circuit = Circuit::from_json(data).unwrap();
        RTCtx::new(circuit).unwrap()
    }

    #[test]
    fn full_name_signal() {
        let mut ctx = fixture();
        ctx.current_component = Some("main".to_string());
        assert_eq!(ctx.build_signal_name("root", vec![]).unwrap(), "main.root");
        assert_eq!(
            ctx.build_signal_name("out", vec![3]).unwrap(),
            "main.out[3]"
        );
        assert_eq!(
            ctx.build_signal_name("x", vec![1, 2]).unwrap(),
            "main.x[1][2]"
        );
    }

    #[test]
    fn full_name_pin() {
        let mut ctx = fixture();
        ctx.current_component = Some("main".to_string());
        assert_eq!(
            ctx.build_pin_name("one", vec![], "x", vec![]).unwrap(),
            "one.x"
        );
        assert_eq!(
            ctx.build_pin_name("hasher", vec![], "in", vec![5]).unwrap(),
            "main.hasher.in[5]"
        );
    }

    #[test]
    fn set_one_signal_and_get_back() {
        let mut ctx = fixture();
        ctx.set_signal("one", vec![], 1.into()).unwrap();
        assert_eq!(
            ctx.get_signal("one", vec![]).unwrap().into_u32().unwrap(),
            1
        );
    }

    #[test]
    fn not_init_signals_starts_at_input_count() {
        let ctx = fixture();
        assert_eq!(
            ctx.not_init_signals[0],
            ctx.circuit.components[0].input_signals as i64
        );
    }

    fn num(n: i64) -> Value {
        Value::Number(BigInt::from(n))
    }

    #[test]
    fn set_get_var_scalar() {
        let mut ctx = fixture();
        ctx.set_var("x", vec![], num(42)).unwrap();
        assert_eq!(ctx.get_var("x", vec![]).unwrap(), num(42));
    }

    #[test]
    fn set_get_var_indexed() {
        let mut ctx = fixture();
        ctx.set_var("x", vec![num(2)], num(7)).unwrap();
        assert_eq!(ctx.get_var("x", vec![num(2)]).unwrap(), num(7));
        assert_eq!(ctx.get_var("x", vec![num(0)]).unwrap(), num(0));
        assert_eq!(ctx.get_var("x", vec![num(1)]).unwrap(), num(0));
    }

    #[test]
    fn set_get_var_nested() {
        let mut ctx = fixture();
        ctx.set_var("x", vec![num(1), num(2)], num(9)).unwrap();
        assert_eq!(ctx.get_var("x", vec![num(1), num(2)]).unwrap(), num(9));
        assert_eq!(
            ctx.get_var("x", vec![]).unwrap(),
            Value::Array(vec![num(0), Value::Array(vec![num(0), num(0), num(9)]),])
        );
    }

    #[test]
    fn get_var_falls_back_to_outer_scope() {
        let mut ctx = fixture();
        ctx.set_var("x", vec![], num(5)).unwrap();
        ctx.scopes.push(FxHashMap::default());
        assert_eq!(ctx.get_var("x", vec![]).unwrap(), num(5));
    }

    #[test]
    fn set_var_shadows_outer_scope() {
        let mut ctx = fixture();
        ctx.set_var("x", vec![], num(5)).unwrap();
        ctx.scopes.push(FxHashMap::default());
        ctx.set_var("x", vec![], num(10)).unwrap();
        assert_eq!(ctx.get_var("x", vec![]).unwrap(), num(10));
        ctx.scopes.pop();
        assert_eq!(ctx.get_var("x", vec![]).unwrap(), num(5));
    }

    #[test]
    fn get_var_undefined_errors() {
        let ctx = fixture();
        assert!(ctx.get_var("nope", vec![]).is_err());
    }
}
