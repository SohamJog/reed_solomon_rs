use crate::galois_field::tables::{GF_EXP, GF_LOG, GF_MUL_TABLE};
use crate::math::addmul::addmul_gfval;
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct GfVal(pub u8);

impl fmt::Display for GfVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl GfVal {
    fn gfval_usize(self) -> usize {
        self.0 as usize
    }

    pub fn pow(&self, val: usize) -> GfVal {
        let mut out = 1u8;
        let mul_base = &GF_MUL_TABLE[self.gfval_usize()];
        for _ in 0..val {
            out = mul_base[out as usize];
        }
        GfVal(out)
    }

    pub fn mul(self, b: GfVal) -> GfVal {
        GfVal(GF_MUL_TABLE[self.gfval_usize()][b.gfval_usize()])
    }

    pub fn div(self, b: GfVal) -> Result<GfVal, &'static str> {
        if b.0 == 0 {
            return Err("divide by zero");
        }
        if self.0 == 0 {
            return Ok(GfVal(0));
        }
        Ok(GfVal(
            GF_EXP[(GF_LOG[self.gfval_usize()] as i32 - GF_LOG[b.gfval_usize()] as i32) as usize],
        ))
    }

    pub fn add(self, b: GfVal) -> GfVal {
        GfVal(self.0 ^ b.0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn inv(self) -> Result<GfVal, &'static str> {
        if self.0 == 0 {
            return Err("invert zero");
        }
        Ok(GfVal(GF_EXP[(255 - GF_LOG[self.gfval_usize()]) as usize]))
    }
}

#[derive(Debug)]
pub struct GfVals(pub Vec<GfVal>);

impl GfVals {
    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|val| format!("{}", val))
            .collect::<Vec<String>>()
            .join(", ")
    }
    pub fn gfvals_zero(size: usize) -> GfVals {
        let out = vec![GfVal(0); size];
        GfVals(out)
    }
    pub fn unsafe_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr() as *const u8,
                self.0.len() * std::mem::size_of::<GfVal>(),
            )
        }
    }
    pub fn unsafe_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.0.as_mut_ptr() as *mut u8,
                self.0.len() * std::mem::size_of::<GfVal>(),
            )
        }
    }

    pub fn dot(&self, b: &GfVals) -> GfVal {
        self.0
            .iter()
            .zip(b.0.iter())
            .map(|(a_i, b_i)| a_i.mul(*b_i))
            .fold(GfVal(0), |acc, val| acc.add(val))
    }
}

#[derive(Clone, Debug)]

pub struct GfPoly(pub Vec<GfVal>);

impl fmt::Display for GfPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values: Vec<u8> = self.0.iter().map(|val| val.0).collect();
        write!(f, "{:?}", values)
    }
}


impl GfPoly {
    pub fn poly_zero(size: usize) -> GfPoly {
        let out = vec![GfVal(0); size];
        GfPoly(out)
    }

    pub fn is_zero(self) -> bool {
        for coef in self.0 {
            if !coef.is_zero() {
                return false;
            }
        }
        true
    }

    pub fn deg(&self) -> usize {
        (self.0.len()) - 1
    }

    pub fn index(&self, power: i32) -> GfVal {
        if power < 0 {
            return GfVal(0);
        }
        let deg = self.deg();
        let which = deg - (power as usize);
        if deg < power as usize {
            return GfVal(0);
        }
        self.0[which]
    }

    pub fn scale(&self, factor: GfVal) -> GfPoly {
        let mut out = GfPoly(vec![GfVal(0); self.0.len()]);
        for (i, coef) in self.0.iter().enumerate() {
            out.0[i] = coef.mul(factor);
        }
        out
    }

    pub fn set(&mut self, pow: usize, coef: GfVal) {
        let deg = self.deg();

        if deg < pow {
            // Extend the polynomial by appending zeroes
            let zeros = vec![GfVal(0); pow - deg];
            self.0.extend(zeros);
        }

        let which = self.deg() - pow;
        if which < self.0.len() {
            self.0[which] = coef;
        }
    }

