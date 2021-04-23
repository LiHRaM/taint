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
