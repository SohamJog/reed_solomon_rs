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
```bash
cargo test
```

# Benchmarks
To run the benchmarks, run 
```bash
cargo bench
```

# Reed-Solomon Error-Correction Algorithm Overview
Reed-Solomon Error Correction Codes (RS-ECC) encode data into multiple shares, allowing recovery even if some shares are lost or corrupted. This library uses polynomial encoding and the Berlekamp-Welch algorithm to reconstruct and correct data efficiently. RS-ECC is widely applied in technologies such as CDs, DVDs, QR codes, satellite communications, DSL, WiMAX, and RAID 6 storage systems, where data integrity is essential.

### Reed-Solomon Encoding Algorithm Summary

Reed-Solomon encoding divides data into *k* original shares, generating *n* total shares by adding *n - k* redundant shares. This enables data recovery even if some shares are lost or corrupted. Each share represents a point on a polynomial of degree *k - 1* over a finite field (GF(256) for this library ).

The encoding process works as follows:

1. **Divide Data into Shares**: The input data is split into *k* shares and used as coefficients of a polynomial $$P(x)$$ of degree *k - 1*.

2. **Create Polynomial**: This polynomial, $$P(x)$$, is evaluated at *n* distinct points, yielding *n* shares. Each point corresponds to a unique share, making it possible to reconstruct the original polynomial from any subset of *k* shares.

3. **Redundant Data for Error Correction**: Using Lagrange interpolation, any *k* shares can recover the polynomial $$P(x)$$, which encodes the original data. The algorithm’s error correction capability, *t*, is determined by the number of redundant shares, allowing correction of up to *t* errors, where $$t = \frac{n - k}{2}$$.

This approach ensures that the encoded data can withstand loss or corruption of up to *n - k* shares, making Reed-Solomon encoding ideal for scenarios requiring robust error correction.


### Berlekamp-Welch Decoding Algorithm Summary (Error Correction)

The Berlekamp-Welch algorithm decodes Reed-Solomon codes by identifying both the original message polynomial and an error-locator polynomial, which helps determine the positions of corrupted symbols. Here’s a step-by-step summary of the algorithm:

1. **Input Parameters**:
   - The total number of shares (symbols) received, *n*.
   - The degree of the original message polynomial, *k - 1*, where *k* is the minimum number of shares required to reconstruct the message.
   - The maximum number of correctable errors, *t*, which satisfies $$t = \frac{n - k}{2}$$.

2. **Define Polynomials**:
   - The message polynomial, $$P(x)$$, represents the original data.
   - An error-locator polynomial, $$E(x)$$, which zeros out the positions of errors in the received data.
   - An error-evaluator polynomial, $$Q(x)$$, that equals the product $$P(x) \cdot E(x)$$ at all non-erroneous points.

3. **Construct Key Equations**:
   - For each share $$(a_i, b_i)$$, where $$a_i$$ is the evaluation point and $$b_i$$ is the received value, we set up equations  $$b_i \cdot E(a_i) = Q(a_i)$$.
   - These form a system of linear equations involving the coefficients of $$E(x)$$ and $$Q(x)$$.

4. **Solve the System**:
   - Use Gaussian elimination (or other linear system-solving techniques) to solve for the coefficients of $$E(x)$$ and $$Q(x)$$.
   - This step has a time complexity of $$O(n^3)$$, where *n* is the number of shares.

5. **Recover the Message Polynomial**:
   - Divide $$Q(x)$$ by $$E(x)$$ to recover the original message polynomial $$P(x)$$.
   - Evaluate $$P(x)$$ at the error-free points to reconstruct the message.

6. **Error Correction**:
   - Use the roots of $$E(x)$$ to identify erroneous positions and correct them by recalculating their values using $$P(x)$$.

This algorithm reconstructs the original data while correcting errors up to *t* corruptions in the shares.


# Implementation Details

This library operates over **GF(256)**, supporting finite field operations for efficient error correction. The data structure for each share is defined as follows:

```rust
pub struct Share {
    /// Number is essentially the X-coordinate on the encoding polynomial
    pub number: usize,
    /// Encoded data
    pub data: Vec<u8>,
}
```

### Initialization
To set up a forward error correcting (FEC) object, you can initialize it by specifying the required number of shares and the total number of shares to be generated:
```rust
let required = 4;
let total = 8;
let f = FEC::new(required, total)?;
```

### Encoding
To encode data into shares, ensure that your data vector is divisible by the required number of shares. If not, it will be padded with underscores during encoding. Here’s how to encode:
```rust
let data: Vec<u8> = b"hello, world! __1234567".to_vec(); // Example data
let output = |s: Share| {
    shares[s.number] = s.clone(); // Deep copy
};
f.encode(&data, output)?;
```

### Decoding
Decoding is straightforward. Assuming you have a vector of shares, you can decode them like this:
```rust
let data = f.decode([].to_vec(), shares)?;
```

### Encoding and Decoding Limits

The limits for encoding and decoding are governed by the parameters:

- **n**: Total number of shares.
- **k**: Minimum number of shares required for recovery.

To enable error correction, it is necessary that $n > k$. The maximum number of correctable errors $t$ can be calculated as:

$$ t = \frac{n - k}{2} $$

# License
This project is licensed under the MIT license. See the `LICENSE` file for more details

# Acknowledgements
This library is based on the Go library [Infectious](https://github.com/vivint/infectious).



