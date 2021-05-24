use hir::itemlikevisit::ItemLikeVisitor;
use rustc_ast::AttrKind;
use rustc_hir as hir;
use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use rustc_middle::{mir::Body, ty::TyCtxt};
use rustc_mir::dataflow::Analysis;
use rustc_span::Symbol;

use crate::{errors::InvalidVariant, taint_analysis::TaintAnalysis};

pub struct TaintConfig {
    pub ownership: bool,
}

impl Default for TaintConfig {
    fn default() -> Self {
        TaintConfig { ownership: false }
    }
}

struct Finder<'tcx> {
    tcx: TyCtxt<'tcx>,
    decls: Option<hir::HirId>,
}

impl<'v> ItemLikeVisitor<'v> for Finder<'_> {
    fn visit_item(&mut self, item: &hir::Item<'_>) {
        let attrs = self
            .tcx
            .hir()
            .attrs(item.hir_id())
            .iter()
            .collect::<Vec<_>>();
        dbg!(attrs);
    }

    fn visit_trait_item(&mut self, _trait_item: &hir::TraitItem<'_>) {}

    fn visit_impl_item(&mut self, _impl_item: &hir::ImplItem<'_>) {}

    fn visit_foreign_item(&mut self, _foreign_item: &hir::ForeignItem<'_>) {}
}

pub fn eval_main(tcx: TyCtxt<'_>, main_id: DefId, config: TaintConfig) -> Option<i64> {
    let mut finder = TaintAttributeFinder::new(tcx);
    tcx.hir().krate().visit_all_item_likes(&mut finder);

    let _ = config;
    let _ = tcx.mir_keys(LOCAL_CRATE);

    let body: &Body = tcx.optimized_mir(main_id);

    let analysis = TaintAnalysis::new(tcx.sess, &finder.info);
    let mut _results = analysis.into_engine(tcx, body).iterate_to_fixpoint();

    None
}

/// Find all attributes in a crate which originate from the `taint` tool.
struct TaintAttributeFinder<'tcx> {
    tcx: TyCtxt<'tcx>,
    info: AttrInfo,
}

#[derive(Default)]
pub struct AttrInfo {
    pub sources: Vec<DefId>,
    pub sinks: Vec<DefId>,
    pub sanitizers: Vec<DefId>,
}

impl<'tcx> TaintAttributeFinder<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        TaintAttributeFinder {
            tcx,
            info: fun_name(),
        }
    }
}

fn fun_name() -> AttrInfo {
    AttrInfo {
        sources: vec![],
        sinks: vec![],
        sanitizers: vec![],
    }
}

impl<'v> ItemLikeVisitor<'v> for TaintAttributeFinder<'_> {
    fn visit_item(&mut self, item: &'v rustc_hir::Item<'_>) {
        let item_id = item.hir_id();
        let def_id = self.tcx.hir().local_def_id(item_id).to_def_id();

        let sym_source = Symbol::intern("source");
        let sym_sink = Symbol::intern("sink");
        let sym_sanitizer = Symbol::intern("sanitizer");

        let attrs = self.tcx.hir().attrs(item_id);
        for attr in attrs {
            if let AttrKind::Normal(ref item, _) = attr.kind {
                if let Some(symbol) = get_taint_attr(item) {
                    if symbol == &sym_source {
                        self.info.sources.push(def_id)
                    } else if symbol == &sym_sink {
                        self.info.sinks.push(def_id)
                    } else if symbol == &sym_sanitizer {
                        self.info.sanitizers.push(def_id)
                    } else {
                        self.tcx.sess.emit_err(InvalidVariant {
                            attr_name: symbol.to_ident_string(),
                            span: item.span(),
                        })
                    }

                    break;
                }
            }
        }
    }

    fn visit_trait_item(&mut self, _trait_item: &rustc_hir::TraitItem<'_>) {}

    fn visit_impl_item(&mut self, _impl_item: &rustc_hir::ImplItem<'_>) {}

    fn visit_foreign_item(&mut self, _foreign_item: &rustc_hir::ForeignItem<'_>) {}
}

fn get_taint_attr(item: &rustc_ast::AttrItem) -> Option<&Symbol> {
    if item.path.segments.len() == 2 && item.path.segments[0].ident.name == Symbol::intern("taint")
    {
        Some(&item.path.segments[1].ident.name)
    } else {
        None
    }
}
