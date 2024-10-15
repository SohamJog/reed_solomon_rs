use crate::fec::fec::*;
use criterion::{criterion_group, criterion_main, Criterion};
use reed_solomon_rs::*;

fn benchmark_encode_decode_one_corruption(c: &mut Criterion) {
    c.bench_function("encode_decode_one_corruption", |b| {
        b.iter(|| {
            let required = 4;
            let total = 8;
            let f = FEC::new(required, total).unwrap();

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

            f.encode(&data, output).unwrap();

            // Corrupt 1 share
            shares[1].data[1] = b'?';

            let result_data = f.decode([].to_vec(), shares).unwrap();

            assert_eq!(String::from_utf8(result_data).unwrap(), "hello, world! __");
        })
    });
}

criterion_group!(benches, benchmark_encode_decode_one_corruption);
criterion_main!(benches);
