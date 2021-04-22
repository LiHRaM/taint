#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use rustc_driver::Compilation;
use rustc_errors::{emitter::HumanReadableErrorType, ColorConfig};
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_session::config::ErrorOutputType;

fn main() {
    rustc_driver::install_ice_hook();
    rustc_driver::init_rustc_env_logger();

    let mut rustc_args: Vec<String> = vec![];

    for arg in std::env::args() {
        rustc_args.push(arg);
    }

    run_compiler(rustc_args, &mut TaintCompilerCallbacks)
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
    fn after_analysis<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> Compilation {
        compiler.session().abort_if_errors();

        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let (_entry_def_id, _) = if let Some((entry_def, x)) = tcx.entry_fn(LOCAL_CRATE) {
                (entry_def, x)
            } else {
                let output_ty = ErrorOutputType::HumanReadable(HumanReadableErrorType::Default(
                    ColorConfig::Auto,
                ));
                rustc_session::early_error(
                    output_ty,
                    "taint can only analyze programs that have a main function",
                );
            };

            // if let Some(return_code) = taint::eval_main(tcx, entry_def_id.to_def_id()) {
            //     std::process::exit(
            //         i32::try_from(return_code).expect("Return value was too large!"),
            //     );
            // }
        });

        compiler.session().abort_if_errors();

        Compilation::Stop
    }
}
