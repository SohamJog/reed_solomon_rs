use crate::galois_field::tables::{GF_EXP, GF_INVERSE, GF_MUL_TABLE};
use crate::math::addmul::addmul;

pub struct PivotSearcher {
    k: usize,
    ipiv: Vec<bool>,
}

impl PivotSearcher {
    pub fn new(k: usize) -> Self {
        PivotSearcher {
            k,
            ipiv: vec![false; k],
        }
    }

    pub fn search(&mut self, col: usize, matrix: &[u8]) -> Option<(usize, usize)> {
        if !self.ipiv[col] && matrix[col * self.k + col] != 0 {
            self.ipiv[col] = true;
            return Some((col, col));
        }

        for row in 0..self.k {
            if self.ipiv[row] {
                continue;
            }

            for i in 0..self.k {
                if !self.ipiv[i] && matrix[row * self.k + i] != 0 {
                    self.ipiv[i] = true;
                    return Some((i, row));
                }
            }
        }

        None
    }
}

pub fn swap_row(matrix: &mut [u8], k: usize, row1: usize, row2: usize) {
    for i in 0..k {
        matrix.swap(row1 * k + i, row2 * k + i);
    }
}

pub fn invert_matrix(matrix: &mut [u8], k: usize) -> Result<(), &'static str> {
    let mut pivot_searcher = PivotSearcher::new(k);
    let mut indxc = vec![0; k];
    let mut indxr = vec![0; k];
    let mut id_row = vec![0; k];

    for col in 0..k {
        let (icol, irow) = match pivot_searcher.search(col, matrix) {
            Some((icol, irow)) => (icol, irow),
            None => return Err("pivot not found"),
        };

        if irow != icol {
            swap_row(matrix, k, irow, icol);
        }

        indxr[col] = irow;
        indxc[col] = icol;
        let mut c: u8 = matrix[icol * k + icol];

        if c == 0 {
            return Err("singular matrix");
        }

        if c != 1 {
            c = GF_INVERSE[c as usize];
            matrix[icol * k + icol] = 1;
            for i in 0..k {
                matrix[icol * k + i] = GF_MUL_TABLE[c as usize][matrix[icol * k + i] as usize];
            }
        }

        id_row[icol] = 1;
        if matrix[icol * k..icol * k + k] != id_row {
            for i in 0..k {
                if i != icol {
                    if i < icol {
                        let (slice1, slice2) = matrix.split_at_mut(icol * k);
                        let row1 = &mut slice1[i * k..i * k + k];
                        let row2 = &mut slice2[0..k];
                        c = row1[icol];
                        row1[icol] = 0;
                        addmul(row1, row2, c);
                    } else {
                        let (slice2, slice1) = matrix.split_at_mut(i * k);
                        let row2 = &mut slice2[icol * k..icol * k + k];
                        let row1 = &mut slice1[0..k];
                        c = row1[icol];
                        row1[icol] = 0;
                        addmul(row1, row2, c);
                    }
                }
            }
        }
        id_row[icol] = 0;
    }

    for i in 0..k {
        if indxr[i] != indxc[i] {
            for row in 0..k {
                matrix.swap(row * k + indxr[i], row * k + indxc[i]);
            }
        }
        for j in i + 1..k {
            if indxr[j] == indxr[i] {
                indxr[j] = indxc[i];
            } else if indxr[j] == indxc[i] {
                indxr[j] = indxr[i];
            }
            if indxc[j] == indxc[i] {
                indxc[j] = indxr[i];
            } else if indxc[j] == indxr[i] {
                indxc[j] = indxc[i];
            }
        }
    }

    Ok(())
}

pub fn create_inverted_vdm(vdm: &mut [u8], k: usize) {
    println!("vdm: {:?}, k: {:?}", vdm, k);
    if k == 1 {
        vdm[0] = 1;
        return;
    }
    let mut b = vec![0; k];
    let mut c = vec![0; k];

    for i in 1..k {
        let mul_p_i = &GF_MUL_TABLE[GF_EXP[i] as usize];
        for j in (k - 1 - (i - 1))..(k - 1) {
            c[j] ^= mul_p_i[c[j + 1]] as usize;
        }
        c[k - 1] ^= GF_EXP[i] as usize;
    }

    // Everything is fine till now

    for row in 0..k {
        let mut index = 0;
        if row != 0 {
            index = GF_EXP[row] as usize;
        }
        let mul_p_row = &GF_MUL_TABLE[index as usize];

        let mut t: u8 = 1;
        b[k - 1] = 1;

        for i in (0..k - 1).rev() {
            b[i] = c[i + 1] ^ mul_p_row[b[i + 1]] as usize;
            t = (b[i] ^ (mul_p_row[t as usize]) as usize) as u8;
        }

        println!("row: {:?}, b: {:?}", row, b);


        let mul_t_inv = &GF_MUL_TABLE[GF_INVERSE[t as usize] as usize];
        for col in 0..k {
            vdm[col * k + row] = mul_t_inv[b[col]];
        }

        println!("row: {:?}, vdm: {:?}\n", row, vdm);


    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invert_matrix_gf256() {
        // Example matrix (3x3 identity matrix in GF(256))
        let mut matrix = vec![1, 0, 0, 0, 1, 0, 0, 0, 1];

        let k = 3; // Dimension of the matrix

        // Call invert_matrix function
        let result = invert_matrix(&mut matrix, k);

        // Check if the function succeeded
        assert!(result.is_ok());

        // Expected result after inversion (should still be the identity matrix)
        let expected_matrix = vec![1, 0, 0, 0, 1, 0, 0, 0, 1];

        // Verify the matrix is correct
        assert_eq!(matrix, expected_matrix);
    }

    #[test]
    fn test_invert_non_identity_matrix_gf256() {
        // Example of a non-identity matrix in GF(256)
        let mut matrix = vec![
            1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 7, 7, 6, 6, 1,
        ];

        let k = 5; // Dimension of the matrix

        // Call invert_matrix function
        let result = invert_matrix(&mut matrix, k);

        // Check if the function succeeded
        assert!(
            result.is_ok(),
            "Expected Ok but got Err: {:?}",
            result.err()
        );

        // Expected result after inversion in GF(256)
        let expected_matrix = vec![
            1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 123, 123, 1, 122, 122, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0,
        ];

        // Verify the matrix is correct
        assert_eq!(matrix, expected_matrix);
    }

    #[test]
    fn another_test_invert_non_identity_matrix_gf256() {
        // Example of a non-identity matrix in GF(256)
        let mut matrix = vec![56, 23, 98, 3, 100, 200, 45, 201, 123];

        let k = 3; // Dimension of the matrix

        // Call invert_matrix function
        let result = invert_matrix(&mut matrix, k);

        // Check if the function succeeded
        assert!(result.is_ok());

        // Expected result after inversion in GF(256)
        let expected_matrix = vec![175, 133, 33, 130, 13, 245, 112, 35, 126];

        // Verify the matrix is correct
        assert_eq!(matrix, expected_matrix);
    }
}
