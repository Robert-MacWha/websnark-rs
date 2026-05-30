# websnark-rs

websnark-rs is a Websnark-compatible Rust library for generating and verifying
zk-SNARK proofs from artifacts produced by the Circom v1 compiler.

## Examples

```rust
# use std::collections::HashMap;
# use ark_bn254::Fr;
# use ark_ff::AdditiveGroup;
# use websnark_rs::circuit::{calculate_witness, Circuit, Value};
# use websnark_rs::proof::generate_proof;
# use websnark_rs::proving_key::ProvingKey;

# let circuit_json = include_bytes!("src/testdata/withdraw.json");
# let proving_key_json = include_str!("src/testdata/withdraw_proving_key.json");
# let inputs_json = include_bytes!("src/testdata/withdraw_input_signals.json");

let circuit: Circuit = serde_json::from_slice(circuit_json).unwrap();
let pk: ProvingKey = serde_json::from_str(proving_key_json).unwrap();
let inputs: HashMap<String, Value> = serde_json::from_slice(inputs_json).unwrap();

let witness = calculate_witness(circuit, inputs).unwrap();
let (proof, pub_signals) = generate_proof(pk, witness, Fr::ZERO, Fr::ZERO).unwrap();
```

## How it works

Circom V1 produces JSON artifacts for the circuit, proving key, and verification key. Unlike Circom V2, these circuit artifacts include JS code for witness generation that needs to be `eval`ed in JS at runtime. websnark-rs includes a Rust interpreter for this JS code, allowing it to generate witnesses directly without needing to run a separate process or embed a JS engine.

Compared to snarkjs, websnark-rs is significantly faster for both witness and proof generation. The below benchmarks were run against the tornadocash withdraw circuit on a Ryzen 5 3600 CPU.

| benchmark    | websnark-rs (parallel) | websnark-rs | snarkjs (tornadocash fork, node 14) |
| ------------ | ---------------------- | ----------- | ----------------------------------- |
| witness (ms) | 190 ms                 | 190 ms      | 754 ms                              |
| proof (ms)   | 300 ms                 | 1300 ms     | 3500 ms                             |

## Features
 - `parallel`: Enables parallel proof generation using Rayon.

## Wasm support
websnark-rs can be compiled to WASM for use in web applications.
