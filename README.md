# Reed-Solomon Error Correction in Rust

This library provides a Rust implementation of Reed-Solomon Error Correction Codes (RS-ECC), allowing encoding and decoding of data for error correction and recovery. It uses the Berlekamp Welch Algorithm for error correction and decoding.  

# Usage

To use this library in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
reed_solomon = { git = "https://github.com/SohamJog/reed_solomon_rs" }
```

# Example

```rust
use reed_solomon_rs::fec::fec::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let required = 4; // Number of pieces required for reconstruction
    let total = 8;    // Total number of pieces to generate

    // Initialize the FEC (Forward Error Correction) instance
    let f = FEC::new(required, total)?;

    // Create a vector to hold the shares
    let mut shares: Vec<Share> = vec![Share { number: 0, data: vec![] }; total];

    // Data to encode
    let data = b"hello, world! __".to_vec();

    // Define the output closure to store generated shares
    let output = |s: Share| {
        shares[s.number] = s.clone(); // Deep copy of the share
    };

    // Encode the data into shares
    f.encode(&data, output)?;

    // Maybe corrupt a share
    shares[1].data[1] = b'?';

    // Decode the shares
    let result_data = f.decode([].to_vec(), shares)?;

    // Verify the result
    match String::from_utf8(result_data) {
        Ok(decoded_string) => {
            println!("Decoded data: {:?}", decoded_string);
        }
        Err(e) => {
            println!("Invalid UTF-8 sequence: {}", e);
        }
    }

    Ok(())
}

```

# Testing
To run the tests, run 
```
cargo test
```

# Benchmarks
To run the benchmarks, run 
```
cargo bench
```

# License
This project is licensed under the MIT license. See the `LICENSE` file for more details

# Acknowledgements
This library is based on the Go library [Infectious](https://github.com/vivint/infectious).



