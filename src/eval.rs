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

pub fn eval_main(tcx: TyCtxt<'_>, main_id: DefId, config: TaintConfig) -> Option<i64> {
    let _ = config;

    let body = tcx.optimized_mir(main_id);

    let analysis = MaybeTaintedLocals::new(tcx.sess);
    let mut _results = analysis.into_engine(tcx, body).iterate_to_fixpoint();

    Some(0)
}
