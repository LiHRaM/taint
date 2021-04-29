fn main() {
    let val = seems_safe();
    output(val); //~ ERROR
}

fn seems_safe() -> i32 {
    input()
}

fn input() -> i32 {
    15
}

fn output(_: i32) {}
