fn main() {
    let a = input();
    let b = a + 3;
    output(b) //~ ERROR function `output` received tainted input [T0001]
}

fn input() -> i32 {
    4
}

fn output(_: i32) {
    ()
}
