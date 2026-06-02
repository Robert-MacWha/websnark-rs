#[cfg(not(target_arch = "wasm32"))]
mod bench {
    use std::hint::black_box;

    use ark_bn254::Fr;
    use ark_ff::AdditiveGroup;
    use criterion::Criterion;
    use websnark_rs::{circuit::Witness, proof::generate_proof, proving_key::ProvingKey};

    fn bench_generate_proof(c: &mut Criterion) {
        let pk_bytes =
            std::fs::read("src/testdata/withdraw_proving_key.json").expect("read proving key");
        let witness_bytes = std::fs::read("src/testdata/witness.json").expect("read witness");

        let pk: ProvingKey = serde_json::from_slice(&pk_bytes).expect("parse proving key");
        let witness: Witness = serde_json::from_slice(&witness_bytes).expect("parse witness");

        let r = Fr::ZERO;
        let s = Fr::ZERO;

        let mut group = c.benchmark_group("generate_proof");
        group.sample_size(20);
        group.bench_function("generate_proof", |b| {
            b.iter_batched(
                || (pk.clone(), witness.clone()),
                |(pk, witness)| {
                    generate_proof(
                        black_box(pk),
                        black_box(witness),
                        black_box(r),
                        black_box(s),
                    )
                    .unwrap()
                },
                criterion::BatchSize::LargeInput,
            )
        });
        group.finish();
    }

    criterion::criterion_group!(benches, bench_generate_proof);
}

#[cfg(not(target_arch = "wasm32"))]
criterion::criterion_main!(bench::benches);

#[cfg(target_arch = "wasm32")]
fn main() {}
