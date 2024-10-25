use crate::fec::fec::*;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use reed_solomon_rs::*;
use std::time::Duration;

// Helper
fn create_shares(fec: &FEC, data: &[u8], total: usize) -> Vec<Share> {
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
fn benchmark_encoding(c: &mut Criterion, data_size: usize, required: usize, total: usize) {
    let data: Vec<u8> = vec![b'x'; data_size];
    let fec = FEC::new(required, total).expect("FEC init failed");

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
                fec.encode(&data, output).expect("Encoding failed");
            })
        },
    );
}

// Decode benchmark
fn benchmark_decoding(
    c: &mut Criterion,
    data_size: usize,
    required: usize,
    total: usize,
    corruption_level: usize,
) {
    let data: Vec<u8> = vec![b'x'; data_size];
    let fec = FEC::new(required, total).expect("FEC init failed");

    // Encode data and corrupt shares
    let mut shares = create_shares(&fec, &data, total);
    corrupt_shares(&mut shares, corruption_level);

    c.bench_function(
        &format!(
            "decode {}B r{} t{} corruption_level{}",
            data_size, required, total, corruption_level
        ),
        |b| {
            b.iter(|| {
                let decoded = fec.decode(vec![], shares.clone()).expect("Decoding failed");
                assert_eq!(&decoded, &data); // Verify correctness. @akhilsb can this be avoided?
            })
        },
    );
}

// All benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    let data_sizes = [64, 128, 256, 512, 1024, 2048];
    let required = 4; // @akhilsb should we also vary this variable?
    let total_configs = vec![8, 12]; // For redundancy. @akhilsb is benching across different redundancies necessary? 

    // Encode benchmarks
    for &data_size in &data_sizes {
        for &total in &total_configs {
            benchmark_encoding(c, data_size, required, total);
        }
    }

    // Decode benchmarks with corruption levels
    let corruption_levels = vec![1, required / 2, required - 1];
    for &data_size in &data_sizes {
        for &total in &total_configs {
            for &corruption_level in &corruption_levels {
                benchmark_decoding(c, data_size, required, total, corruption_level);
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


// use crate::fec::fec::*;
// use criterion::{criterion_group, criterion_main, Criterion};
// use reed_solomon_rs::*;

// fn initialize_fec() -> FEC {

//     let required = 4;
//     let total = 8;
//     FEC::new(required, total).unwrap()
// }

// fn benchmark_encode_decode_one_corruption(c: &mut Criterion) {

//     c.bench_function("encode_decode_one_corruption", |b| {
//         // Initialize FEC once, outside of the benchmark iteration
//         let total = 8;
//         let f = initialize_fec();

//         b.iter(|| {
//             let mut shares: Vec<Share> = vec![
//                 Share {
//                     number: 0,
//                     data: vec![]
//                 };
//                 total
//             ];

//             let data = b"hello, world! __".to_vec();

//             let output = |s: Share| {
//                 shares[s.number] = s.clone();
//             };
//             // Time 68.225 ns

//             // Encode
//             f.encode(&data, output).unwrap();
//             // Time: 878.69 ns

//             // Corrupt 1 share
//             shares[1].data[1] = b'?';

//             // // Decode
//             let result_data = f.decode([].to_vec(), shares).unwrap();
//             //Time: 2.1287 µs without corruption
//             // Time: 6.2442 µs

//             assert_eq!(String::from_utf8(result_data).unwrap(), "hello, world! __");
//         })
//     });
// }

// criterion_group!(benches, benchmark_encode_decode_one_corruption);
// criterion_main!(benches);
