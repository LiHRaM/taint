fn main() {
    let val = input();
    output(val); //~ ERROR function `output` received tainted input [T0001]
}

fn input() -> i32 {
    15
}

fn output(_: i32) {
    ()
}
