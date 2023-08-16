use hir::intravisit::Visitor;
use rustc_ast::AttrKind;
use rustc_errors::struct_span_err;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;

/// Find all attributes in a crate which originate from the `taint` tool.
pub struct TaintAttributeFinder<'tcx> {
    tcx: TyCtxt<'tcx>,
    pub(crate) info: AttrInfo,
}

#[derive(Default, Debug)]
pub struct AttrInfo {
    pub sources: Vec<DefId>,
    pub sinks: Vec<DefId>,
    pub sanitizers: Vec<DefId>,
}

#[derive(Debug)]
pub enum AttrInfoKind {
    Source,
    Sink,
    Sanitizer,
}

impl AttrInfo {
    pub fn get_kind(&self, id: &DefId) -> Option<AttrInfoKind> {
        if self.sources.contains(id) {
            Some(AttrInfoKind::Source)
        } else if self.sinks.contains(id) {
            Some(AttrInfoKind::Sink)
        } else if self.sanitizers.contains(id) {
            Some(AttrInfoKind::Sanitizer)
        } else {
            None
        }
    }
}

impl<'tcx> TaintAttributeFinder<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        TaintAttributeFinder {
            tcx,
            info: AttrInfo::default(),
        }
    }
}

impl TaintAttributeFinder<'_> {
    fn visit_hir_id(&mut self, item_id: hir::HirId) {
        let sym_source = Symbol::intern("source");
        let sym_sink = Symbol::intern("sink");
        let sym_sanitizer = Symbol::intern("sanitizer");

        let def_id = item_id.owner.to_def_id();
        let attrs = self.tcx.hir().attrs(item_id);
        for attr in attrs {
            if let AttrKind::Normal(ref kind) = attr.kind {
                let item = &kind.item;
                if let Some(symbol) = get_taint_attr(item) {
                    if symbol == &sym_source {
                        self.info.sources.push(def_id)
                    } else if symbol == &sym_sink {
                        self.info.sinks.push(def_id)
                    } else if symbol == &sym_sanitizer {
                        self.info.sanitizers.push(def_id)
                    } else {
                        struct_span_err!(self.tcx.sess, item.span(), T0002, "Taint attribute `{}` is invalid. We currently only support `source`, `sink`, and `sanitizer`", symbol.to_ident_string()).emit();
                    };
                    break;
                }
            }
        }
    }
}

impl<'v> Visitor<'v> for TaintAttributeFinder<'_> {
    fn visit_item(&mut self, item: &'v rustc_hir::Item<'_>) {
        self.visit_hir_id(item.hir_id());
    }

    fn visit_trait_item(&mut self, trait_item: &rustc_hir::TraitItem<'_>) {
        self.visit_hir_id(trait_item.hir_id());
    }

    fn visit_impl_item(&mut self, impl_item: &rustc_hir::ImplItem<'_>) {
        self.visit_hir_id(impl_item.hir_id());
    }

    fn visit_foreign_item(&mut self, foreign_item: &rustc_hir::ForeignItem<'_>) {
        self.visit_hir_id(foreign_item.hir_id());
    }
}

fn get_taint_attr(item: &rustc_ast::AttrItem) -> Option<&Symbol> {
    if item.path.segments.len() == 2 && item.path.segments[0].ident.name == Symbol::intern("taint")
    {
        Some(&item.path.segments[1].ident.name)
    } else {
        None
    }
}
