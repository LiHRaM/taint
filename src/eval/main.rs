use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_mir::dataflow::Analysis;

use crate::eval::attributes::TaintAttributeFinder;
use crate::taint_analysis::TaintAnalysis;

pub struct TaintConfig {
    pub ownership: bool,
}

impl Default for TaintConfig {
    fn default() -> Self {
        TaintConfig { ownership: false }
    }
}

pub fn eval_main(tcx: TyCtxt<'_>, main_id: DefId, config: TaintConfig) -> Option<i64> {
    let mut finder = TaintAttributeFinder::new(tcx);
    tcx.hir().krate().visit_all_item_likes(&mut finder);

    let _ = config;
    let entry = tcx.optimized_mir(main_id);

    let _ = TaintAnalysis::new(tcx, &finder.info)
        .into_engine(tcx, entry)
        .pass_name("taint_analysis")
        .iterate_to_fixpoint();

    None
}
