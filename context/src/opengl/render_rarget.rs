use core::Color32;

use super::gl;
use crate::render_target::RenderTarget;
use crate::Error;

use crate::render_target::Native as _;

pub struct Native {
    view_port: (u32, u32, u32, u32),
    clear_color: Option<Color32>,
}

impl Native {
    pub(super) fn new(render_target: RenderTarget) -> Self {
        let mut rt = Self {
            view_port: render_target.view_port,
            clear_color: render_target.clear_color,
        };

        rt.set_clear_color(render_target.clear_color);

        rt
    }

    pub(super) fn bind(&mut self) -> Result<(), Error> {
        let (x, y, w, h) = self.view_port;
        let x = x
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport x conversion wraps i32"))?;
        let y = y
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport y conversion wraps i32"))?;
        let w = w
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport width conversion wraps i32"))?;
        let h = h
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport height conversion wraps i32"))?;

        unsafe {
            //gl::Enable(gl::SCISSOR_TEST);
            //gl::Scissor(x as i32, y as i32, w as i32, h as i32);
            gl::Viewport(x, y, w, h);
        }

        Ok(())
    }
}

impl crate::render_target::Native for Native {
    fn clear(&mut self) {
        let mut flags = 0;
        if let Some(color) = self.clear_color {
            flags |= gl::COLOR_BUFFER_BIT;
            let (r, g, b, a) = color.as_rgba();
            unsafe {
                gl::ClearColor(r, g, b, a);
            }
        }

        unsafe {
            gl::Clear(flags);
        }
    }

    fn set_clear_color(&mut self, color: Option<core::Color32>) {
        self.clear_color = color;
    }
}
