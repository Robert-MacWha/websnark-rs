#[cfg(not(target_arch = "wasm32"))]
mod bench {
    use std::hint::black_box;

    use ark_bn254::Fr;
    use ark_ff::AdditiveGroup;
    use criterion::Criterion;
    use websnark_rs::{circuit::Witness, proof::prove, proving_key::ProvingKey};

    fn bench_prove(c: &mut Criterion) {
        let pk_str = std::fs::read_to_string("src/testdata/withdraw_proving_key.json")
            .expect("read proving key");
        let witness_str =
            std::fs::read_to_string("src/testdata/witness.json").expect("read witness");

        let pk: ProvingKey = serde_json::from_str(&pk_str).expect("parse proving key");
        let witness: Witness = serde_json::from_str(&witness_str).expect("parse witness");

        let r = Fr::ZERO;
        let s = Fr::ZERO;

        let mut group = c.benchmark_group("prove");
        group.sample_size(20);
        group.bench_function("prove", |b| {
            b.iter_batched(
                || (pk.clone(), witness.clone()),
                |(pk, witness)| {
                    prove(
                        black_box(&pk),
                        black_box(&witness),
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

    criterion::criterion_group!(benches, bench_prove);
}

#[cfg(not(target_arch = "wasm32"))]
criterion::criterion_main!(bench::benches);

#[cfg(target_arch = "wasm32")]
fn main() {}
