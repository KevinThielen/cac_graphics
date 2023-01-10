use core::Color32;

#[derive(Copy, Clone)]
pub struct RenderTarget {
    pub clear_color: Option<Color32>,
    pub view_port: (u32, u32, u32, u32),
}

impl RenderTarget {
    #[must_use]
    pub const fn with_clear_color(view_port: (u32, u32, u32, u32), clear_color: Color32) -> Self {
        Self {
            clear_color: Some(clear_color),
            view_port,
        }
    }
}

pub trait Native {
    fn clear(&mut self);
    fn set_clear_color(&mut self, color: Option<Color32>);
}
