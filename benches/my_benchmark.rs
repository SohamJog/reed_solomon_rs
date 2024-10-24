use crate::fec::fec::*;
use criterion::{criterion_group, criterion_main, Criterion};
use reed_solomon_rs::*;
//use std::sync::Once;

// Use a `Once` to ensure setup happens only once
// static INIT: Once = Once::new();

fn initialize_fec() -> FEC {
    
    let required = 4;
    let total = 8;
    FEC::new(required, total).unwrap()
}

// fn setup_once() {
//     // Perform any initialization that should happen only once
//     INIT.call_once(|| {
//         // Any other global setup logic
//         let _ = initialize_fec();
//     });
// }

fn benchmark_encode_decode_one_corruption(c: &mut Criterion) {
    // Run setup once before the benchmark
    // setup_once();

    c.bench_function("encode_decode_one_corruption", |b| {
        // Initialize FEC once, outside of the benchmark iteration
        let total = 8;  
        let f = initialize_fec();

        b.iter(|| {
            let mut shares: Vec<Share> = vec![
                Share {
                    number: 0,
                    data: vec![]
                };
                total
            ];

            let data = b"hello, world! __".to_vec();

            let output = |s: Share| {
                shares[s.number] = s.clone();
            };

            // Encode
            f.encode(&data, output).unwrap();
            // Time: 878.69 ns

            // Corrupt 1 share
            shares[1].data[1] = b'?';
            
            // // Decode
            let result_data = f.decode([].to_vec(), shares).unwrap();
            //Time: 2.1287 µs without corruption
            // Time: 6.2442 µs

            assert_eq!(String::from_utf8(result_data).unwrap(), "hello, world! __");
        })
    });
}

criterion_group!(benches, benchmark_encode_decode_one_corruption);
criterion_main!(benches);


// use crate::fec::fec::*;
// use criterion::{criterion_group, criterion_main, Criterion};
// use reed_solomon_rs::*;

// fn benchmark_encode_decode_one_corruption(c: &mut Criterion) {
//     c.bench_function("encode_decode_one_corruption", |b| {
//         b.iter(|| {
//             let required = 4;
//             let total = 8;
//             let f = FEC::new(required, total).unwrap();

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

//             f.encode(&data, output).unwrap();

//             // Corrupt 1 share
//             shares[1].data[1] = b'?';

//             let result_data = f.decode([].to_vec(), shares).unwrap();

//             assert_eq!(String::from_utf8(result_data).unwrap(), "hello, world! __");
//         })
//     });
// }

// criterion_group!(benches, benchmark_encode_decode_one_corruption);
// criterion_main!(benches);


