/// Contains math functions for GF(256).
pub mod galois_field {
    pub mod gf_alg;
    pub mod tables;
}

/// Contains auxiliary math functions.
pub mod math {
    pub mod addmul;
    pub mod pivot_searcher;
}

/// Contains FEC(Forward Error Correction) implementations.
pub mod fec {
    pub mod fec;
}

/// Contains the Berlekamp Welch Decoder and auxiliary functions
pub mod decoder {
    pub mod berlekamp_welch;
}

#[cfg(test)]
mod tests {
    use crate::fec::fec::*;

    #[test]
    fn test_encode_decode_no_corruption() -> Result<(), Box<dyn std::error::Error>> {
        let required = 4;
        let total = 8;
        let f = FEC::new(required, total)?;

        let mut shares: Vec<Share> = vec![
            Share {
                number: 0,
                data: vec![]
            };
            total
        ];

        let mut data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&mut data, output)?;

        let result_data = f.decode([].to_vec(), shares)?;

        assert_eq!(String::from_utf8(result_data)?, "hello, world! __");
        Ok(())
    }

    #[test]
    fn test_encode_decode_one_corruption() -> Result<(), Box<dyn std::error::Error>> {
        let required = 4;
        let total = 8;
        let f = FEC::new(required, total)?;

        let mut shares: Vec<Share> = vec![
            Share {
                number: 0,
                data: vec![]
            };
            total
        ];

        let mut data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&mut data, output)?;

        // Corrupt 1 share
        shares[1].data[1] = b'?';

        let result_data = f.decode([].to_vec(), shares)?;

        assert_eq!(String::from_utf8(result_data)?, "hello, world! __");
        Ok(())
    }

    #[test]
    fn test_encode_decode_two_corruptions() -> Result<(), Box<dyn std::error::Error>> {
        let required = 4;
        let total = 8;
        let f = FEC::new(required, total)?;

        let mut shares: Vec<Share> = vec![
            Share {
                number: 0,
                data: vec![]
            };
            total
        ];

        let mut data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&mut data, output)?;

        //Corrupt 2 shares
        shares[0].data[0] = b'?';
        shares[1].data[0] = b'?';

        // The following corruptions are redundant
        shares[0].data[1] = b'?';
        shares[0].data[2] = b'?';
        shares[0].data[3] = b'?';

        let result_data = f.decode([].to_vec(), shares)?;

        assert_eq!(String::from_utf8(result_data)?, "hello, world! __");
        Ok(())
    }

    #[test]
    fn test_encode_decode_three_corruptions_should_fail() {
        let required = 4;
        let total = 8;
        let f = FEC::new(required, total).unwrap();

        let mut shares: Vec<Share> = vec![
            Share {
                number: 0,
                data: vec![]
            };
            total
        ];

        let mut data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&mut data, output).unwrap();

        // Corrupt 3 shares

        shares[0].data[0] = b'?';
        shares[1].data[0] = b'?';
        shares[2].data[0] = b'?';

        let result_data = f.decode([].to_vec(), shares);

        // Expect an error due to too many corruptions
        assert!(result_data.is_err());
    }
}