    pub fn add(&self, b: &GfPoly) -> GfPoly {

        let mut size = self.0.len();
        if b.0.len() > size {
            size = b.0.len();
        }
        let mut out = GfPoly(vec![GfVal(0); size]);

        for i in 0..size {
            let pi = self.index(i as i32);
            let bi = b.index(i as i32);
            out.set(i, pi.add(bi));
        }

        out
    }

    pub fn eval(&self, x: GfVal) -> GfVal {
        let mut out = GfVal(0);
        for i in 0..=self.deg() {
            let x_i = x.pow(i);
            let p_i = self.index(i as i32);
            out = out.add(p_i.mul(x_i));
        }
        out
    }

    pub fn div(&mut self, mut b: GfPoly) -> Result<(GfPoly, GfPoly), Box<dyn std::error::Error>> {
        // Sanitize the divisor by removing leading zeros
        let mut q = GfPoly::poly_zero(0);
        while !b.0.is_empty() && b.0[0].is_zero() {
            b.0.remove(0);
        }
        if b.0.is_empty() {
            return Err("divide by zero".into());
        }

        // Sanitize the base poly as well
        while !self.0.is_empty() && self.0[0].is_zero() {
            self.0.remove(0);
        }
        if self.0.is_empty() {
            return Ok((GfPoly::poly_zero(1), GfPoly::poly_zero(1)));
        }


        while b.deg() <= self.deg() {

            let leading_p = self.index(self.deg() as i32);
            let leading_b = b.index(b.deg() as i32);
            let coef = match leading_p.div(leading_b) {
                Ok(coef) => coef,
                Err(e) => {
                    return Err(e.into());
                }
            };
            let new_vec = vec![coef];
            q.0.extend(new_vec);

            let scaled = b.scale(coef);
            let padding = GfPoly(vec![GfVal(0); self.deg() - scaled.deg()]); // Create a zero polynomial for padding
            let padded = GfPoly([scaled.0, padding.0].concat());
            *self = self.add(&padded);
            if !self.0[0].is_zero() {
                return Err(format!("Alg error: {}", self).into());
            }
            self.0.drain(..1);
        }

        while self.0.len() > 0 && self.0[0].is_zero() {
            self.0.drain(..1);
        }

        return Ok((q, self.clone()));
    }
}

#[derive(Debug)]
pub struct GfMat {
    pub r: usize,
    pub c: usize,
    pub d: GfVals,
}

impl GfMat {
    pub fn matrix_zero(r: usize, c: usize) -> GfMat {
        GfMat {
            r,
            c,
            d: GfVals::gfvals_zero(r * c),
        }
    }

    pub fn to_string(&self) -> String {
        if self.r == 0 {
            return String::new();
        }

        let mut out = String::new();
        for i in 0..self.r - 1 {
            out.push_str(&format!("{:?}\n", self.index_row(i).to_string()));
        }
        out.push_str(&format!("{:?}", self.index_row(self.r - 1).to_string()));

        out
    }

    fn index(&self, i: usize, j: usize) -> usize {
        self.c * i + j
    }

    pub fn get(&self, i: usize, j: usize) -> GfVal {
        self.d.0[self.index(i, j)]
    }

    pub fn set(&mut self, i: usize, j: usize, val: GfVal) {
        let index = self.index(i, j);
        self.d.0[index] = val; // This is fine; the mutable borrow is used here.
    }

    // Mutable version of index_row
    pub fn index_row_mut(&mut self, i: usize) -> &mut [GfVal] {
        let start = self.index(i, 0);
        let end = self.index(i + 1, 0);
        &mut self.d.0[start..end]
    }

    pub fn index_row(&self, i: usize) -> GfVals {
        let start = self.index(i, 0);
        let end = self.index(i + 1, 0);
        GfVals(self.d.0[start..end].to_vec())
    }

    pub fn swap_row(&mut self, i: usize, j: usize) {
        let mut tmp = vec![GfVal(0); self.c];
        let ri = self.index_row(i).0;
        let rj = self.index_row(j).0;

        tmp.copy_from_slice(&ri);
        for (idx, &val) in rj.iter().enumerate() {
            self.set(i, idx, val);
        }
        for (idx, &val) in tmp.iter().enumerate() {
            self.set(j, idx, val);
        }
    }

