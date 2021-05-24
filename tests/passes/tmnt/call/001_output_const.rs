#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    output(1);
}

#[taint::sink]
fn output(_: i32) {
    ()
}
