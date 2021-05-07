use proc_macro::*;

#[proc_macro_attribute]
pub fn sink(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn source(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}
