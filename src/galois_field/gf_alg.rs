use crate::galois_field::tables::{GF_EXP, GF_MUL_TABLE, GF_LOG};

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
        Ok(GfVal(GF_EXP[(GF_LOG[self.gfval_usize()] as i32 - GF_LOG[b.gfval_usize()] as i32) as usize]))
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
        Ok(GfVal(GF_EXP[(255-GF_LOG[self.gfval_usize()]) as usize]))
    }
}


pub struct GfVals(pub [GfVal]);

impl GfVals {
    pub fn unsafe_bytes(&self) -> &[u8] {
        // SAFETY: We're converting GfVals into a byte slice, which requires unsafe.
        // This is safe as long as GfVal is a simple wrapper around u8, which it is.
        unsafe {
            let ptr = self.0.as_ptr() as *const u8;
            let len = self.0.len() * std::mem::size_of::<GfVal>();
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn dot(&self, b: &GfVals) -> GfVal {
        let mut out = GfVal(0);
        for i in 0..self.0.len() {
            out = out.add(self.0[i].mul(b.0[i]));
        }
        out
    }
}

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

    pub fn deg(self) -> usize {
        (self.0.len()) - 1
    }

    pub fn scale(&self, factor: GfVal) -> GfPoly {
        let mut out = GfPoly(vec![GfVal(0); self.0.len()]);
        for (i, coef) in self.0.iter().enumerate() {
            out.0[i] = coef.mul(factor);
        }
        out
    }

    // pub fn set(&mut self, pow: usize, coef: GfVal) {
    //     let deg = self.deg();

    //     if deg < pow {
    //         // Extend the polynomial by appending zeroes
    //         let zeros = vec![GfVal(0); pow-deg];
    //         self.0.extend(zeros);
    //     }

    //     let which = self.deg() - pow;
    //     if which < self.0.len() {
    //         self.0[which] = coef;
    //     }
    // }
}


