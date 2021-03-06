// This program takes unsanitised input in one branch of an if statement
// Since we cannot at compile time say which branch will be taken, we must assume that b may be tainted
// and throw an error.

#![feature(register_tool)]
#![register_tool(taint)]

fn main() {
    //This input is not an issue, as we allow input to be used to decide control flow
    let a = input();
    let b;
    if a < 5 {
        b = input(); // This input is an issue, as b may be used in the output function.
    } else {
        b = 5;
    }
    output(b); //~ ERROR function `output` received tainted input [T0001]
}

#[taint::source]
fn input() -> i32 {
    4
}

#[taint::sink]
fn output(_: i32) {
    ()
}
