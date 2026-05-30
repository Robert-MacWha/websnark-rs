fn main() {
    println!("Hello, world!");
}

// use std::{collections::HashMap, fs};

// use tracing::info;
// use tracing_subscriber::fmt::format::FmtSpan;

// use tc_transpiler::{
//     circuit::{Value, calculate_witness::calculate_witness, circuit::Circuit},
//     generate_proof::generate_proof,
//     proof::Proof,
//     proving_key::ProvingKey,
// };

// fn main() -> Result<(), anyhow::Error> {
//     tracing_subscriber::fmt()
//         .with_span_events(FmtSpan::CLOSE)
//         .init();

//     info!("Loading circuit");
//     let circuit_data = fs::read("src/testdata/withdraw.json")?;
//     let circuit: Circuit = serde_json::from_slice(&circuit_data)?;

//     info!("Loading proving key");
//     let pk_data = fs::read("src/testdata/withdraw_proving_key.json")?;
//     let pk: ProvingKey = serde_json::from_slice(&pk_data)?;

//     info!("Loading input signals");
//     let input_signals_data = fs::read("src/testdata/withdraw_input_signals.json")?;
//     let input_signals: HashMap<String, Value> = serde_json::from_slice(&input_signals_data)?;

//     let witness = calculate_witness(circuit, input_signals)?;

//     let r = uint!(0_U256).try_into()?;
//     let s = uint!(0_U256).try_into()?;

//     let (proof, pub_signals) = generate_proof(pk, witness, r, s)?;

//     let expected_proof_data = fs::read("src/testdata/proof.json")?;
//     let expected_proof: Proof = serde_json::from_slice(&expected_proof_data)?;

//     assert_eq!(proof, expected_proof);
//     info!("Proof matches expected proof!");

//     Ok(())
// }
