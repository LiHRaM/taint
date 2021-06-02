// Test that our analysis does not fail on the `source` attribute.

#![feature(register_tool)]
#![register_tool(taint)]

#[taint::source]
fn main() {}
