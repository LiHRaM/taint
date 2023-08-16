use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_mir_dataflow::Analysis;

use crate::eval::attributes::TaintAttributeFinder;
use crate::taint_analysis::TaintAnalysis;

pub fn eval_main(tcx: TyCtxt<'_>, main_id: DefId) {
    // Find all functions in the current crate that have been tagged
    let mut finder = TaintAttributeFinder::new(tcx);
    tcx.hir().visit_all_item_likes_in_crate(&mut finder);

    let entry = tcx.optimized_mir(main_id);

    let _ = TaintAnalysis::new(tcx, &finder.info)
        .into_engine(tcx, entry)
        .pass_name("taint_analysis")
        .iterate_to_fixpoint();
}
