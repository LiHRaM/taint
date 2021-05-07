extern crate taint;

use taint::{sink, source};

fn main() {
    let val = seems_safe();
    output(val); //~ ERROR
}

fn seems_safe() -> i32 {
    input()
}

#[source]
fn input() -> i32 {
    15
}

#[sink]
fn output(_: i32) {}
