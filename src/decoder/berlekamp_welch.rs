use crate::{
    fec::fec::{Share, FEC},
    galois_field::gf_alg::{GfMat, GfPoly, GfVal, GfVals},
    math::addmul::addmul,
};

// TODO: Fix error handling as in return the error that the functions return

// Berlekamp Welch functions for FEC
impl FEC {
    pub fn decode(
        &self,
        mut dst: Vec<u8>,
        mut shares: Vec<Share>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.correct(&mut shares)?;

        if shares.len() == 0 {
            return Err(("Must specify at least one share").into());
        }

        let piece_len = shares[0].data.len();
        let result_len = piece_len * self.k;
        if dst.capacity() < result_len {
            dst = vec![0u8; result_len];
        } else {
            dst.resize(result_len, 0);
        }
        self.rebuild(shares, |s: Share| {
            dst[s.number * piece_len..(s.number + 1) * piece_len].copy_from_slice(&s.data);
        })?;
        return Ok(dst);
    }

    pub fn decode_no_concat<F>(
        &self,
        mut shares: Vec<Share>,
        output: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Share),
    {
        self.correct(&mut shares)?;

        return self.rebuild(shares, output);
    }

    pub fn correct(&self, shares: &mut Vec<Share>) -> Result<(), Box<dyn std::error::Error>> {

        if shares.len() < self.k {
            return Err(format!("Must specify at least the number of required shares").into());
        }
        shares.sort();

        // fast path: check to see if there are no errors by evaluating it with the syndrome matrix
        let synd = match self.syndrome_matrix(&shares) {
            Ok(synd) => synd,
            Err(err) => return Err(err.into()),
        };

        let mut buf = vec![0u8; shares[0].data.len()];
        for i in 0..synd.r {
            for j in 0..buf.len() {
                buf[j] = 0;
            }
            for j in 0..synd.c {
                addmul(
                    buf.as_mut_slice(),
                    shares[j].data.as_slice(),
                    synd.get(i, j).0,
                );
            }

            for j in 0..buf.len() {
                if buf[j] == 0 {
                    continue;
                }
                let data = match self.berlekamp_welch(&shares, j) {
                    Ok(data) => data,
                    Err(err) => return Err(err.into()),
                };
                for i in 0..shares.len() {
                    shares[i].data[j] = data[shares[i].number];
                }
            }
        }
        Ok(())
    }

    pub fn berlekamp_welch(
        &self,
        shares: &Vec<Share>,
        index: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
     
        let k = self.k;
        let r = shares.len();
        let e = (r - k) / 2; // deg of E polynomial
        let q = e + k; // deq of Q polynomial

        if e <= 0 {
            return Err(("Not enough shares!").into());
        }

        let interp_base = GfVal(2);
        let eval_point = |num: usize| -> GfVal {
            if num == 0 {
                GfVal(0)
            } else {
                interp_base.pow(num - 1)
            }
        };
        let dim = q + e;
        let mut s = GfMat::matrix_zero(dim, dim); // constraint matrix
        let mut a = GfMat::matrix_zero(dim, dim); // augmented matrix
        let mut f = GfVals::gfvals_zero(dim); // constant column
        let mut u = GfVals::gfvals_zero(dim); // solution column

        for i in 0..dim {
            let x_i = eval_point(shares[i].number);
            let r_i = GfVal(shares[i].data[index]);

            f.0[i] = x_i.pow(e).mul(r_i);

            for j in 0..q {
                s.set(i, j, x_i.pow(j));
                if i == j {
                    a.set(i, j, GfVal(1));
                }
            }

            for k in 0..e {
                let j = k + q;
                s.set(i, j, x_i.pow(k).mul(r_i));
                if i == j {
                    a.set(i, j, GfVal(1));
                }
            }
        }

        // invert and put the result in a
        if s.invert_with(&mut a).is_err() {
            return Err(("Error inverting matrix").into());
        }

        // multiply the inverted matrix by the column vector
        for i in 0..dim {
            let ri = a.index_row(i);
            u.0[i] = ri.dot(&f);
        }

        // reverse u for easier construction of the polynomials
        let len_u = u.0.len();
        for i in 0..len_u / 2 {
            u.0.swap(i, len_u - i - 1);
        }

        let mut q_poly = GfPoly(u.0[e..].to_vec());
        let mut e_poly = GfPoly(vec![GfVal(1)]);
        e_poly.0.extend_from_slice(&u.0[..e]);

        let (p_poly, rem) = match q_poly.div(e_poly) {
            Ok((p_poly, rem)) => (p_poly, rem),
            Err(err) => return Err(err.into()),
        };

        if !rem.is_zero() {
            return Err(("too many errors to reconstruct").into());
        }

        let mut out = vec![0u8; self.n];
        for i in 0..out.len() {
            let mut pt = GfVal(0);
            if i != 0 {
                pt = interp_base.pow(i - 1);
            }
            out[i] = p_poly.eval(pt).0;
        }

        return Ok(out);
    }

    pub fn syndrome_matrix(
        &self,
        shares: &Vec<Share>,
    ) -> Result<GfMat, Box<dyn std::error::Error>> {
        let mut keepers = vec![false; self.n];
        let mut share_count = 0;
        for share in shares {
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
                    continue;
                }

                out.set(i, j - skipped, GfVal(self.vand_matrix[i * self.n + j]));
            }
        }

        if out.standardize().is_err() {
            return Err(("Matrix standardizing failed").into());
        }

        return Ok(out.parity());
    }
}
