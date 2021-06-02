// Test that we reject annotations in the `taint` namespace that aren't supported.

#![feature(register_tool)]
#![register_tool(taint)]

#[taint::not_valid] //~ ERROR Taint attribute `not_valid` is invalid. We currently only support `source`, `sink`, and `sanitizer` [T0002]
fn main() {}
