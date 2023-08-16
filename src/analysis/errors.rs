use rustc_macros::Diagnostic;
use rustc_span::Span;

#[derive(Diagnostic)]
#[diag(taint_func_received_tainted_input, code="T0001")]
pub(crate) struct TaintedSink {
    pub fn_name: String,
    #[label(taint_sink_function)]
    pub span: Span,
}

#[derive(Diagnostic)]
#[diag(taint_attribute_is_invalid, code="T0002")]
pub(crate) struct InvalidVariant {
    pub attr_name: String,
    #[label(taint_invalid_taint_attribute)]
    pub span: Span,
}
