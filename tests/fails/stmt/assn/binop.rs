#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let a = input();
    let b = a + 3;
    output(b) //~ ERROR function `output` received tainted input [T0001]
}

#[taint::source]
fn input() -> i32 {
    4
}

#[taint::sink]
fn output(_: i32) {
    ()
}
