//! Helper functions for implementing the compiler callbacks necessary for taint analysis.

use rustc_hir as hir;
use rustc_session::config::ErrorOutputType;

use hir::def_id::LOCAL_CRATE;
use taint::eval;

use std::convert::TryFrom;

/// Perform the taint analysis.
pub fn mir_analysis(tcx: rustc_middle::ty::TyCtxt) {
        let (entry_def_id, _) = if let Some((entry_def, x)) = tcx.entry_fn(LOCAL_CRATE) {
        (entry_def, x)
    } else {
        err_msg(
            "this tool currently only supports taint analysis on programs with a main function",
        );
    };

    let main_id = entry_def_id.to_def_id();
    let config = eval::TaintConfig::default();

    if let Some(return_code) = eval::eval_main(tcx, main_id, config) {
        std::process::exit(i32::try_from(return_code).expect("Return value was too large!"));
    }
}

fn err_msg(msg: &str) -> ! {
    rustc_session::early_error(ErrorOutputType::default(), msg);
}

