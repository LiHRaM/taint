// Test that our analysis does not fail on the `sanitizer` attribute.

#![feature(register_tool)]
#![register_tool(taint)]

#[taint::sanitizer]
fn main() {}
