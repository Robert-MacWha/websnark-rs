#[cfg(not(target_arch = "wasm32"))]
mod bench {
    use std::collections::HashMap;

    use criterion::{Criterion, criterion_group};
    use std::hint::black_box;
    use websnark_rs::circuit::{Circuit, Value, generate_witness};

    fn bench_generate_witness(c: &mut Criterion) {
        let circuit_str =
            std::fs::read_to_string("src/testdata/withdraw.json").expect("read circuit");
        let input_str = std::fs::read_to_string("src/testdata/withdraw_input_signals.json")
            .expect("read inputs");

        let circuit: Circuit = serde_json::from_str(&circuit_str).expect("parse circuit");
        let input_signals: HashMap<String, Value> =
            serde_json::from_str(&input_str).expect("parse inputs");

        let mut group = c.benchmark_group("witness");
        group.sample_size(20);
        group.bench_function("withdraw", |b| {
            b.iter_batched(
                || (circuit.clone(), input_signals.clone()),
                |(circuit, input_signals)| {
                    generate_witness(black_box(circuit), black_box(input_signals)).expect("witness")
                },
                criterion::BatchSize::LargeInput,
            )
        });
        group.finish();
    }

    criterion_group!(benches, bench_generate_witness);
}

#[cfg(not(target_arch = "wasm32"))]
criterion::criterion_main!(bench::benches);

#[cfg(target_arch = "wasm32")]
fn main() {}
