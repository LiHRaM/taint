#![feature(test)]

use std::{env, path::PathBuf};

use colored::*;
use compiletest_rs as compiletest;
use compiletest_rs::common::Mode as TestMode;

#[test]
fn test_runner() {
    env::set_var("TAINT_ENV_VAR_TEST", "0");
    env::set_var("TAINT_TEMP", env::temp_dir());
    env::set_var("RUST_BACKTRACE", "1");
    
    let target = get_target();

    run_compile_pass("tests/passes", &target);
    run_compile_fail("tests/fails", &target);
}

fn get_target() -> String {
    env::var("TAINT_TEST_TARGET").unwrap_or_else(|_| get_host())
}

fn get_host() -> String {
    let command = std::process::Command::new(taint_path());
    let version_meta = rustc_version::VersionMeta::for_command(command)
        .expect("failed to parse rustc version info");
    version_meta.host
}

/// Get the path of the taint executable.
/// You can set a custom location by setting the `TAINT` environment variable.
/// Otherwise, we will get the information from the `CARGO_BIN_EXE_<bin>` name,
/// as documented in: https://doc.rust-lang.org/cargo/reference/environment-variables.html.
fn taint_path() -> PathBuf {
    PathBuf::from(option_env!("TAINT").unwrap_or(env!("CARGO_BIN_EXE_taint")))
}

fn run_compile_pass(path: &str, target: &str) {
    eprintln!(
        "{}",
        format!(
            "## Running passes tests in {} against taint for target {}",
            path, target
        )
        .green()
        .bold()
    );

    run_tests(TestMode::Ui, path, target);
}

fn run_compile_fail(path: &str, target: &str) {
    eprintln!(
        "{}",
        format!(
            "## Running fails tests in {} against taint for target {}",
            path, target
        )
        .green()
        .bold()
    );

    run_tests(TestMode::CompileFail, path, target);
}

fn run_tests(mode: TestMode, path: &str, target: &str) {
    let in_rustc_test_suite = option_env!("RUSTC_STAGE").is_some();
    let flags = get_flags(in_rustc_test_suite);
    let config = get_config(mode, path, target, flags);

    compiletest::run_tests(&config);
}

fn get_flags(in_rustc_test_suite: bool) -> String {
    let mut flags = vec!["--edition 2018".into()];
    if in_rustc_test_suite {
        flags.push("-Astable-features".into());
    } else {
        flags.push("-Dwarnings -Dunused".to_owned());
    }
    if let Ok(sysroot) = env::var("TAINT_SYSROOT") {
        flags.push(format!("--sysroot {}", sysroot));
    }
    if let Ok(extra_flags) = env::var("TAINT_FLAGS") {
        flags.push(extra_flags);
    }
    let flags = flags.join(" ");
    eprintln!("    Compiler flags: {}", flags);
    flags
}

fn get_config(
    mode: TestMode,
    path: &str,
    target: &str,
    flags: String,
) -> compiletest::common::ConfigWithTemp {
    let mut config = compiletest::Config::default().tempdir();
    config.mode = mode;
    config.src_base = PathBuf::from(path);
    config.rustc_path = taint_path();
    config.filters = env::args().nth(1).into_iter().collect();
    config.host = get_host();
    config.target = target.to_owned();
    config.target_rustcflags = Some(flags);
    config
}
