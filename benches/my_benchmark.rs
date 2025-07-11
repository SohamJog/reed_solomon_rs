use crate::fec::fec::*;
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use reed_solomon_rs::*;
use std::time::Duration;

// Helper
fn create_shares(fec: &FEC, data: &mut [u8], total: usize) -> Vec<Share> {
    let mut shares = vec![
        Share {
            number: 0,
            data: vec![]
        };
        total
    ];

    let output = |s: Share| {
        shares[s.number] = s.clone();
    };
    fec.encode(data, output).expect("Encoding failed");
    shares
}

// Modify randmo bytes
fn corrupt_shares(shares: &mut Vec<Share>, corruption_level: usize) {
    let mut rng = SmallRng::from_entropy();
    for _ in 0..corruption_level {
        let share_index = rng.gen_range(0, shares.len());
        let byte_index = rng.gen_range(0, shares[share_index].data.len());
        shares[share_index].data[byte_index] ^= 0xFF; // Flip some bits
    }
}

// Encode benchmark
fn benchmark_encoding(
    c: &mut BenchmarkGroup<WallTime>,
    data_size: usize,
    required: usize,
    total: usize,
) {
    let mut data: Vec<u8> = vec![b'x'; data_size];
    let fec = FEC::new(required, total).expect("FEC init failed");

    c.throughput(criterion::Throughput::Bytes(data_size.try_into().unwrap()));

    c.bench_function(
        &format!("encode {}B r{} t{}", data_size, required, total),
        |b| {
            b.iter(|| {
                let mut shares = vec![
                    Share {
                        number: 0,
                        data: vec![]
                    };
                    total
                ];
                let output = |s: Share| {
                    shares[s.number] = s.clone();
                };
                fec.encode(&mut data, output).expect("Encoding failed");
            })
        },
    );
}

// Decode benchmark
fn benchmark_decoding(
    c: &mut BenchmarkGroup<WallTime>,
    data_size: usize,
    required: usize,
    total: usize,
    corruption_level: usize,
) {
    let mut data: Vec<u8> = vec![b'x'; data_size];
    let fec = FEC::new(required, total).expect("FEC init failed");

    // Encode data and corrupt shares
    let mut shares = create_shares(&fec, &mut data, total);
    corrupt_shares(&mut shares, corruption_level);
    c.throughput(criterion::Throughput::Bytes(data_size.try_into().unwrap()));

    c.bench_function(
        &format!(
            "decode {}B r{} t{} corruption_level{}",
            data_size, required, total, corruption_level
        ),
        |b| {
            b.iter(|| {
                let decoded = fec.decode(vec![], shares.clone()).expect("Decoding failed");
                assert_eq!(&decoded, &data);
            })
        },
    );
}

// All benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    let data_sizes = [64, 512, 1024, 2048];
    let required = 4;
    let total_configs = vec![8, 12];

    // Encode benchmarks
    for &data_size in &data_sizes {
        for &total in &total_configs {
            let mut group = c.benchmark_group(format!("Galois 8 Encode {:?}", data_size));

            benchmark_encoding(&mut group, data_size, required, total);
        }
    }

    // Decode benchmarks with corruption levels
    let corruption_levels = vec![1, required / 2, required - 1];
    for &data_size in &data_sizes {
        for &total in &total_configs {
            for &corruption_level in &corruption_levels {
                let mut group = c.benchmark_group(format!("Galois 8 Decode {:?} with corruption level{:?}", data_size, corruption_level));

                benchmark_decoding(&mut group, data_size, required, total, corruption_level);
            }
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(5));
    targets = criterion_benchmark
}
criterion_main!(benches);

