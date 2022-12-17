use super::{gl, Context, GLContext};
use crate::render_target;

impl<C: GLContext> render_target::Context for Context<C> {
    fn clear(&mut self, target: &render_target::RenderTarget) {
        if let Some(color) = target.clear_color {
            let (r, g, b, a) = color.as_rgba();
            unsafe {
                gl::ClearColor(r, g, b, a);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
        }
    }
}
