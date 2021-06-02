// Test that const values are not considered taints.

#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    output(1);
}

#[taint::sink]
fn output(_: i32) {
    ()
}
