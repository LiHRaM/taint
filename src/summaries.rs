//! Function summaries

/// How a function influences the flow of taint in a program.
#[derive(Clone, Debug)]
pub struct Summary<'tcx> {
    /// The function name.
    pub name: &'tcx str,
    pub taint_type: TaintType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaintType {
    Marked(Mark),
    Inferred(Infer),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mark {
    Source,
    Sink,
    Sanitizer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Infer {
    Source,
    Sink,
    Sanitizer,
}
