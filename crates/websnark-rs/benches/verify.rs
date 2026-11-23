#[cfg(not(target_arch = "wasm32"))]
mod bench {
    use std::hint::black_box;

    use criterion::Criterion;
    use websnark_rs::{
        circuit::Witness,
        proof::{Proof, verify},
        verifying_key::VerifyingKey,
    };

    fn bench_verify(c: &mut Criterion) {
        let vk_str = std::fs::read_to_string("src/testdata/withdraw_verification_key.json")
            .expect("read verifying key");
        let proof_str = std::fs::read_to_string("src/testdata/proof.json").expect("read proof");
        let witness_str =
            std::fs::read_to_string("src/testdata/witness.json").expect("read witness");

        let vk: VerifyingKey = serde_json::from_str(&vk_str).expect("parse verifying key");
        let proof: Proof = serde_json::from_str(&proof_str).expect("parse proof");
        let witness: Witness = serde_json::from_str(&witness_str).expect("parse witness");
        let pub_signals = &witness[1..vk.ic.len()];

        let mut group = c.benchmark_group("verify");
        group.sample_size(20);
        group.bench_function("verify", |b| {
            b.iter(|| verify(black_box(&vk), black_box(&proof), black_box(pub_signals)).unwrap());
        });
        group.finish();
    }

    criterion::criterion_group!(benches, bench_verify);
}

#[cfg(not(target_arch = "wasm32"))]
criterion::criterion_main!(bench::benches);

#[cfg(target_arch = "wasm32")]
fn main() {}
