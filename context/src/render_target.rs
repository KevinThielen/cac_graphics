use cac_core::{image, math::URect, Color32};

use crate::Error;

#[derive(Copy, Clone)]
pub struct RenderTarget {
    pub clear_color: Option<Color32>,
    pub viewport: URect,
}

impl RenderTarget {
    #[must_use]
    pub const fn with_clear_color(viewport: URect, clear_color: Color32) -> Self {
        Self {
            clear_color: Some(clear_color),
            viewport,
        }
    }
}

pub trait Native {
    /// Creates an image from the pixels of the rendertarget.
    ///
    /// # Errors
    /// Depends on the native implementation
    fn read_pixels(&self, format: image::Format, viewport: URect) -> Result<image::Image, Error>;
    fn clear(&mut self);
    fn set_clear_color(&mut self, color: Option<Color32>);
    fn set_viewport(&mut self, viewport: URect);
}
