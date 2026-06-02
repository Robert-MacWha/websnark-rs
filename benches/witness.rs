use std::collections::HashMap;

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use websnark_rs::circuit::{Circuit, Value, calculate_witness};

fn bench_calculate_witness(c: &mut Criterion) {
    let circuit_str = std::fs::read_to_string("src/testdata/withdraw.json").expect("read circuit");
    let input_str =
        std::fs::read_to_string("src/testdata/withdraw_input_signals.json").expect("read inputs");

    let circuit = Circuit::from_json(&circuit_str).expect("parse circuit");
    let input_signals: HashMap<String, Value> =
        serde_json::from_str(&input_str).expect("parse inputs");

    let mut group = c.benchmark_group("witness");
    group.sample_size(20);
    group.bench_function("withdraw", |b| {
        b.iter(|| {
            calculate_witness(black_box(circuit.clone()), black_box(input_signals.clone()))
                .expect("witness")
        })
    });
    group.finish();
}

criterion_group!(benches, bench_calculate_witness);
criterion_main!(benches);
