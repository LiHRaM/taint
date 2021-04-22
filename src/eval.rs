use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

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

    Some(0)
}
