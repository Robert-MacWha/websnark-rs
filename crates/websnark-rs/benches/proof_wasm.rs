#![cfg(target_arch = "wasm32")]
use std::hint::black_box;

use ark_bn254::Fr;
use ark_ff::AdditiveGroup;
use wasm_bindgen_test::{Criterion, Instant};
use websnark_rs::{circuit::Witness, proof::generate_proof, proving_key::ProvingKey};

const PK_JSON: &str = include_str!("../src/testdata/withdraw_proving_key.json");
const WITNESS_JSON: &str = include_str!("../src/testdata/witness.json");

/// Required because wasm-bindgen-test doesn't support setting sample_size on
/// the criterion instance.
const _: () = {
    #[unsafe(export_name = "__wbgb__proof_wasm::bench_generate_proof")]
    extern "C" fn __wbgt_test(cx: &::wasm_bindgen_test::__rt::Context) {
        cx.execute_sync(
            "proof_wasm::bench_generate_proof",
            || {
                let mut c = Criterion::default()
                    .with_location("benches/proof_wasm.rs", "proof_wasm")
                    .sample_size(10)
                    .measurement_time(std::time::Duration::from_secs(60));
                bench_generate_proof(&mut c);
            },
            None,
            None,
        );
    }
};

fn bench_generate_proof(c: &mut Criterion) {
    let pk = ProvingKey::from_json(PK_JSON).expect("parse proving key");
    let witness = Witness::from_json(WITNESS_JSON).expect("parse witness");

    let r = Fr::ZERO;
    let s = Fr::ZERO;

    c.bench_function("generate_proof", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;

            for _ in 0..iters {
                let pk = pk.clone();
                let witness = witness.clone();

                let start = Instant::now();
                generate_proof(
                    black_box(pk),
                    black_box(witness),
                    black_box(r),
                    black_box(s),
                )
                .unwrap();
                total += start.elapsed();
            }

            total
        });
    });
}
