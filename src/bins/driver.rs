#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_ast;

use rustc_driver::Compilation;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

mod callback_impls;

fn main() {
    rustc_driver::install_ice_hook();
    rustc_driver::init_rustc_env_logger();
    init_tracing();

    let mut rustc_args: Vec<String> = vec![];

    for arg in std::env::args() {
        rustc_args.push(arg);
    }

    run_compiler(rustc_args, &mut TaintCompilerCallbacks)
}

/// We want our own tracing to debug the taint analysis.
/// Enable tracing via the `TAINT_LOG` environment variable.
///
/// Example: `TAINT_LOG=INFO cargo run -- tests/fails/simple.rs`
///
/// It is configured for minimal hassle.
/// It logs when functions marked with `#[instrument]` are entered,
/// and does not require any further code (such as the `event!` macro
/// provided by `tracing`).
fn init_tracing() {
    if let Ok(filter) = EnvFilter::try_from_env("TAINT_LOG") {
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::ENTER)
            .with_env_filter(filter)
            .without_time()
            .init();
    }
}

fn run_compiler(mut args: Vec<String>, callbacks: &mut (dyn rustc_driver::Callbacks + Send)) -> ! {
    if let Some(sysroot) = compile_time_sysroot() {
        let sysroot_flag = "--sysroot";
        if !args.iter().any(|e| e == sysroot_flag) {
            args.push(sysroot_flag.to_owned());
            args.push(sysroot);
        }
    }

    let exit_code = rustc_driver::catch_with_exit_code(move || {
        rustc_driver::RunCompiler::new(&args, callbacks).run()
    });
    std::process::exit(exit_code)
}

fn compile_time_sysroot() -> Option<String> {
    if option_env!("RUSTC_STAGE").is_some() {
        None
    } else {
        let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
        let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
        Some(match (home, toolchain) {
            (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
            _ => option_env!("RUST_SYSROOT")
                .expect("To build this without rustup, set the RUST_SYSROOT env var at build time")
                .to_owned(),
        })
    }
}

struct TaintCompilerCallbacks;

impl rustc_driver::Callbacks for TaintCompilerCallbacks {
    fn config(&mut self, _config: &mut rustc_interface::Config) {}

    /// We use procedural macro attributes to tag sources, sinks and sanitizers.
    /// Since they are removed during expansion, we can still find them after parsing.
    /// We use this stage to find them and create lists which we can use while analyzing the MIR.
    fn after_analysis<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> Compilation {
        compiler.session().abort_if_errors();
        enter_with_fn(queries, callback_impls::mir_analysis);
        compiler.session().abort_if_errors();
        Compilation::Stop
    }
}

fn enter_with_fn<'tcx, F: Fn(rustc_middle::ty::TyCtxt)>(
    queries: &'tcx rustc_interface::Queries<'tcx>,
    enter_fn: F,
) {
    queries.global_ctxt().unwrap().peek_mut().enter(enter_fn);
}
