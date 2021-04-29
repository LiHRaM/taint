use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_middle::{mir::Body, ty::TyCtxt};
use rustc_mir::dataflow::Analysis;

use crate::{analysis::taint_analysis::TaintAnalysis, Summary, TaintProperty};

pub struct TaintConfig {
    pub ownership: bool,
}

impl Default for TaintConfig {
    fn default() -> Self {
        TaintConfig { ownership: false }
    }
}

/// Run the taint analysis, assuming that the `DefId` passed is the identifier of the main function in the program.
/// First, we create function signatures for the functions in the local crate.
/// Essentially, these function signatures specify:
/// Functions that always, never or sometimes return a tainted value.
/// Functions that sometimes return a tainted value must signify which factors influence the taint of the return value,
/// We will only support pure functions, i.e. whose taint status is either constant, or depends on functions passed as parameters.
pub fn eval_main(tcx: TyCtxt<'_>, main_id: DefId, config: TaintConfig) -> Option<i64> {
    let _ = config;
    let _ = tcx.mir_keys(LOCAL_CRATE);

    // Hardcode summaries for now
    let summaries: Vec<Summary> = vec![
        Summary {
            name: "output",
            is_source: TaintProperty::Never,
            is_sink: TaintProperty::Always,
        },
        Summary {
            name: "input",
            is_source: TaintProperty::Always,
            is_sink: TaintProperty::Never,
        },
        Summary {
            name: "seems_safe",
            is_source: TaintProperty::Always,
            is_sink: TaintProperty::Never,
        },
    ];

    let body: &Body = tcx.optimized_mir(main_id);

    let analysis = TaintAnalysis::new(tcx.sess, summaries);
    let mut _results = analysis.into_engine(tcx, body).iterate_to_fixpoint();

    None
}
