use crate::{Infer, Mark, Summary, TaintType};

use rustc_index::bit_set::BitSet;
use rustc_middle::mir::{
    visit::Visitor, BasicBlock, Body, HasLocalDecls, Local, Location, Operand, Place, Rvalue,
    Statement, StatementKind, Terminator, TerminatorKind,
};

use rustc_mir::dataflow::{Analysis, AnalysisDomain, Forward};
use rustc_session::Session;
use rustc_span::Span;

use super::taint_domain::TaintDomain;

/// A dataflow analysis that tracks whether a value may carry a taint.
///
/// Taints are introduced through sources, and consumed by sinks.
/// Ideally, a sink never consumes a tainted value - this should result in an error.
pub struct TaintAnalysis<'tcx, 'analysis> {
    session: &'tcx Session,
    summaries: &'analysis [Summary<'tcx>],
}

impl<'tcx, 'analysis> TaintAnalysis<'tcx, 'analysis> {
    pub fn new(session: &'tcx Session, summaries: &'analysis [Summary<'tcx>]) -> Self {
        TaintAnalysis { session, summaries }
    }
}

impl<'tcx, 'analysis> AnalysisDomain<'tcx> for TaintAnalysis<'tcx, 'analysis> {
    type Domain = BitSet<Local>;
    const NAME: &'static str = "TaintAnalysis";

    type Direction = Forward;

    fn bottom_value(&self, body: &Body<'tcx>) -> Self::Domain {
        // bottom = untainted
        BitSet::new_empty(body.local_decls().len())
    }

    fn initialize_start_block(&self, _body: &Body<'tcx>, _state: &mut Self::Domain) {
        // Locals start out being untainted
    }
}

impl<'tcx, 'analysis> Analysis<'tcx> for TaintAnalysis<'tcx, 'analysis> {
    fn apply_statement_effect(
        &self,
        state: &mut Self::Domain,
        statement: &Statement<'tcx>,
        location: Location,
    ) {
        self.transfer_function(state)
            .visit_statement(statement, location);
    }

    fn apply_terminator_effect(
        &self,
        state: &mut Self::Domain,
        terminator: &Terminator<'tcx>,
        location: Location,
    ) {
        self.transfer_function(state)
            .visit_terminator(terminator, location);
    }

    fn apply_call_return_effect(
        &self,
        _state: &mut Self::Domain,
        _block: BasicBlock,
        _func: &Operand<'tcx>,
        _args: &[Operand<'tcx>],
        _return_place: Place<'tcx>,
    ) {
        // do nothing
    }
}

impl<'tcx, 'analysis> TaintAnalysis<'tcx, 'analysis> {
    fn transfer_function<T>(
        &'tcx self,
        state: &'tcx mut T,
    ) -> TransferFunction<'tcx, 'analysis, T> {
        TransferFunction {
            state,
            session: self.session,
            summaries: self.summaries,
        }
    }
}

struct TransferFunction<'tcx, 'analysis, T> {
    state: &'tcx mut T,
    session: &'tcx Session,
    summaries: &'analysis [Summary<'tcx>],
}

impl<'tcx, 'analysis, T: TaintDomain<Local>> Visitor<'tcx>
    for TransferFunction<'tcx, 'analysis, T>
{
    fn visit_statement(&mut self, statement: &Statement<'tcx>, _: Location) {
        let Statement { source_info, kind } = statement;

        self.visit_source_info(source_info);

        // TODO(Hilmar): Match more statement kinds
        #[allow(clippy::single_match)]
        match kind {
            StatementKind::Assign(box (ref place, ref rvalue)) => {
                self.t_visit_assign(place, rvalue);
            }
            _ => (),
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, _: Location) {
        let Terminator { source_info, kind } = terminator;

        self.visit_source_info(source_info);

        match kind {
            TerminatorKind::Goto { .. } => {}
            TerminatorKind::SwitchInt { .. } => {}
            TerminatorKind::Return => {}
            TerminatorKind::Call {
                func,
                args,
                destination,
                fn_span,
                ..
            } => self.t_visit_call(func, args, destination, fn_span),
            TerminatorKind::Assert { .. } => {}
            _ => {}
        }
    }
}

impl<'tcx, 'analysis, T> TransferFunction<'tcx, 'analysis, T>
where
    Self: Visitor<'tcx>,
    T: TaintDomain<Local>,
{
    fn t_visit_assign(&mut self, place: &Place, rvalue: &Rvalue) {
        match rvalue {
            // If we assign a constant to a place, the place is clean.
            Rvalue::Use(Operand::Constant(_)) => {
                self.state.mark_tainted(place.local);
            }

            // Otherwise we propagate the taint
            Rvalue::Use(Operand::Copy(f) | Operand::Move(f)) => {
                self.state.propagate(f.local, place.local);
            }

            Rvalue::BinaryOp(_, ref b) => {
                let (ref o1, ref o2) = **b;
                match (o1, o2) {
                    (Operand::Constant(_), Operand::Constant(_)) => {
                        self.state.mark_untainted(place.local);
                    }
                    (Operand::Copy(a) | Operand::Move(a), Operand::Copy(b) | Operand::Move(b)) => {
                        if self.state.is_tainted(a.local) || self.state.is_tainted(b.local) {
                            self.state.mark_tainted(place.local);
                        } else {
                            self.state.mark_untainted(place.local);
                        }
                    }
                    (Operand::Copy(p) | Operand::Move(p), Operand::Constant(_))
                    | (Operand::Constant(_), Operand::Copy(p) | Operand::Move(p)) => {
                        if self.state.is_tainted(p.local) {
                            self.state.mark_tainted(place.local);
                        } else {
                            self.state.mark_untainted(place.local);
                        }
                    }
                }
            }
            Rvalue::UnaryOp(_, Operand::Move(p) | Operand::Copy(p)) => {
                self.state.propagate(p.local, place.local);
            }
            _ => {}
        }
    }

    fn t_visit_call(
        &mut self,
        func: &Operand,
        args: &[Operand],
        destination: &Option<(Place, BasicBlock)>,
        span: &Span,
    ) {
        let name = func
            .constant()
            .expect("Operand is not a function")
            .to_string();

        if let Some(summary) = self.summaries.iter().find(|x| name == x.name) {
            let Summary { taint_type, .. } = summary;
            match taint_type {
                TaintType::Marked(Mark::Sink) | TaintType::Inferred(Infer::Sink) => {
                    self.t_visit_sink(name, args, span);
                }
                TaintType::Marked(Mark::Source) | TaintType::Inferred(Infer::Source) => {
                    self.t_visit_source_destination(destination);
                }
                _ => {}
            }
        }
    }

    fn t_visit_source_destination(&mut self, destination: &Option<(Place, BasicBlock)>) {
        if let Some((place, _)) = destination {
            self.state.mark_tainted(place.local);
        }
    }

    fn t_visit_sink(&mut self, name: String, args: &[Operand], span: &Span) {
        if args
            .iter()
            .map(|op| op.place().unwrap().local)
            .any(|el| self.state.is_tainted(el))
        {
            self.session.emit_err(super::errors::TaintedSink {
                fn_name: name,
                span: *span,
            });
        }
    }
}
