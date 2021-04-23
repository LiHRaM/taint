use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_mir::dataflow::Analysis;

use crate::analysis::MaybeTaintedLocals;

pub struct TaintConfig {
    pub ownership: bool,
}

impl Default for TaintConfig {
    fn default() -> Self {
        TaintConfig { ownership: false }
    }
}

pub fn eval_main<'tcx>(tcx: TyCtxt<'tcx>, main_id: DefId, config: TaintConfig) -> Option<i64> {
    let _ = tcx;
    let _ = main_id;
    let _ = config;

    let body = tcx.optimized_mir(main_id);

    let analysis = MaybeTaintedLocals {};
    let mut results = analysis
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .into_results_cursor(body);

    for (bb, _) in body.basic_blocks().iter_enumerated() {
        results.seek_before_primary_effect(body.terminator_loc(bb));
        let state = results.get();
        dbg!(state);
    }
    Some(0)
}
