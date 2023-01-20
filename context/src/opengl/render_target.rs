use cac_core::math::URect;
use cac_core::{image, Color32};

use super::gl;
use crate::render_target::RenderTarget;
use crate::Error;

use crate::render_target::Native as _;

pub struct Native {
    viewport: URect,
    clear_color: Option<Color32>,
}

impl Native {
    pub(super) fn new(render_target: RenderTarget) -> Self {
        let mut rt = Self {
            viewport: render_target.viewport,
            clear_color: render_target.clear_color,
        };

        rt.set_clear_color(render_target.clear_color);

        rt
    }

    pub(super) fn bind(&mut self) -> Result<(), Error> {
        let x = self
            .viewport
            .x
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport x conversion wraps i32"))?;
        let y = self
            .viewport
            .y
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport y conversion wraps i32"))?;
        let w = self
            .viewport
            .width
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport width conversion wraps i32"))?;
        let h = self
            .viewport
            .height
            .try_into()
            .map_err(|_| Error::ConversionFailed("viewport height conversion wraps i32"))?;

        unsafe {
            gl::Viewport(x, y, w, h);
            gl::Scissor(x, y, w, h);
        }

        Ok(())
    }
}

impl crate::render_target::Native for Native {
    fn clear(&mut self) {
        let mut flags = 0;
        if let Some(color) = self.clear_color {
            flags |= gl::COLOR_BUFFER_BIT;
            let [r, g, b, a] = color.as_rgba();
            unsafe {
                gl::ClearColor(r, g, b, a);
            }
        }

        unsafe {
            gl::Clear(flags);
        }
    }

    fn set_clear_color(&mut self, color: Option<Color32>) {
        self.clear_color = color;
    }

    fn read_pixels(&self, format: image::Format, rect: URect) -> Result<image::Image, Error> {
        let count = (rect.width * rect.height)
            .try_into()
            .map_err(|_| Error::ConversionFailed("rect dimensions to usize"))?;

        let mut data = format.create_storage(count);

        let (gl_format, kind) = match format {
            image::Format::RgbU8 => (gl::RGB, gl::UNSIGNED_BYTE),
            image::Format::RgbF32 => (gl::RGB, gl::FLOAT),
            image::Format::RgbaU8 => (gl::RGBA, gl::UNSIGNED_BYTE),
            image::Format::RgbaF32 => (gl::RGBA, gl::FLOAT),
        };

        let x = rect
            .x
            .try_into()
            .map_err(|_| Error::ConversionFailed("rect.x to GLint"))?;

        let y = rect
            .y
            .try_into()
            .map_err(|_| Error::ConversionFailed("rect.y to GLint"))?;

        let width = rect
            .width
            .try_into()
            .map_err(|_| Error::ConversionFailed("rect.width to GLsizei"))?;

        let height = rect
            .height
            .try_into()
            .map_err(|_| Error::ConversionFailed("rect.height to GLsizei"))?;

        unsafe { gl::ReadPixels(x, y, width, height, gl_format, kind, data.as_mut_ptr()) };

        image::Image::new(rect.width, rect.height, format, data)
            .map_err(|e| Error::ExternalError(e.to_string()))
    }

    fn set_viewport(&mut self, viewport: URect) {
        self.viewport = viewport;
        //rebind, since it changed
        self.bind().unwrap();
    }
}
