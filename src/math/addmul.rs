use crate::galois_field::tables::GF_MUL_TABLE;

pub fn addmul(z: &mut [u8], x: &[u8], y: u8) {

    println!("Called addmul z: {:?}, x: {:?}, y: {:?}", z, x, y);

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
