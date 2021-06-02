// This test uses unsanitised input to decide control flow in a program.
// We do not consider an if condition to be a taint sink, so this program should not throw an error.

#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let a = input();
    let b;
    if a < 5 {
        b = 1
    } else {
        b = 2
    }
    output(b);
}

#[taint::source]
fn input() -> i32 {
    3
}

#[taint::sink]
fn output<T>(_: T) {
    ()
}
