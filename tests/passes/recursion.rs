fn main() {
    recurse_decrement(10);
}

fn recurse_decrement(i: i32) -> i32 {
    if i > 0 {
        recurse_decrement(i - 1)
    } else {
        0
    }
}
