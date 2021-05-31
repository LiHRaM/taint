#![feature(register_tool)]
#![register_tool(taint)]

#[taint::not_valid] //~ ERROR
fn main() {}
