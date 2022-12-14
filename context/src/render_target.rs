use core::Color32;

pub trait Context {
    fn clear(&mut self, target: &RenderTarget);
}

pub struct RenderTarget {
    pub clear_color: Option<Color32>,
}

impl RenderTarget {
    pub fn new() -> Self {
        Self { clear_color: None }
    }

    pub fn with_clear_color(clear_color: Color32) -> Self {
        Self {
            clear_color: Some(clear_color),
        }
    }

    pub fn clear<C: Context>(&self, ctx: &mut C) {
        ctx.clear(self);
    }
}
