# Taint Analysis

This project uses the Rust compiler to implement a taint analysis, hooking into the compiler to process an intermediate representation of the code.

## Setting up:

We use compiler internals through `#![feature(rustc_private)]`.
To use those compiler internals, we must use the nightly toolchain, with some additional components.
These are all defined in the "rust-toolchain" file, and automatically recognised by cargo.

To test that the binary correctly hooks into and runs the compiler, run the following command:
```
cargo test
```

### Running the compiler

`rustc_driver` allows us to run the compiler, and `rustc_interface` provides APIs for hooking into the right places to perform the analysis.

- https://rustc-dev-guide.rust-lang.org/rustc-driver.html
- https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html