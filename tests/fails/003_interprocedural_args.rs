#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let mut buffer = 0;
    set_buffer(&mut buffer);
    output(buffer); //~ ERROR
}

fn set_buffer(buffer: &mut i32) {
    *buffer = input();
}

#[taint::source]
fn input() -> i32 {
    15
}

#[taint::sink]
fn output(_: i32) {
    ()
}
