use crate::fec::fec::*;
use reed_solomon_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let data = b"hello, world! __12345678".to_vec();

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

    // Corrupting the data
    //  for i in 0..4 {
    //     //for j in 1..3 {
    //         shares[i].data[i] = b'?'
    //     //}
    //  }

    //  for i in 0..8 {
    //         shares[i].data[i%4] = b'?';
    //  }

    // for i in 0..4 {
        
    // }
    shares[0].data[1] = b'?';
    shares[1].data[2] = b'?';
    shares[2].data[3] = b'?';
    shares[3].data[4] = b'?';
    shares[4].data[5] = b'?';
    // shares[5].data[5] = b'?';




    // shares[2].data[0] = b'?';

    //shares[0].data[1] = b'?';
    //  shares[0].data[2] = b'?';
    //  shares[0].data[3] = b'?';
    //shares[1].data[1] = b'?';

    let data = f.decode([].to_vec(), shares)?;

    for i in 0..data.len() {
        println!("Share {}: {:?}", i, data[i]);
    }
    match String::from_utf8(data) {
        Ok(s) => println!("got: {:?}", s),
        Err(e) => println!("Invalid UTF-8 sequence: {}", e),
    }
    Ok(())
}
