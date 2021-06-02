// Test the most basic propagation of a taint.
// Since input and output are both annotated functions,
// we do not analyze their MIR.

#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let val = input();
    output(val); //~ ERROR function `output` received tainted input [T0001]
}

#[taint::source]
fn input() -> i32 {
    15
}

#[taint::sink]
fn output(_: i32) {
    ()
}
