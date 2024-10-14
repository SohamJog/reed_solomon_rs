use crate::fec::fec::*;

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

        let data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&data, output)?;

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

        let data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&data, output)?;

        // Corrupt 1 share
        shares[1].data[1] = b'?';

        let result_data = f.decode([].to_vec(), shares)?;

        assert_eq!(String::from_utf8(result_data)?, "hello, world! __");
        Ok(())
    }

    #[test]
    fn test_encode_decode_four_corruptions() -> Result<(), Box<dyn std::error::Error>> {
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

        let data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&data, output)?;

        // Corrupt 4 shares
        for i in 1..3 {
            for j in 1..3 {
                shares[i].data[j] = b'?'
            }
         }

        let result_data = f.decode([].to_vec(), shares)?;

        assert_eq!(String::from_utf8(result_data)?, "hello, world! __");
        Ok(())
    }

    #[test]
    fn test_encode_decode_five_corruptions_should_fail() {
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

        let data = b"hello, world! __".to_vec();

        let output = |s: Share| {
            shares[s.number] = s.clone();
        };

        f.encode(&data, output).unwrap();

        // Corrupt 5 shares
        for i in 1..3 {
            for j in 1..3 {
                shares[i].data[j] = b'?'
            }
         }
         shares[0].data[0] = b'?';
         shares[0].data[1] = b'?';

        let result_data = f.decode([].to_vec(), shares);

        // Expect an error due to too many corruptions
        assert!(result_data.is_err());
    }


}
