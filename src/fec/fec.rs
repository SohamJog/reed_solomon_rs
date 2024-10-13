use crate::galois_field::tables::{GF_EXP, GF_MUL_TABLE};
use crate::math::addmul::addmul;
use crate::math::pivot_searcher::{create_inverted_vdm, invert_matrix};
use std::error::Error;

#[derive(Debug)]
pub struct FEC {
    pub k: usize,
    pub n: usize,
    pub enc_matrix: Vec<u8>,
    pub vand_matrix: Vec<u8>,
}

// Number is the x coordinate
// Data is the y coordinate
#[derive(Debug)]
pub struct Share {
    pub number: usize,
    pub data: Vec<u8>,
}

impl Clone for Share {
    fn clone(&self) -> Share {
        Share {
            number: self.number,
            data: self.data.clone(), // Deep copy of Vec<u8>
        }
    }
}

impl PartialEq for Share {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Eq for Share {}

impl PartialOrd for Share {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Share {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
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

    pub fn encode_single(
        &self,
        input: &[u8],
        output: &mut [u8],
        num: usize,
    ) -> Result<(), Box<dyn Error>> {
        let size = input.len();
        let k = self.k;
        let n = self.n;
        let enc_matrix = &self.enc_matrix;

        if num >= n {
            return Err(format!("num must be less than {}", n).into());
        }

        if size % k != 0 {
            return Err(format!("input length must be a multiple of {}", k).into());
        }

        let block_size = size / k;

        if output.len() != block_size {
            return Err(format!("output length must be {}", block_size).into());
        }

        if num < k {
            output.copy_from_slice(&input[num * block_size..(num + 1) * block_size]);
            return Ok(());
        }

        output.fill(0);

        for i in 0..k {
            addmul(
                output,
                &input[i * block_size..(i + 1) * block_size],
                enc_matrix[num * k + i],
            );
        }

        Ok(())
    }

    pub fn rebuild<F>(
        &self,
        mut shares: Vec<Share>,
        mut output: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Share),
    {
        println!("Rebuild called with shares: {:?}", shares);
        let size = shares.len();
        let k = self.k;
        let n = self.n;
        let enc_matrix = &self.enc_matrix;

        if size < k {
            return Err(("Not enough Shares!").into());
        }
        let share_size = shares[0].data.len();
        shares.sort();

        let mut m_dec = vec![0u8; k * k];
        let mut indexes = vec![0; k];
        let mut sharesv: Vec<Vec<u8>> = vec![vec![]; k];

        let mut shares_b_iter = 0;
        let mut shares_e_iter = size - 1;

        for i in 0..k {
            let mut share_id: usize = 0;
            let mut share_data: Vec<u8> = Vec::new();
            if let Some(share) = shares.get(shares_b_iter) {
                if share.number == i {
                    share_id = share.number;
                    share_data = share.data.clone();
                    shares_b_iter += 1;
                } else if let Some(share) = shares.get(shares_e_iter) {
                    share_id = share.number;
                    share_data = share.data.clone();
                    shares_e_iter -= 1;
                }
            }
            if share_id >= n {
                return Err(format!("invalid share id {}", share_id).into());
            }
            if share_id < k {
                m_dec[i * (k + 1)] = 1;
                println!("Number: {:?}, Data: {:?}", share_id, share_data);
                output(Share {
                    number: share_id,
                    data: share_data.clone(),
                });
            } else {
                m_dec[i * k..i * k + k]
                    .copy_from_slice(&enc_matrix[share_id * k..share_id * k + k]);
            }
            sharesv[i] = share_data;
            indexes[i] = share_id;
        }

        println!("BEFORE INVERT m_dec = {:?}", m_dec);

        if invert_matrix(&mut m_dec, k).is_err() {
            return Err(("Matrix inversion failed").into());
        }

        println!("m_dec = {:?}", m_dec);

        let mut buf = vec![0u8; share_size];

        for i in 0..indexes.len() {
            if indexes[i] >= k {
                buf.fill(0);

                for col in 0..k {
                    addmul(&mut buf, &sharesv[col], m_dec[i * k + col]);

                    println!("Number: {:?}, Data: {:?}", i, buf);

                    output(Share {
                        number: i,
                        data: buf.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}
