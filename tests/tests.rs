// Copyright (C) 2016 Symtern Project Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-Apache
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified, or
// distributed except according to those terms.
extern crate compiletest_rs as compiletest;
extern crate symtern;

use std::path::PathBuf;

fn run_mode<P>(mode: &'static str, path: P)
    where P: Into<Option<&'static str>>
{
    let mut config = compiletest::Config::default();
    let cfg_mode = mode.parse().expect("Invalid mode");

    config.mode = cfg_mode;
    config.src_base = PathBuf::from(format!("tests/{}", path.into().unwrap_or(mode)));
    config.target_rustcflags = Some("-L target/debug -L target/debug/deps".to_string());

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail", None);
    run_mode("run-fail", None);
    run_mode("run-pass", "../examples");
}
