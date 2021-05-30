use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use rustc_index::bit_set::BitSet;
use rustc_middle::{
    mir::{
        visit::Visitor, BasicBlock, Body, Constant, HasLocalDecls, Local, Location, Operand, Place,
        Rvalue, Statement, StatementKind, Terminator, TerminatorKind,
    },
    ty::{TyCtxt, TyKind},
};

use rustc_mir::dataflow::{Analysis, AnalysisDomain, Forward};
use rustc_span::Span;

use tracing::instrument;

use crate::eval::attributes::{AttrInfo, AttrInfoKind};

use super::taint_domain::{PointsAwareTaintDomain, TaintDomain};

pub(crate) type PointsMap = HashMap<Local, HashSet<Local>>;

type InitSet = Vec<Option<bool>>;

/// A dataflow analysis that tracks whether a value may carry a taint.
///
/// Taints are introduced through sources, and consumed by sinks.
/// Ideally, a sink never consumes a tainted value - this should result in an error.
pub struct TaintAnalysis<'tcx, 'inter> {
    tcx: TyCtxt<'tcx>,
    info: &'inter AttrInfo,
    init: InitSet,
    points: RefCell<PointsMap>,
}

impl<'tcx, 'inter> TaintAnalysis<'tcx, 'inter> {
    /// Call on `main` function
    pub fn new(tcx: TyCtxt<'tcx>, info: &'inter AttrInfo) -> Self {
        Self::new_with_init(tcx, info, InitSet::new())
    }

    /// Call on dependencies
    #[inline]
    fn new_with_init(tcx: TyCtxt<'tcx>, info: &'inter AttrInfo, init: InitSet) -> Self {
        TaintAnalysis {
            tcx,
            info,
            init,
            points: RefCell::new(PointsMap::new()),
        }
    }
}

struct TransferFunction<'tcx, 'inter, 'intra> {
    tcx: TyCtxt<'tcx>,
    info: &'inter AttrInfo,
    state: &'intra mut PointsAwareTaintDomain<'intra, Local>,
}

impl<'inter> AnalysisDomain<'inter> for TaintAnalysis<'_, '_> {
    type Domain = BitSet<Local>;
    const NAME: &'static str = "TaintAnalysis";

    type Direction = Forward;

    fn bottom_value(&self, body: &Body<'inter>) -> Self::Domain {
        // bottom = definitely untainted
        BitSet::new_empty(body.local_decls().len())
    }

    fn initialize_start_block(&self, body: &Body<'inter>, state: &mut Self::Domain) {
        // For the main function, locals all start out untainted.
        // For other functions, however, we must check if they receive tainted parameters.
        if !self.init.is_empty() {
            for (_, arg) in self
                .init
                .iter()
                .zip(body.args_iter())
                .filter(|(&t, _)| t.unwrap_or(false))
            {
                state.set_taint(arg, true);
            }
        }
    }
}

impl<'tcx, 'inter, 'intra> Analysis<'intra> for TaintAnalysis<'tcx, 'inter> {
    fn apply_statement_effect(
        &self,
        state: &mut Self::Domain,
        statement: &Statement<'intra>,
        location: Location,
    ) {
        TransferFunction {
            tcx: self.tcx,
            info: self.info,
            state: &mut PointsAwareTaintDomain {
                state,
                map: &mut self.points.borrow_mut(),
            },
        }
        .visit_statement(statement, location);
    }

    fn apply_terminator_effect(
        &self,
        state: &mut Self::Domain,
        terminator: &Terminator<'intra>,
        location: Location,
    ) {
        TransferFunction {
            tcx: self.tcx,
            info: self.info,
            state: &mut PointsAwareTaintDomain {
                state,
                map: &mut self.points.borrow_mut(),
            },
        }
        .visit_terminator(terminator, location);
    }

    fn apply_call_return_effect(
        &self,
        _state: &mut Self::Domain,
        _block: BasicBlock,
        _func: &Operand<'intra>,
        _args: &[Operand<'intra>],
        _return_place: Place<'intra>,
    ) {
        // do nothing
    }
}

impl<'tcx> std::fmt::Debug for TransferFunction<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.state))
    }
}

impl<'inter> Visitor<'inter> for TransferFunction<'_, '_, '_> {
    fn visit_statement(&mut self, statement: &Statement<'inter>, _: Location) {
        let Statement { source_info, kind } = statement;

        self.visit_source_info(source_info);

        if let StatementKind::Assign(box (ref place, ref rvalue)) = kind {
            self.t_visit_assign(place, rvalue);
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator<'inter>, _: Location) {
        let Terminator { source_info, kind } = terminator;

        self.visit_source_info(source_info);

        match kind {
            TerminatorKind::Goto { .. } => {}
            TerminatorKind::SwitchInt { .. } => {}
            TerminatorKind::Return => {}
            TerminatorKind::Call {
                func: Operand::Constant(ref c),
                args,
                destination,
                fn_span,
                ..
            } => {
                self.t_visit_call(c, args, destination, fn_span);
            }
            TerminatorKind::Assert { .. } => {}
            _ => {}
        }
    }
}

