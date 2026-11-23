#![cfg(target_arch = "wasm32")]

use std::hint::black_box;

use wasm_bindgen_test::{Criterion, Instant};
use websnark_rs::{
    circuit::Witness,
    proof::{Proof, verify},
    verifying_key::VerifyingKey,
};

const VK_JSON: &str = include_str!("../src/testdata/withdraw_verification_key.json");
const PROOF_JSON: &str = include_str!("../src/testdata/proof.json");
const WITNESS_JSON: &str = include_str!("../src/testdata/witness.json");

/// Required because wasm-bindgen-test doesn't support setting sample_size on
/// the criterion instance.
const _: () = {
    #[unsafe(export_name = "__wbgb__verify_wasm::bench_verify")]
    extern "C" fn __wbgt_test(cx: &::wasm_bindgen_test::__rt::Context) {
        cx.execute_sync(
            "verify_wasm::bench_verify",
            || {
                let mut c = Criterion::default()
                    .with_location("benches/verify_wasm.rs", "verify_wasm")
                    .sample_size(10)
                    .measurement_time(std::time::Duration::from_secs(10));
                bench_verify(&mut c);
            },
            None,
            None,
        );
    }
};

fn bench_verify(c: &mut Criterion) {
    let vk: VerifyingKey = serde_json::from_str(VK_JSON).expect("parse verifying key");
    let proof: Proof = serde_json::from_str(PROOF_JSON).expect("parse proof");
    let witness: Witness = serde_json::from_str(WITNESS_JSON).expect("parse witness");
    let pub_signals = &witness[1..vk.ic.len()];

    c.bench_function("verify", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;

            for _ in 0..iters {
                let start = Instant::now();
                verify(black_box(&vk), black_box(&proof), black_box(pub_signals)).unwrap();
                total += start.elapsed();
            }

            total
        });
    });
}
