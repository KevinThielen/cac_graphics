use cac_context::{opengl, Context};
use glfw::Context as _;

pub struct GLFWContext(glfw::Window);

impl opengl::GLContext for GLFWContext {
    fn swap_buffers(&mut self) {
        self.0.swap_buffers();
    }

    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void {
        self.0.get_proc_address(name)
    }
}

pub fn new_glfw(version: (u8, u8)) -> anyhow::Result<super::Context, anyhow::Error> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    // Create a windowed mode window and its OpenGL context
    // window hints have to be set before the window is created
    glfw.window_hint(glfw::WindowHint::ContextVersion(
        version.0.into(),
        version.1.into(),
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    // Create a windowed mode window and its OpenGL context
    let (mut window, _events) = glfw
        .create_window(
            crate::CONTEXT_WIDTH,
            crate::CONTEXT_HEIGHT,
            "GLFW TestSuite",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();

    let mut ctx = opengl::Context::new(GLFWContext(window))?;
    ctx.update();

    glfw.poll_events();

    let ctx = super::Context::OpenGLGLFW(ctx);

    Ok(ctx)
}
