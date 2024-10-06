use crate::fec::fec::*;
use reed_solomon_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the total number of shares and create a buffer to store them.
    let required = 4;
    let total = 8;

    // This is a *FEC which required 'required' pieces for reconstruction at minimum
    // and generate 'total' total pieces
    let f = FEC::new(required, total)?;

    let mut shares: Vec<Share> = vec![
        Share {
            number: 0,
            data: vec![]
        };
        total
    ]; // Initializes with default Share instances

    // The data to encode, needs to be padded to multiple of required
    let data = b"hello, world! __".to_vec();

    for i in 0..total {
        println!("Share {}: {:?}", i, shares[i]);
    }

    {
        let output = |s: Share| {
            shares[s.number] = s.clone(); // deep copy
        };
        f.encode(&data, output)?;
    }
    for i in 0..total {
        println!("Share {}: {:?}", i, shares[i]);
    }

    // TODO corrupt the data maybe

    let data = f.decode([].to_vec(), shares)?;

    for i in 0..data.0.len() {
      println!("Share {}: {:?}", i, data.0[i]);
    }
    Ok(())
}
