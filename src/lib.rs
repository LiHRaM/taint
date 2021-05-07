#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(box_patterns)]

extern crate rustc_apfloat;
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_mir;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

mod analysis;
mod summaries;

pub mod eval;

pub use analysis::*;
pub use annotations::*;
pub use summaries::*;
