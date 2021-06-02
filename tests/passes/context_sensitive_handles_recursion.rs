// Test that the context sensitive functionality
// does not loop on mutually recursive functions.

fn main() {
    recurse_decrement_even(10);
}

fn recurse_decrement_even(i: i32) -> i32 {
    if i > 0 {
        recurse_decrement_odd(i - 1)
    } else {
        0
    }
}

fn recurse_decrement_odd(i: i32) -> i32 {
    if i > 0 {
        recurse_decrement_even(i - 1)
    } else {
        0
    }
}
