use crate::gl43_core as gl;

pub trait GLContext {
    fn swap_buffers(&mut self);
    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void;
}

pub struct Context<C: GLContext> {
    gl_context: C,
}

impl<C: GLContext> Context<C> {
    pub fn new(mut context: C) -> Self {
        gl::load_with(|name| context.get_proc_address(name));
        Self {
            gl_context: context,
        }
    }

    pub fn update(&mut self) {
        self.gl_context.swap_buffers();
    }
}
