extern crate taint;

use taint::{sink, source};

fn main() {
    let val = input();
    output(val); //~ ERROR function `output` received tainted input [T0001]
}

#[source]
fn input() -> i32 {
    15
}

#[sink]
fn output(_: i32) {
    ()
}
