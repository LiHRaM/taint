# Taint Analysis

This project uses the Rust compiler to implement a taint analysis, hooking into the compiler to process an intermediate representation of the code.

## Setting up:

We use compiler internals through `#![feature(rustc_private)]`.
For now, we only support using the `rustup-toolchain` command at the root of this workspace.
It installs a custom toolchain with the necessary components to compile and run the taint binary.

To test that the binary correctly hooks into and runs the compiler, run the following command:

```
cargo test
```

### Running the compiler

`rustc_driver` allows us to run the compiler, and `rustc_interface` provides APIs for hooking into the right places to perform the analysis.

- https://rustc-dev-guide.rust-lang.org/rustc-driver.html
- https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html

Since `rustc_interface` is unstable, we need to make sure that the compiler toolchain being used is the one that is supported.