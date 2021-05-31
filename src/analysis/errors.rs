use rustc_macros::SessionDiagnostic;
use rustc_span::Span;

#[derive(SessionDiagnostic)]
#[error = "T0001"]
pub struct TaintedSink {
    pub fn_name: String,
    #[message = "function `{fn_name}` received tainted input"]
    #[label = "sink function"]
    pub span: Span,
}

#[derive(SessionDiagnostic)]
#[error = "T0002"]
pub struct InvalidVariant {
    pub attr_name: String,
    #[message = "Taint attribute `{attr_name}` is invalid. We currently only support `source`, `sink`, and `sanitizer`"]
    #[label = "invalid taint attribute"]
    pub span: Span,
}