    pub fn scale_row(&mut self, i: usize, val: GfVal) {
        let ri = self.index_row_mut(i);
        for j in 0..ri.len() {
            ri[j] = ri[j].mul(val);
        }
    }

    pub fn addmul_row(&mut self, i: usize, j: usize, val: GfVal) {
        let ri = self.index_row(i);
        let rj = self.index_row_mut(j);

        addmul_gfval(rj, &ri.0, val);
    }

    pub fn invert_with(&mut self, a: &mut GfMat) -> Result<(), &'static str> {
        for i in 0..self.r {
            let mut p_row = i;
            let mut p_val = self.get(i, i);

            for j in (i + 1)..self.r {
                if p_val.is_zero() {
                    p_row = j;
                    p_val = self.get(j, i);
                }
            }

            if p_val.is_zero() {
                continue; // If the pivot value is zero, skip to the next iteration
            }

            if p_row != i {
                self.swap_row(i, p_row);
                a.swap_row(i, p_row);
            }

            let inv = p_val.inv().map_err(|_| "Inverse calculation failed")?;
            self.scale_row(i, inv);
            a.scale_row(i, inv);

            for j in (i + 1)..self.r {
                let leading = self.get(j, i);
                self.addmul_row(i, j, leading);
                a.addmul_row(i, j, leading);
            }
        }

        for i in (1..self.r).rev() {
            for j in (0..i).rev() {
                let trailing = self.get(j, i);
                self.addmul_row(i, j, trailing);
                a.addmul_row(i, j, trailing);
            }
        }

        Ok(())
    }

    pub fn standardize(&mut self) -> Result<(), &'static str> {
        for i in 0..self.r {
            let mut p_row = i;
            let mut p_val = self.get(i, i);

            for j in (i + 1)..self.r {
                if p_val.is_zero() {
                    p_row = j;
                    p_val = self.get(j, i);
                } else {
                    break;
                }
            }

            if p_val.is_zero() {
                continue;
            }

            if p_row != i {
                // DEBUG HERE
                self.swap_row(i, p_row);
            }

            // DEBUG HERE
            let inv = p_val.inv().map_err(|_| "Inverse calculation failed")?;

            self.scale_row(i, inv);


            for j in (i + 1)..self.r {
                let leading = self.get(j, i);
                self.addmul_row(i, j, leading);
            }
        }

        for i in (1..self.r).rev() {
            for j in (0..i).rev() {
                let trailing = self.get(j, i);
                self.addmul_row(i, j, trailing);
            }
        }

        Ok(())
    }

    // Not in place
    pub fn parity(&self) -> GfMat {
        // Assume m is in standard form already
        // Form: [I_r | P]
        // Output will be [-P_transpose | I_(c - r)]
        // Characteristic 2 means we do not need the negative.

        let mut out = GfMat::matrix_zero(self.c - self.r, self.c);

        // Step 1: Fill in the identity. It starts at column offset r.
        for i in 0..(self.c - self.r) {
            out.set(i, i + self.r, GfVal(1));
        }

        // Step 2: Fill in the transposed P matrix.
        for i in 0..(self.c - self.r) {
            for j in 0..self.r {
                out.set(i, j, self.get(j, i + self.r));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_gf_poly_div() {
        // Create the dividend (q) as a GfPoly
        let mut q = GfPoly(vec![
            GfVal(0x5e),
            GfVal(0x60),
            GfVal(0x8c),
            GfVal(0x3d),
            GfVal(0xc6),
            GfVal(0x8e),
            GfVal(0x7e),
            GfVal(0xa5),
            GfVal(0x2c),
            GfVal(0xa4),
            GfVal(0x04),
            GfVal(0x8a),
            GfVal(0x2b),
            GfVal(0xc2),
            GfVal(0x36),
            GfVal(0x0f),
            GfVal(0xfc),
            GfVal(0x3f),
            GfVal(0x09),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
        ]);

        // Create the divisor (e) as a GfPoly
        let e = GfPoly(vec![
            GfVal(0x01),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
            GfVal(0x00),
        ]);

        // Perform division
        let result = q.div(e);

        // Check for errors
        assert!(
            result.is_ok(),
            "Expected successful division, but got an error"
        );
    }
}
