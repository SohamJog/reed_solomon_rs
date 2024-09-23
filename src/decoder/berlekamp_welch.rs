use crate::{
    fec::fec::{Share, FEC},
    galois_field::gf_alg::{GfMat, GfVal},
};

// Berlekamp Welch functions for FEC
impl FEC {
    pub fn decode() {}

    pub fn decode_no_concat() {}

    pub fn correct(&self, mut shares: Vec<Share>) -> Result<(), Box<dyn std::error::Error>> {
        if shares.len() < self.k {
            return Err(format!("Must specify at least the number of required shares").into());
        }
        shares.sort();

        // Syndrome matrix stuff

        Ok(())
    }

    pub fn berlekamp_welch() {}

    pub fn syndrome_matrix(&self, shares: Vec<Share>) -> Result<GfMat, Box<dyn std::error::Error>> {
        let mut keepers = vec![false; self.n];
        let mut share_count = 0;
        for share in &shares {
            if !keepers[share.number as usize] {
                keepers[share.number as usize] = true;
                share_count += 1;
            }
        }
        // create a vandermonde matrix but skip columns where we're missing the share

        let mut out = GfMat::matrix_zero(self.k, share_count);
        for i in 0..self.k {
            let mut skipped = 0;
            for j in 0..self.n {
                if !keepers[j] {
                    skipped = skipped + 1;
                }
                out.set(i, j - skipped, GfVal(self.vand_matrix[i * self.n + j]));
            }
        }

        if out.standardize().is_err() {
            return Err(("Matrix standardizing failed").into());
        }

        return Ok(out);
    }
}
