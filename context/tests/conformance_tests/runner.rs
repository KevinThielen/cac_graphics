use crate::Context;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, Mutex},
    //time::Instant,
};

pub const TEST_OK: &str = concat!("\x1b[92m", "ok", "\x1b[0m");
pub const TEST_FAIL: &str = concat!("\x1b[31m", "FAILED", "\x1b[0m");

pub struct Entry {
    pub name: &'static str,
    pub reason: String,
}

pub struct TestReport {
    pub errors: Vec<Entry>,
    pub context: &'static str,
}

impl TestReport {
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            context: "",
        }
    }
    pub const fn with_context(context: &'static str) -> Self {
        Self {
            errors: Vec::new(),
            context,
        }
    }

    pub fn with_entry(name: &'static str, reason: String) -> Self {
        Self {
            errors: vec![Entry { name, reason }],
            context: name,
        }
    }

    pub fn failed(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print_errors(&self) {
        if !self.errors.is_empty() {
            log::error!("failure:");
            for e in &self.errors {
                log::error!("\n---- {} stdout ----\n{}", e.name, e.reason);
            }
        }
    }
}

#[macro_export]
macro_rules! TEST {
    ($f:ident) => {{
        $crate::runner::TestCase {
            name: concat!(module_path!(), "::", stringify!($f)),
            func: |ctx: &mut $crate::Context| -> _ {
                match ctx {
                    #[cfg(not(target_arch = "wasm32"))]
                    $crate::Context::OpenGLGLFW(ctx) => $f(ctx),
                    #[cfg(target_arch = "wasm32")]
                    $crate::Context::WebGL(ctx) => $f(ctx),
                }
            },
        }
    }};
}

#[macro_export]
macro_rules! check {
    ($cond:expr) => {
        if !$cond {
            anyhow::bail!("{} = {}, {}:{}", stringify!($cond), $cond, file!(), line!())
        }
    };
}

#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        anyhow::bail!("{}, {}:{}", $msg, file!(), line!())
    };
}

pub struct TestCase {
    pub name: &'static str,
    pub func: fn(&mut Context) -> anyhow::Result<(), anyhow::Error>,
}

fn get_panic_message(payload: &(dyn std::any::Any + Send), location: &Option<String>) -> String {
    let location = location.as_ref().map_or("", |loc| loc);

    payload.downcast_ref::<&str>().map_or_else(
        || {
            payload.downcast_ref::<String>().map_or_else(
                || format!("Unknown panic, {location}"),
                |s| format!("{s}, {location}"),
            )
        },
        |s| format!("{s}, {location}"),
    )
}

pub fn run_tests(
    panic_loc: &Arc<Mutex<Option<String>>>,
    prefix: &'static str,
    ctx: &mut Context,
    tests: &[crate::runner::TestCase],
) -> TestReport {
    let mut report = TestReport::with_context(prefix);
    log::info!("\nrunning {} tests", tests.len());
    //let timer = Instant::now();
    for test in tests.iter() {
        //always reset the context to prevent state leaking through
        ctx.reset();
        print!("test {} ... ", test.name);
        let result = catch_unwind(AssertUnwindSafe(|| (test.func)(ctx)));

        let error_entry = match result {
            Ok(result) => match result {
                Ok(()) => ctx.poll_errors().map(|e| Entry {
                    name: test.name,
                    reason: e.concat(),
                }),
                Err(e) => Some(Entry {
                    name: test.name,
                    reason: e.to_string(),
                }),
            },
            Err(e) => {
                let p = panic_loc.lock().unwrap().take();
                let msg = get_panic_message(e.as_ref(), &p);

                Some(Entry {
                    name: test.name,
                    reason: msg,
                })
            }
        };

        if let Some(e) = error_entry {
            report.errors.push(e);
            log::error!("test {} ... {TEST_FAIL}", test.name);
        } else {
            log::info!("test {} ... {TEST_OK}", test.name);
        }
    }

    let failed = report.errors.len();
    let test_result = if failed == 0 { TEST_OK } else { TEST_FAIL };
    let passed = tests.len() - failed;
    //let time = timer.elapsed().as_secs_f32();
    let time = 0.0;

    log::info!("\ntest result: {test_result}. {passed} passed; {failed} failed; 0 ignored; 0 measured; 0 filtered out; finished in {time:.2}s\n");

    report
}