impl<'long> TransferFunction<'_, '_, '_>
where
    Self: Visitor<'long>,
{
    #[instrument]
    fn t_visit_assign(&mut self, place: &Place, rvalue: &Rvalue) {
        match rvalue {
            // If we assign a constant to a place, the place is clean.
            Rvalue::Use(Operand::Constant(_)) | Rvalue::UnaryOp(_, Operand::Constant(_)) => {
                self.state.set_taint(place.local, false)
            }

            // Otherwise we propagate the taint
            Rvalue::Use(Operand::Copy(f) | Operand::Move(f)) => {
                self.state.propagate(f.local, place.local);
            }

            Rvalue::BinaryOp(_, box b) | Rvalue::CheckedBinaryOp(_, box b) => match b {
                (Operand::Constant(_), Operand::Constant(_)) => {
                    self.state.set_taint(place.local, false);
                }
                (Operand::Copy(a) | Operand::Move(a), Operand::Copy(b) | Operand::Move(b)) => {
                    if self.state.get_taint(a.local) || self.state.get_taint(b.local) {
                        self.state.set_taint(place.local, true);
                    } else {
                        self.state.set_taint(place.local, false);
                    }
                }
                (Operand::Copy(p) | Operand::Move(p), Operand::Constant(_))
                | (Operand::Constant(_), Operand::Copy(p) | Operand::Move(p)) => {
                    self.state.propagate(p.local, place.local);
                }
            },
            Rvalue::UnaryOp(_, Operand::Move(p) | Operand::Copy(p)) => {
                self.state.propagate(p.local, place.local);
            }
            Rvalue::Ref(_region_kind, _borrow_kind, p) => {
                self.state.add_ref(place, p);
            }

            Rvalue::Repeat(_, _) => {}
            Rvalue::ThreadLocalRef(_) => {}
            Rvalue::AddressOf(_, _) => {}
            Rvalue::Len(_) => {}
            Rvalue::Cast(_, _, _) => {}
            Rvalue::NullaryOp(_, _) => {}
            Rvalue::Discriminant(_) => {}
            Rvalue::Aggregate(_, _) => {}
        }
    }

    #[instrument]
    fn t_visit_call(
        &mut self,
        func: &Constant,
        args: &[Operand],
        destination: &Option<(Place, BasicBlock)>,
        span: &Span,
    ) {
        let name = func.to_string();
        let id = match func.literal.ty().kind() {
            TyKind::FnDef(id, _args) => Some(id),
            _ => None,
        }
        .unwrap();

        match self.info.get_kind(id) {
            Some(AttrInfoKind::Source) => self.t_visit_source_destination(destination),
            Some(AttrInfoKind::Sanitizer) => self.t_visit_sanitizer_destination(destination),
            Some(AttrInfoKind::Sink) => self.t_visit_sink(name, args, span),
            None => self.t_visit_analysis(args, id, destination),
        }
    }

    fn t_visit_analysis(
        &mut self,
        args: &[Operand],
        id: &rustc_hir::def_id::DefId,
        destination: &Option<(Place, BasicBlock)>,
    ) {
        let init = args
            .iter()
            .map(|arg| match arg {
                Operand::Copy(p) | Operand::Move(p) => Some(self.state.get_taint(p.local)),
                Operand::Constant(_) => None,
            })
            .collect::<Vec<_>>();
        let target_body = self.tcx.optimized_mir(*id);
        let mut results = TaintAnalysis::new_with_init(self.tcx, self.info, init)
            .into_engine(self.tcx, target_body)
            .pass_name("taint_analysis")
            .iterate_to_fixpoint()
            .into_results_cursor(target_body);
        if let Some(last) = target_body.basic_blocks().last() {
            results.seek_to_block_end(last);
            let end_state = results.get();

            // Check if return place is tainted, in which case this is a typical sink.
            if end_state.get_taint(Local::from_usize(0)) {
                self.t_visit_source_destination(destination);
            }

            let arg_map = args
                .iter()
                .map(|arg| {
                    arg.place()
                        .expect("constant submitted to function call")
                        .local
                })
                .zip(target_body.args_iter())
                .collect::<Vec<_>>();

            // Check if any variables which were passed in are tainted at this point.
            for (caller_arg, callee_arg) in arg_map {
                self.state
                    .set_taint(caller_arg, end_state.get_taint(callee_arg));
            }
        }
    }

    fn t_visit_source_destination(&mut self, destination: &Option<(Place, BasicBlock)>) {
        if let Some((place, _)) = destination {
            self.state.set_taint(place.local, true);
        }
    }

    fn t_visit_sanitizer_destination(&mut self, destination: &Option<(Place, BasicBlock)>) {
        if let Some((place, _)) = destination {
            self.state.set_taint(place.local, false);
        }
    }

    fn t_visit_sink(&mut self, name: String, args: &[Operand], span: &Span) {
        if args.iter().map(|op| op.place()).any(|el| {
            if let Some(place) = el {
                self.state.get_taint(place.local)
            } else {
                false
            }
        }) {
            self.tcx.sess.emit_err(super::errors::TaintedSink {
                fn_name: name,
                span: *span,
            });
        }
    }
}
