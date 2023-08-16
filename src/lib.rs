#![feature(rustc_private)]
#![feature(box_patterns)]

extern crate rustc_driver;
extern crate rustc_apfloat;
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_macros;
//extern crate rustc_builtin_macros;
extern crate rustc_fluent_macro;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

mod analysis;

pub mod eval;

pub use analysis::*;
// The fluent_messages! macro generates translations for diagnostic messages
// see https://rustc-dev-guide.rust-lang.org/diagnostics/translation.html
// The two imports below are required by this macro
use rustc_fluent_macro::fluent_messages;
use rustc_errors::{DiagnosticMessage, SubdiagnosticMessage};

fluent_messages! { "./messages.ftl" }
