// Test whether we recognize annotations within `impl` blocks.

#![feature(register_tool)]
#![register_tool(taint)]

struct ImplStruct;

fn main() {
    let val = ImplStruct::input();
    ImplStruct::output(val); //~ ERROR function `ImplStruct::output` received tainted input [T0001]
}

impl ImplStruct {
    #[taint::source]
    fn input() -> i32 {
        15
    }

    #[taint::sink]
    fn output(_: i32) {
        ()
    }
}
