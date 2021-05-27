#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let val = get_input();
    output(val);
}

fn get_input() -> i32 {
    input()
}

#[taint::source]
fn input() -> i32 {
    15
}

#[taint::sink]
fn output(_: i32) {
    ()
}
