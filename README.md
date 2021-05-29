# Taint Analysis

This project is a static [taint analysis](https://en.wikipedia.org/wiki/Taint_checking) tool for the Rust programming language.
We use Rust compiler internals to inspect [MIR](https://rustc-dev-guide.rust-lang.org/mir/), which is an intermediate representation of Rust, useful for [dataflow analysis](https://en.wikipedia.org/wiki/Data-flow_analysis).

## Examples

For examples of how this tool can be used, and what the expected results would be, please have a look at the `tests/` folder.
We have examples of program which should emit no errors, and programs where the taint analysis should detect a sink receiving possibly tainted data.

## Setting Up

We use the `rust-toolchain` file to manage which version of the compiler we use, as well as any additional components.
Since this project uses compiler internals and the `#![feature(rustc_private)]` feature, we must use nightly.
Cargo should automatically recognize the toolchain file, and download the necessary toolchain and components when you build the project.

## Tests

We have tried to make sure that running tests does not deviate from the typical Rust project experience, and should be as simple as typing in the following command:

```
cargo test
```

## Licensing

We use the MIT license, available in the `LICENSE` file.

## Compiler Internals

`rustc_driver` allows us to run the compiler, and `rustc_interface` provides APIs for hooking into the right places to perform the analysis.

- https://rustc-dev-guide.rust-lang.org/rustc-driver.html
- https://doc.rust-lang.org/nightly/nightly-rustc/rustc_interface/index.html
