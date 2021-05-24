#![feature(register_tool)]
#![register_tool(taint)]


fn main() {
    let mut _x = input();
    _x = !false;
    output(_x);
}

#[taint::source]
fn input() -> bool {
    true
}

#[taint::sink]
fn output(_: bool) {}
