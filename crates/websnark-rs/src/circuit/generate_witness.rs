use std::collections::HashMap;

use crate::{
    circuit::witness::Witness,
    circuit::{Circuit, rt_ctx::RTCtx, value::Value},
};
use anyhow::{Result, bail};
use ark_bn254::Fr;
use num_bigint::BigInt;
use tracing::instrument;

/// Generate the witness for a given circuit and input signals.
///
/// The `input_signals` map should contain the named input values for the circuit.
#[instrument(skip_all)]
pub fn generate_witness(
    circuit: Circuit,
    input_signals: HashMap<String, Value>,
) -> Result<Witness> {
    let mut ctx = RTCtx::new(circuit)?;
    ctx.set_signal("one", vec![], 1.into())?;

    for (c, v) in ctx.not_init_signals.clone().iter().enumerate() {
        if *v == 0 {
            ctx.trigger_component(c)?;
        }
    }

    for (name, values) in input_signals {
        ctx.current_component = Some("main".to_string());
        iterate_selector(&mut ctx, &name, values, &mut Vec::new())?;
    }

    for i in 0..ctx.circuit.n_inputs {
        let idx = ctx.circuit.input_idx(i)?;
        if ctx.witness[idx as usize].is_none() {
            bail!(
                "input signal not assigned: {}",
                ctx.circuit.signal_names(i)?
            );
        }
    }

    for i in 0..ctx.witness.len() {
        if ctx.witness[i].is_none() {
            bail!(
                "signal not assigned: {}",
                ctx.circuit.signal_names(i as u64)?
            )
        }
    }

    let output = ctx.witness[..ctx.circuit.n_vars as usize]
        .iter()
        .map(|v| (*v).ok_or(anyhow::anyhow!("signal not assigned")))
        .collect::<Result<Vec<Fr>>>()?;
    Ok(Witness::new(output))
}

fn iterate_selector(
    ctx: &mut RTCtx,
    name: &str,
    values: Value,
    sels: &mut Vec<BigInt>,
) -> Result<()> {
    match values {
        Value::Number(_) => {
            ctx.set_signal(
                name,
                sels.iter().map(|s| Value::Number(s.clone())).collect(),
                values,
            )?;
        }
        Value::Fr(_) => {
            ctx.set_signal(
                name,
                sels.iter().map(|s| Value::Number(s.clone())).collect(),
                values,
            )?;
        }
        Value::Array(arr) => {
            for (i, val) in arr.into_iter().enumerate() {
                sels.push(i.into());
                iterate_selector(ctx, name, val, sels)?;
                sels.pop();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_generate_witness() {
        let circuit_data = include_str!("../testdata/withdraw.json");
        let circuit = Circuit::from_json(circuit_data).unwrap();

        let input_signal_data = include_str!("../testdata/withdraw_input_signals.json");
        let input_signals: HashMap<String, Value> =
            serde_json::from_str(input_signal_data).unwrap();

        let expected_witness_data = include_str!("../testdata/witness.json");
        let expected_witness = Witness::from_json(expected_witness_data).unwrap();

        let witness = generate_witness(circuit, input_signals).unwrap();
        assert_eq!(witness, expected_witness);
    }
}
