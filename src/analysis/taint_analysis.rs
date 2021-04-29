use crate::{Summary, TaintProperty};

use super::extensions::GenKillBitSetExt;
use rustc_index::bit_set::BitSet;
use rustc_middle::mir::{
    visit::Visitor, BasicBlock, Body, HasLocalDecls, Local, Location, Operand, Place, Rvalue,
    Statement, StatementKind, Terminator, TerminatorKind,
};

use rustc_mir::dataflow::{AnalysisDomain, Forward, GenKill, GenKillAnalysis};
use rustc_session::Session;
use rustc_span::Span;

/// A dataflow analysis that tracks whether a value may carry a taint.
///
/// Taints are introduced through sources, and consumed by sinks.
/// Ideally, a sink never consumes a tainted value - this should result in an error.
pub struct TaintAnalysis<'sess> {
    session: &'sess Session,
    summaries: Vec<Summary<'sess>>,
}

impl<'sess> TaintAnalysis<'sess> {
    pub fn new(session: &'sess Session, summaries: Vec<Summary<'sess>>) -> Self {
        TaintAnalysis { session, summaries }
    }
}

impl<'tcx> AnalysisDomain<'tcx> for TaintAnalysis<'tcx> {
    type Domain = BitSet<Local>;
    const NAME: &'static str = "MaybeTaintedLocals";

    type Direction = Forward;

    fn bottom_value(&self, body: &Body<'tcx>) -> Self::Domain {
        // bottom = untainted
        BitSet::new_empty(body.local_decls().len())
    }

    fn initialize_start_block(&self, _body: &Body<'tcx>, _state: &mut Self::Domain) {
        // Locals start out being untainted
    }
}

impl<'tcx> GenKillAnalysis<'tcx> for TaintAnalysis<'tcx> {
    type Idx = Local;

    fn statement_effect(
        &self,
        trans: &mut impl GenKill<Self::Idx>,
        statement: &Statement<'tcx>,
        location: Location,
    ) {
        self.transfer_function(trans)
            .visit_statement(statement, location);
    }

    fn terminator_effect(
        &self,
        trans: &mut impl GenKill<Self::Idx>,
        terminator: &Terminator<'tcx>,
        location: Location,
    ) {
        self.transfer_function(trans)
            .visit_terminator(terminator, location);
    }

    fn call_return_effect(
        &self,
        _trans: &mut impl GenKill<Self::Idx>,
        _block: BasicBlock,
        _func: &Operand<'tcx>,
        _args: &[Operand<'tcx>],
        _return_place: Place<'tcx>,
    ) {
        // do nothing
    }
}

impl<'a> TaintAnalysis<'a> {
    fn transfer_function<T>(&'a self, domain: &'a mut T) -> TransferFunction<'a, T> {
        TransferFunction {
            domain,
            session: self.session,
            summaries: self.summaries.clone(),
        }
    }
}

struct TransferFunction<'tcx, T> {
    domain: &'tcx mut T,
    session: &'tcx Session,
    summaries: Vec<Summary<'tcx>>,
}

impl<'tcx, T> Visitor<'tcx> for TransferFunction<'_, T>
where
    T: GenKill<Local>,
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

impl<'tcx, T> TransferFunction<'tcx, T>
where
    Self: Visitor<'tcx>,
    T: GenKill<Local>,
{
    fn t_visit_assign(&mut self, place: &Place, rvalue: &Rvalue) {
        match rvalue {
            // If we assign a constant to a place, the place is clean.
            Rvalue::Use(Operand::Constant(_)) => self.domain.kill(place.local),

            // Otherwise we propagate the taint
            Rvalue::Use(Operand::Copy(f) | Operand::Move(f)) => {
                self.domain.propagate(f.local, place.local);
            }

            Rvalue::BinaryOp(_, ref b) => {
                let (ref o1, ref o2) = **b;
                match (o1, o2) {
                    (Operand::Constant(_), Operand::Constant(_)) => self.domain.kill(place.local),
                    (Operand::Copy(a) | Operand::Move(a), Operand::Copy(b) | Operand::Move(b)) => {
                        if self.domain.is_tainted(a.local) || self.domain.is_tainted(b.local) {
                            self.domain.gen(place.local);
                        } else {
                            self.domain.kill(place.local);
                        }
                    }
                    (Operand::Copy(p) | Operand::Move(p), Operand::Constant(_))
                    | (Operand::Constant(_), Operand::Copy(p) | Operand::Move(p)) => {
                        if self.domain.is_tainted(p.local) {
                            self.domain.gen(place.local);
                        } else {
                            self.domain.kill(place.local);
                        }
                    }
                }
            }
            Rvalue::UnaryOp(_, Operand::Move(p) | Operand::Copy(p)) => {
                self.domain.propagate(p.local, place.local);
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

        if let Some((is_source, is_sink)) =
            if let Some(summary) = self.summaries.iter().find(|x| name == x.name) {
                let Summary { is_source: taints, is_sink: sink, .. } = summary;
                Some((taints.to_owned(), sink.to_owned()))
            } else {
                None
            }
        {
            // dbg!((&name, &is_source, &is_sink));
            match is_source {
                TaintProperty::Never => {}
                TaintProperty::Always => self.t_visit_source_destination(destination),
                TaintProperty::Sometimes(_) => {}
            }

            match is_sink {
                TaintProperty::Never => {}
                TaintProperty::Always => self.t_visit_sink(name, args, span),
                TaintProperty::Sometimes(_) => {}
            }
        }
    }

    fn t_visit_source_destination(&mut self, destination: &Option<(Place, BasicBlock)>) {
        if let Some((place, _)) = destination {
            self.domain.gen(place.local);
        }
    }

    fn t_visit_sink(&mut self, name: String, args: &[Operand], span: &Span) {
        if args
            .iter()
            .map(|op| op.place().unwrap().local)
            .any(|el| self.domain.is_tainted(el))
        {
            self.session.emit_err(super::errors::TaintedSink {
                fn_name: name,
                span: *span,
            });
        }
    }
}
