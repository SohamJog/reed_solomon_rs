use crate::galois_field::{gf_alg::GfVal, tables::GF_MUL_TABLE};

pub fn addmul(z: &mut [u8], x: &[u8], y: u8) {

    if y == 0 {
        return;
    }

    // Safety: We assume `x` has the same length as `z`.
    // The bounds check is removed by slicing `x` to the length of `z`.
    let x = &x[..z.len()];

    let gf_mul_y = &GF_MUL_TABLE[y as usize];
    for (zi, &xi) in z.iter_mut().zip(x.iter()) {
        *zi ^= gf_mul_y[xi as usize];
    }
}

pub fn addmul_gfval(z: &mut [GfVal], x: &[GfVal], y: GfVal) {
    // If y is zero, no need to do anything
    if y.0 == 0 {
        return;
    }

    // Safety: We assume `x` has the same length as `z`.
    // The bounds check is removed by slicing `x` to the length of `z`.
    let x = &x[..z.len()];

    // Get the multiplication table for the value of `y`
    let gf_mul_y = &GF_MUL_TABLE[y.0 as usize];

    // Iterate over `z` and `x`, mutating `z` in-place
    for (zi, &GfVal(xi)) in z.iter_mut().zip(x.iter()) {
        zi.0 ^= gf_mul_y[xi as usize];
    }
}
