#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(C)]
pub struct Color32 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color32 {
    /// Almost black with a touch of green
    pub const DARK_JUNGLE_GREEN: Self = Self::from_rgb(0.102, 0.141, 0.129);
    /// Grape like purple
    pub const PERSIAN_INDIGO: Self = Self::from_rgb(0.20, 0.0, 0.30);
    /// Dirty White
    pub const GAINSBORO: Self = Self::from_rgb(0.79, 0.92, 0.87);
    /// It's really nice to look at
    pub const UNITY_YELLOW: Self = Self::from_rgb(1.0, 0.92, 0.016);

    /// The Color Black
    pub const BLACK: Self = Self::from_rgb(0.0, 0.0, 0.0);
    /// The Color Red
    pub const RED: Self = Self::from_rgb(1.0, 0.0, 0.0);
    /// The Color Blue
    pub const BLUE: Self = Self::from_rgb(0.0, 0.0, 1.0);
    /// The Color Green
    pub const GREEN: Self = Self::from_rgb(0.0, 1.0, 0.0);
    /// The Color Yellow
    pub const YELLOW: Self = Self::from_rgb(1.0, 1.0, 0.0);
    /// The Color White
    pub const WHITE: Self = Self::from_rgb(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[must_use]
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub const fn as_rgb(&self) -> (f32, f32, f32) {
        (self.r, self.g, self.b)
    }

    #[must_use]
    pub const fn as_rgba(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}
