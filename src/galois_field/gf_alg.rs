use crate::galois_field::tables::{GF_EXP, GF_LOG, GF_MUL_TABLE};

#[derive(Copy, Clone)]
pub struct GfVal(pub u8);

impl GfVal {
    fn gfval_usize(self) -> usize {
        self.0 as usize
    }

    fn pow(self, val: usize) -> GfVal {
        let mut out = 1u8;
        let mul_base = &GF_MUL_TABLE[self.gfval_usize()];
        for _ in 0..val {
            out = mul_base[out as usize];
        }
        GfVal(out)
    }

    fn mul(self, b: GfVal) -> GfVal {
        GfVal(GF_MUL_TABLE[self.gfval_usize()][b.gfval_usize()])
    }

    fn div(self, b: GfVal) -> Result<GfVal, &'static str> {
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

    fn add(self, b: GfVal) -> GfVal {
        GfVal(self.0 ^ b.0)
    }

    fn is_zero(self) -> bool {
        self.0 == 0
    }

    fn inv(self) -> Result<GfVal, &'static str> {
        if self.0 == 0 {
            return Err("invert zero");
        }
        Ok(GfVal(GF_EXP[(255 - GF_LOG[self.gfval_usize()]) as usize]))
    }
}

//pub struct GfVals(pub [GfVal]);

pub struct GfVals(pub Vec<GfVal>);

impl GfVals {
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

    pub fn dot(&self, b: &GfVals) -> GfVal {
        self.0
            .iter()
            .zip(b.0.iter())
            .map(|(a_i, b_i)| a_i.mul(*b_i))
            .fold(GfVal(0), |acc, val| acc.add(val))
    }
}

#[derive(Clone)]
pub struct GfPoly(pub Vec<GfVal>);

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
        let mut out = vec![GfVal(0); size];

        for i in 0..size {
            let pi = self.index(i as i32);
            let bi = b.index(i as i32);
            out[i] = pi.add(bi);
        }

        GfPoly(out)
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

    pub fn div(&mut self, mut b: GfPoly) -> Result<(GfPoly, GfPoly), &'static str> {
        // Sanitize the divisor by removing leading zeros
        let mut q = GfPoly::poly_zero(0);
        while !b.0.is_empty() && b.0[0].is_zero() {
            b.0.remove(0);
        }
        if b.0.is_empty() {
            return Err("divide by zero");
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
                    return Err(e);
                }
            };
            let new_vec = vec![coef];
            q.0.extend(new_vec);

            let scaled = b.scale(coef);
            let padding = GfPoly(vec![GfVal(0); self.deg() - scaled.deg()]); // Create a zero polynomial for padding
            let padded = GfPoly([scaled.0, padding.0].concat()); // No need for `&` here
            *self = self.add(&padded);
            if !self.0[0].is_zero() {
                return Err("Alg Error");
            }
            self.0.drain(..1);
        }

        while self.0.len() > 0 && self.0[0].is_zero() {
            self.0.drain(..1);
        }

        return Ok((q, self.clone()));
    }
}

pub struct GfMat {
    r: usize,
    c: usize,
    d: GfVals,
}

impl GfMat {
    pub fn matrix_zero(r: usize, c: usize) -> GfMat {
        GfMat {
            r,
            c,
            d: GfVals::gfvals_zero(r*c)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module

    #[test]
    fn test_gf_poly_div() {
        // Create the dividend (q) as a GfPoly
        let mut q = GfPoly(vec![
            GfVal(0x5e), GfVal(0x60), GfVal(0x8c), GfVal(0x3d), GfVal(0xc6), GfVal(0x8e),
            GfVal(0x7e), GfVal(0xa5), GfVal(0x2c), GfVal(0xa4), GfVal(0x04), GfVal(0x8a),
            GfVal(0x2b), GfVal(0xc2), GfVal(0x36), GfVal(0x0f), GfVal(0xfc), GfVal(0x3f),
            GfVal(0x09), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00),
            GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00),
        ]);

        // Create the divisor (e) as a GfPoly
        let e = GfPoly(vec![
            GfVal(0x01), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00),
            GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00), GfVal(0x00),
        ]);

        // Perform division
        let result = q.div(e);

        // Check for errors
        assert!(result.is_ok(), "Expected successful division, but got an error");
    }
}
