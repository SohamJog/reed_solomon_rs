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
                  return Some((row, i));
              }
          }
      }

      None
  }
}


pub fn swap(a: &mut u8, b: &mut u8) {
  let tmp = *a;
  *a = *b;
  *b = tmp;
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
          for i in 0..k {
              swap(&mut matrix[irow * k + i], &mut matrix[icol * k + i]);
          }
      }

      indxr[col] = irow;
      indxc[col] = icol;
      let pivot_row = &mut matrix[icol * k..icol * k + k];
      let mut c = pivot_row[icol];

      if c == 0 {
          return Err("singular matrix");
      }

      if c != 1 {
          c = GF_INVERSE[c as usize];
          pivot_row[icol] = 1;
          for i in 0..k {
              pivot_row[i] = GF_MUL_TABLE[c as usize][pivot_row[i] as usize];
          }
      }

      id_row[icol] = 1;
      if pivot_row != id_row.as_slice() {
          let mut p = matrix;
          for i in 0..k {
              if i != icol {
                  c = p[icol];
                  p[icol] = 0;
                  addmul(&mut p[0..k], pivot_row, c);
              }
              p = &mut p[k..];
          }
      }

      id_row[icol] = 0;
  }

  for i in 0..k {
      if indxr[i] != indxc[i] {
          for row in 0..k {
              swap(&mut matrix[row * k + indxr[i]], &mut matrix[row * k + indxc[i]]);
          }
      }
  }

  Ok(())
}

pub fn create_inverted_vdm(vdm: &mut [u8], k: usize) {
  if k == 1 {
    vdm[0] = 1;
    return;
  }
  let mut b = vec![0; k];
  let mut c = vec![0; k];

  for i in 1..k {
    let mul_p_i = &GF_MUL_TABLE[GF_EXP[i] as usize];
    for j in (k-1-(i-1))..(k-1) {
      c[j] ^= mul_p_i[c[j+1]];
    }
    c[k-1] ^= GF_EXP[i]
  }

  for row in 0..k {
    let mut index = 0;
    if row != 0 {
      index = GF_EXP[row] as usize;
    }
    let mul_p_row = &GF_MUL_TABLE[index as usize];

    let mut t: u8 = 1;
    b[k-1] = 1;

    for i in (0..k-1).rev {
      b[i] = c[i+1] ^ mul_p_row[b[i+1]];
      t = b[i] ^ mul_p_row[t];
    }

    let mul_t_inv = &GF_MUL_TABLE[GF_INVERSE[index] as usize];
    for col in 0..k {
      vdm[col*k+row] = mul_t_inv[b[col]];
    }
  }

}