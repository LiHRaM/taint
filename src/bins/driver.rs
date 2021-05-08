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
use std::convert::TryFrom;

fn main() {
    // We use the default hook for internal compiler errors (ICE).
    rustc_driver::install_ice_hook();
    // This method call is used to set up rust logging.
    rustc_driver::init_rustc_env_logger();

    // Gets the command line arguments sent to the program and adds them to a vector.
    let mut rustc_args: Vec<String> = vec![];

    for arg in std::env::args() {
        rustc_args.push(arg);
    }

    // We use our function to run the compiler with the supplied arguements and the callbacks
    // which we would like the rust compiler to make during the compilation process.
    run_compiler(rustc_args, &mut TaintCompilerCallbacks)
}

// Our wrapper function for running the rust compiler
fn run_compiler(mut args: Vec<String>, callbacks: &mut (dyn rustc_driver::Callbacks + Send)) -> ! {
    // We first make sure that the sysroot argument to our executable is set.
    // The sysroot should be pointing to the toolchain named in the "rust-toolchain" file.
    if let Some(sysroot) = compile_time_sysroot() {
        let sysroot_flag = "--sysroot";
        if !args.iter().any(|e| e == sysroot_flag) {
            args.push(sysroot_flag.to_owned());
            args.push(sysroot);
        }
    }

    // We run the rust compiler using our arguments and callbacks, and get record the exit code.
    // We create a closure to run the compiler, specify that it should take ownership of captured variables, and then execute the closure.
    let exit_code = rustc_driver::catch_with_exit_code(move || {
        rustc_driver::RunCompiler::new(&args, callbacks).run()
    });

    // We close the process using the exit code from the compiler.
    std::process::exit(exit_code)
}

// A function to get the sysroot of the toolchain used to compile the project.
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

// We create an empty struct on which we will implement the callbacks,
// that the compiler should run for our analysis to be conducted.
struct TaintCompilerCallbacks;

// We here implement the callback trait on our empty struct
impl rustc_driver::Callbacks for TaintCompilerCallbacks {
    // We want our analysis to run after the rust compiler has done its own analysis.
    // Other options are after_parsing and after_expansion.
    fn after_analysis<'tcx>(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> Compilation {
        // If the compiler has already found errors before reaching our analysis we stop the compilation.
        compiler.session().abort_if_errors();

        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            // We make sure that the program we are compiling has a main function.
            let (entry_def_id, _) = if let Some((entry_def, x)) = tcx.entry_fn(LOCAL_CRATE) {
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

            // This is where we call the eval_main function to run our own analysis starting at the main function.
            if let Some(return_code) = taint::eval::eval_main(
                tcx,
                entry_def_id.to_def_id(),
                taint::eval::TaintConfig::default(),
            ) {
                std::process::exit(
                    i32::try_from(return_code).expect("Return value was too large!"),
                );
            }
        });

        compiler.session().abort_if_errors();

        Compilation::Stop
    }
}
