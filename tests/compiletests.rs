extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

#[cfg(debug_assertions)]
const DEPS: &'static str = "-L target/debug -L target/debug/deps";
#[cfg(not(debug_assertions))]
const DEPS: &'static str = "-L target/release -L target/release/deps";

fn run_mode(mode: &'static str) {
    let mut config = compiletest::default_config();
    config.mode = mode.parse().ok().expect("Invalid mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.target_rustcflags = Some(DEPS.to_owned());
    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail");
}
