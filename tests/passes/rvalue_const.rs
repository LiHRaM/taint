#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let x = 1;
    output(x);
}

#[taint::sink]
fn output(_: i32) {}
