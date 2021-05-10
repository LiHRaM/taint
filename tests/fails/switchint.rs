// This program takes unsanitised input in one branch of an if statement
// Since we cannot at compiletime say which branch will be taken, we must assume that b may be tainted
// and throw an error.

fn main() {
    //This input is not an issue, as we allow input to be used to decide control flow
    let a = input();
    let b;
    if a < 5 {
        b = input(); // This input is an issue, as b may be used in the output function.
    } else {
        b = 5;
    }
    output(b); //~ERROR function `output` received tainted input [T0001]
}

fn input() -> i32 {
    4
}

fn output<T>(_: T) {
    ()
}
