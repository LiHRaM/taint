#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let val = seems_safe();
    output(val); //~ ERROR
}

fn seems_safe() -> i32 {
    input()
}

#[taint::source]
fn input() -> i32 {
    15
}

#[taint::sink]
fn output(_: i32) {}
