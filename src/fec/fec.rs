use crate::galois_field::tables::{GF_EXP, GF_MUL_TABLE};
use crate::math::addmul::addmul;
use crate::math::pivot_searcher::create_inverted_vdm;
use std::error::Error;

pub struct FEC {
    pub k: usize,
    pub n: usize,
    pub enc_matrix: Vec<u8>,
    pub vand_matrix: Vec<u8>,
}

pub struct Share {
    pub number: usize,
    pub data: Vec<u8>,
}

impl FEC {
    pub fn new(k: usize, n: usize) -> Result<FEC, Box<dyn Error>> {
        if k <= 0 || n <= 0 || k > 256 || n > 256 || k > n {
            return Err("requires 1 <= k <= n <= 256".into());
        }

        let mut enc_matrix = vec![0u8; n * k];
        let mut temp_matrix = vec![0u8; n * k];
        create_inverted_vdm(&mut temp_matrix, k);

        for i in k * k..temp_matrix.len() {
            temp_matrix[i] = GF_EXP[((i / k) * (i % k)) % 255];
        }

        for i in 0..k {
            enc_matrix[i * (k + 1)] = 1;
        }

        for row in (k * k..n * k).step_by(k) {
            for col in 0..k {
                let pa = &temp_matrix[row..];
                let pb = &temp_matrix[col..];
                let mut acc = 0u8;
                for (_i, (pa, pb)) in pa.iter().zip(pb.iter().step_by(k)).enumerate().take(k) {
                    acc ^= GF_MUL_TABLE[*pa as usize][*pb as usize];
                }
                enc_matrix[row + col] = acc;
            }
        }

        // vand_matrix has more columns than rows
        // k rows, n columns.
        let mut vand_matrix = vec![0u8; k * n];
        vand_matrix[0] = 1;
        let mut g = 1u8;
        for row in 0..k {
            let mut a = 1u8;
            for col in 1..n {
                vand_matrix[row * n + col] = a; // 2.pow(i * j) FIGURE IT OUT
                a = GF_MUL_TABLE[g as usize][a as usize];
            }
            g = GF_MUL_TABLE[2][g as usize];
        }

        Ok(FEC {
            k,
            n,
            enc_matrix,
            vand_matrix,
        })
    }

    pub fn required(&self) -> usize {
        self.k
    }

    pub fn total(&self) -> usize {
        self.n
    }

    // Encode will take input data and encode to the total number of pieces n this
    // *FEC is configured for. It will call the callback output n times.
    //
    // The input data must be a multiple of the required number of pieces k.
    // Padding to this multiple is up to the caller.
    //
    // Note that the byte slices in Shares passed to output may be reused when
    // output returns.

    pub fn encode<F>(&self, input: &[u8], mut output: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Share),
    {
        let size = input.len();
        let k = self.k;
        let n = self.n;
        let enc_matrix = &self.enc_matrix;

        if size % k != 0 {
            return Err(format!("input length must be a multiple of {}", k).into());
        }

        let block_size = size / k;

        for i in 0..k {
            output(Share {
                number: i,
                data: input[i * block_size..(i + 1) * block_size].to_vec(),
            });
        }

        let mut fec_buf = vec![0u8; block_size];
        for i in k..n {
            fec_buf.iter_mut().for_each(|byte| *byte = 0);

            for j in 0..k {
                addmul(
                    &mut fec_buf,
                    &input[j * block_size..(j + 1) * block_size],
                    enc_matrix[i * k + j],
                );
            }

            output(Share {
                number: i,
                data: fec_buf.clone(),
            });
        }

        Ok(())
    }

    
}
