#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    let val = recursive_decrement(10);
    process(val); //~ERROR function `process` received tainted input
}

#[taint::source]
fn zero() -> i32 {
    0
}

#[taint::sink]
fn process(_: i32) {}

fn recursive_decrement(i: i32) -> i32 {
    if i > 0 {
        recursive_decrement(i - 1)
    } else {
        zero()
    }
}
