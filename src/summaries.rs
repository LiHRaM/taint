//! Function summaries

/// How a function influences the flow of taint in a program.
#[derive(Clone, Debug)]
pub struct Summary<'tcx> {
    /// The function name.
    pub name: &'tcx str,
    /// Whether this function returns a tainted value.
    pub is_source: TaintProperty,
    /// Whether this function passes a value to a sink.
    pub is_sink: TaintProperty,
}

/// An element of a function signature.
///
/// For example, it could signify whether a function returns a taint.
#[derive(Clone, Debug, PartialEq)]
pub enum TaintProperty {
    /// This property never holds.
    Never,
    /// This property always holds.
    Always,
    /// This property is influenced function parameters,
    /// who are identified by index in the inner vector.
    Sometimes(Vec<usize>),
}
