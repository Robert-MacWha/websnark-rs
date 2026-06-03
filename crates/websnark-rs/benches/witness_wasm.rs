#![cfg(target_arch = "wasm32")]
use std::{collections::HashMap, hint::black_box};

use wasm_bindgen_test::{Criterion, Instant};
use websnark_rs::circuit::{Circuit, Value, generate_witness};

const CIRCUIT_BYTES: &[u8] = include_bytes!("../src/testdata/withdraw.json");
const INPUT_BYTES: &[u8] = include_bytes!("../src/testdata/withdraw_input_signals.json");

/// Required because wasm-bindgen-test doesn't support setting sample_size on
/// the criterion instance.
const _: () = {
    #[unsafe(export_name = "__wbgb__witness_wasm::bench_generate_witness")]
    extern "C" fn __wbgt_test(cx: &::wasm_bindgen_test::__rt::Context) {
        cx.execute_sync(
            "witness_wasm::bench_generate_witness",
            || {
                let mut c = Criterion::default()
                    .with_location("benches/witness_wasm.rs", "witness_wasm")
                    .sample_size(10)
                    .measurement_time(std::time::Duration::from_secs(60));
                bench_generate_witness(&mut c);
            },
            None,
            None,
        );
    }
};

fn bench_generate_witness(c: &mut Criterion) {
    let circuit: Circuit = serde_json::from_slice(CIRCUIT_BYTES).expect("parse circuit");
    let input: HashMap<String, Value> =
        serde_json::from_slice(INPUT_BYTES).expect("parse input signals");

    c.bench_function("generate_witness", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;

            for _ in 0..iters {
                let circuit = circuit.clone();
                let input = input.clone();

                let start = Instant::now();
                generate_witness(black_box(circuit), black_box(input)).unwrap();
                total += start.elapsed();
            }

            total
        })
    });
}
