use std::collections::HashMap;

use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_middle::{mir::Body, ty::TyCtxt};
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

    let mir_bodies: HashMap<DefId, &Body> = {
        let mut res = HashMap::new();
        for local_def_id in tcx.mir_keys(LOCAL_CRATE) {
            let def_id = local_def_id.to_def_id();
            res.insert(def_id, tcx.optimized_mir(def_id));
        }
        res
    };

    let _ = TaintAnalysis::new(tcx, &finder.info)
        .into_engine(tcx, mir_bodies.get(&main_id).unwrap())
        .pass_name("taint_analysis")
        .iterate_to_fixpoint();

    None
}
