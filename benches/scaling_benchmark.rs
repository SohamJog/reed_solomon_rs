// use crate::fec::fec::*;
use reed_solomon_rs::fec::fec::*;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};

/*
Message size 256*n bits
Shards n = 16 to 200
Observed scaling behavior

nodes = n, t = n-1/3

16 nodes -> 156.07 µs
32 nodes -> 1.3696 ms
48 nodes -> 4.5043 ms
64 nodes -> 10.209 ms
80 nodes -> 22.012 ms
96 nodes -> 39.584 ms
112 nodes -> 67.883 ms
128 nodes -> 104.84 ms
144 nodes -> 139.75 ms
160 nodes -> 221.09 ms
176 nodes -> 311.44 ms
192 nodes -> 359.90 ms


*/

fn create_and_corrupt_shares(required: usize, total: usize, data: &[u8]) -> (FEC, Vec<Share>) {
    let fec = FEC::new(required, total).expect("FEC init failed");

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

    fec.encode(&mut data.clone(), output)
        .expect("Encoding failed");

    let corruption_level = required;
    // let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..corruption_level {
        let share_index = rng.gen_range(0..shares.len());
        let byte_index = rng.gen_range(0..shares[share_index].data.len());
        shares[share_index].data[byte_index] ^= 0xAA; // flipping bits
    }

    (fec, shares)
}
fn trim_trailing_underscores(mut v: Vec<u8>) -> Vec<u8> {
    while let Some(&last) = v.last() {
        if last == b'_' {
            v.pop();
        } else {
            break;
        }
    }
    v
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scaling Error Correction");

    for n in (16..=200).step_by(16) {
        let required = (2 * n) / 3;
        let data_size = 32 * n; // 256*n bits = 32*n bytes
        let data: Vec<u8> = vec![42; data_size]; // Dummy data

        let (fec, shares) = create_and_corrupt_shares(required, n, &data);

        group.bench_function(format!("decode_n{}", n), |b| {
            b.iter(|| {
                let recovered = fec.decode(vec![], shares.clone()).expect("Decode failed");
                // remove tailing underscores from recovered

                // assert_eq!(recovered, data);
                let trimmed = trim_trailing_underscores(recovered);

                assert_eq!(trimmed, data);
            });
        });
    }

    group.finish();
}

criterion_group!(scaling_benches, benchmark_scaling);
criterion_main!(scaling_benches);
