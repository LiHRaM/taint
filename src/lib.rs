#![feature(rustc_private)]
#![feature(box_syntax)]

extern crate rustc_apfloat;
extern crate rustc_ast;
#[allow(unused_imports)]
#[macro_use]
extern crate rustc_middle;
extern crate rustc_data_structures;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_mir;
extern crate rustc_span;
extern crate rustc_target;

mod analysis;

pub mod eval;
