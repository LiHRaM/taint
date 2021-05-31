#![feature(register_tool)]
#![register_tool(taint)]

struct ImplStruct;

fn main() {
    let val = ImplStruct::input();
    ImplStruct::output(val); //~ ERROR
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
