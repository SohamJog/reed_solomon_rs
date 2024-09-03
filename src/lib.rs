pub mod galois_field {
    pub mod gf_alg;
    pub mod tables;
}

pub mod math {
    pub mod addmul;
    pub mod pivot_searcher;
}

pub mod fec {
    pub mod fec;
}

pub mod decoder {
    pub mod berlekamp_welch;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
