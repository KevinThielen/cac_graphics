use std::sync::{Arc, Mutex};

use cac_context::Context as _;

use crate::runner::{self, run_tests};

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {

    } else {
        mod opengl;
    }
}

pub enum Context {
    #[cfg(not(target_arch = "wasm32"))]
    OpenGLGLFW(cac_context::opengl::Context<opengl::GLFWContext>),
    #[cfg(target_arch = "wasm32")]
    WebGL(WebGLContext),
}

impl Context {
    pub fn reset(&mut self) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::OpenGLGLFW(ctx) => ctx.reset(),
        }
    }

    pub fn poll_errors(&mut self) -> Option<Vec<String>> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::OpenGLGLFW(ctx) => ctx.poll_errors(),
        }
    }
}

pub enum TestSuite {
    GlfwWithOpenGL(u8, u8),
    WebGL,
}

impl TestSuite {
    pub fn run(&self, tests: &[runner::TestCase]) -> runner::TestReport {
        let old_hook = std::panic::take_hook();
        let panic_loc = Arc::new(Mutex::new(None));

        std::panic::set_hook({
            let msg = panic_loc.clone();

            Box::new(move |info| {
                if let Some(loc) = info.location() {
                    let mut msg = msg.lock().unwrap();
                    *msg = Some(format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));
                };
            })
        });

        log::info!("\n------ Context: {} ------", &self.name());
        let report = match *self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::GlfwWithOpenGL(major, minor) => match opengl::new_glfw((major, minor)) {
                Ok(mut ctx) => run_tests(&panic_loc, "glfw_with_opengl", &mut ctx, tests),
                Err(e) => runner::TestReport::with_entry("glfw_with_opengl", e.to_string()),
            },
            #[cfg(target_arch = "wasm32")]
            Self::WebGL => {
                let mut ctx = Context::WebGL(WebGLContext {});
                run_tests(&panic_loc, "webgl", &mut ctx, tests)
            }
            _ => {
                log::info!("Context not supported on platform, skipping");
                runner::TestReport::new()
            }
        };

        std::panic::set_hook(old_hook);
        report
    }

    fn name(&self) -> String {
        match self {
            Self::GlfwWithOpenGL(major, minor) => format!("GLFW OpenGL{major}.{minor}"),
            Self::WebGL => "WebGL".to_string(),
        }
    }
}
