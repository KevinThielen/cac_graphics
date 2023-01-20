#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]

pub mod color32;
pub mod gen_vec;
pub mod image;

pub use color32::Color32;

pub mod math {

    pub use glam::{
        ivec2, ivec3, ivec4, mat2, mat3, mat4, quat, uvec2, uvec3, uvec4, vec2, vec3, vec4, IVec2,
        IVec3, IVec4, Mat2, Mat3, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
    };

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
    pub enum Origin {
        BottomLeft,
        #[default]
        TopLeft,
    }

    pub type Rect = rect::Rect<f32>;
    pub type URect = rect::Rect<u32>;
    pub type IRect = rect::Rect<i32>;

    mod rect {
        use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
        pub struct Rect<T> {
            pub x: T,
            pub y: T,
            pub width: T,
            pub height: T,
        }

        impl<
                T: Copy + Sub<Output = T> + Add<Output = T> + AddAssign + SubAssign + Mul<Output = T>,
            > Rect<T>
        {
            pub const fn new(x: T, y: T, width: T, height: T) -> Self {
                Self {
                    x,
                    y,
                    width,
                    height,
                }
            }

            pub fn with_points(start: (T, T), end: (T, T)) -> Self {
                Self {
                    x: start.0,
                    y: start.1,
                    width: end.0 - start.0,
                    height: end.1 - start.1,
                }
            }
        }
    }
}
