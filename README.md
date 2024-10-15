# Reed-Solomon Error Correction in Rust

This library provides a Rust implementation of Reed-Solomon Error Correction Codes (RS-ECC), allowing encoding and decoding of data for error correction and recovery. It uses the Berlekamp Welch Algorithm for error correction and decoding.  

## Usage

To use this library in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
reed_solomon = { git = "https://github.com/SohamJog/reed_solomon_rs" }
```

# Testing
To run the tests, run 
```
cargo test
```

# License
This project is licensed under the MIT license. See the `LICENSE` file for more details

# Acknowledgements
This library is based on the Go library [Infectious](https://github.com/vivint/infectious).



