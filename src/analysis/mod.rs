use rustc_index::bit_set::BitSet;
use rustc_middle::mir::{
    visit::Visitor, BasicBlock, Body, HasLocalDecls, Local, Location, Operand, Place, Rvalue,
    Statement, StatementKind, Terminator,
};
use rustc_mir::dataflow::{AnalysisDomain, Forward, GenKill, GenKillAnalysis};
use rustc_session::Session;
use rustc_span::Span;

use extensions::GenKillBitSetExt;

mod errors;
mod extensions;

/// A dataflow analysis that tracks whether a value may carry a taint.
///
/// Taints are introduced through sources, and consumed by sinks.
/// Ideally, a sink never consumes a tainted value - this should result in an error.
pub struct MaybeTaintedLocals<'sess> {
    sess: &'sess Session,
}

impl<'sess> MaybeTaintedLocals<'sess> {
    pub fn new(sess: &'sess Session) -> Self {
        MaybeTaintedLocals { sess }
    }
}

impl<'tcx> AnalysisDomain<'tcx> for MaybeTaintedLocals<'tcx> {
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

impl<'tcx> GenKillAnalysis<'tcx> for MaybeTaintedLocals<'tcx> {
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

impl<'a> MaybeTaintedLocals<'a> {
    fn transfer_function<T>(&'a self, trans: &'a mut T) -> TransferFunction<'a, T> {
        TransferFunction {
            domain: trans,
            sess: self.sess,
        }
    }
}

struct TransferFunction<'a, T> {
    domain: &'a mut T,
    sess: &'a Session,
}

impl<'a, T> TransferFunction<'a, T>
where
    T: GenKill<Local>,
{
    fn handle_assignment(&mut self, assignment: &(Place, Rvalue)) {
        let (target, ref rval) = *assignment;
        match rval {
            // If we assign a constant to a place, the place is clean.
            Rvalue::Use(Operand::Constant(_)) => self.domain.kill(target.local),

            // Otherwise we propagate the taint
            Rvalue::Use(Operand::Copy(f) | Operand::Move(f)) => {
                self.domain.propagate(f.local, target.local);
            }

            Rvalue::BinaryOp(_, ref b) => {
                let (ref o1, ref o2) = **b;
                match (o1, o2) {
                    (Operand::Constant(_), Operand::Constant(_)) => self.domain.kill(target.local),
                    (Operand::Copy(a) | Operand::Move(a), Operand::Copy(b) | Operand::Move(b)) => {
                        if self.domain.is_tainted(a.local) || self.domain.is_tainted(b.local) {
                            self.domain.gen(target.local);
                        } else {
                            self.domain.kill(target.local);
                        }
                    }
                    (Operand::Copy(p) | Operand::Move(p), Operand::Constant(_))
                    | (Operand::Constant(_), Operand::Copy(p) | Operand::Move(p)) => {
                        if self.domain.is_tainted(p.local) {
                            self.domain.gen(target.local);
                        } else {
                            self.domain.kill(target.local);
                        }
                    }
                }
            }
            Rvalue::UnaryOp(_, Operand::Move(p) | Operand::Copy(p)) => {
                self.domain.propagate(p.local, target.local);
            }
            _ => {}
        }
    }

    fn handle_call(
        &mut self,
        func: &Operand,
        _args: &[Operand],
        destination: &Option<(Place, BasicBlock)>,
        span: &Span,
    ) {
        let name = func
            .constant()
            .expect("Operand is not a function")
            .to_string();

        // Sources taint their output
        if name.starts_with("input") {
            if let Some((place, _)) = destination {
                self.domain.gen(place.local);
            }
        }

        if name.starts_with("output")
            && _args
                .iter()
                .map(|op| op.place().unwrap().local)
                .any(|el| self.domain.is_tainted(el))
        {
            self.sess.emit_err(errors::TaintedSink {
                fn_name: name,
                span: *span,
            });
        }
    }
}

impl<'tcx, T> Visitor<'tcx> for TransferFunction<'_, T>
where
    T: GenKill<Local>,
{
    fn visit_statement(&mut self, statement: &Statement<'tcx>, _location: Location) {
        if let StatementKind::Assign(ref assignment) = statement.kind {
            self.handle_assignment(assignment);
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, _location: Location) {
        match &terminator.kind {
            rustc_middle::mir::TerminatorKind::Goto { target: _ } => {}
            rustc_middle::mir::TerminatorKind::SwitchInt {
                discr: _discr,
                switch_ty: _switch_ty,
                targets: _targets,
            } => {}
            rustc_middle::mir::TerminatorKind::Return => {}
            rustc_middle::mir::TerminatorKind::Call {
                func,
                args,
                destination,
                cleanup: _cleanup,
                from_hir_call: _from_hir_call,
                fn_span,
            } => self.handle_call(func, args, destination, fn_span),
            rustc_middle::mir::TerminatorKind::Assert {
                cond: _cond,
                expected: _expected,
                msg: _msg,
                target: _target,
                cleanup: _cleanup,
            } => {}
            _ => {}
        }
    }
}
