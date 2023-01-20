#![cfg(test)]
//! Custom test runner for graphics tests
//!
//! Graphical applications have certain requirements that the default test runner in Rust doesn't
//! satisfy, such as:
//! - The window/graphics context needs to be on the main thread
//! - The Context can only be created once, due to some inherent static state
//! - Panicking is a nono on some platforms, or forced abort, so it should not be used for failing
//! tests. Better stick with Results
//!
//! How to run:
//! Just run cargo test
//!
//! Requirements:
//! - Desktop: None
//! - Web: TODO
//!
//! Writing tests:
//! - A test function accepts a &impl Context and returns a `TestResult`
//! - use Ok(()), check!(some condition), error!("some error message") to return the result

#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

#[macro_use]
mod runner;
mod platform;

mod buffer;
mod context;
mod render_target;

const CONTEXT_WIDTH: u32 = 800;
const CONTEXT_HEIGHT: u32 = 600;

type TestResult = anyhow::Result<(), anyhow::Error>;
use runner::TestCase;

use platform::{Context, TestSuite};

//TODO: Some macro to just annotate tests with, like the #[test] attribute
fn collect_tests() -> Vec<runner::TestCase> {
    let mut tests = Vec::with_capacity(100);

    tests.append(&mut context::tests());
    tests.append(&mut render_target::tests());
    tests.append(&mut buffer::tests());

    tests
}

fn collect_suits() -> Vec<TestSuite> {
    vec![TestSuite::GlfwWithOpenGL(4, 3), TestSuite::WebGL]
}

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        fn main() -> Result<(), &'static str> {
            console_log::init().expect("failed to init logger");
            shared_main()
        }
   } else {
        fn main() -> Result<(), &'static str> {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .format_level(false)
                .format_target(false)
                .format_timestamp(None)
                .init();

            shared_main()
        }
   }
}

fn shared_main() -> Result<(), &'static str> {
    let tests = collect_tests();
    let suits = collect_suits();

    let mut reports: Vec<runner::TestReport> = Vec::new();
    for s in &suits {
        let report = s.run(&tests);
        report.print_errors();
        reports.push(report);
    }

    if reports.iter().any(runner::TestReport::failed) {
        Err("one or more tests failed")
    } else {
        Ok(())
    }
}
