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
