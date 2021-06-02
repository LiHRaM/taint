// Test that our analysis does not fail on the `sink` attribute.

#![feature(register_tool)]
#![register_tool(taint)]

#[taint::sink]
fn main() {}
