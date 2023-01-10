use crate::Error;
use std::fmt::Display;

/// Trait to mark data that is safe to pass to the buffers.
///
/// # Safety
/// The data inside the types that implement this trait MUST follow the C repr.
/// This can't be enforced by the compiler, so the programmer is responsible to
/// mark their structs with repr(C)
pub unsafe trait FlatData {}

#[derive(Copy, Clone)]
pub enum Kind {
    Vertex,
}

#[derive(Default, Clone, Copy)]
pub enum Access {
    #[default]
    Once,
    Frequent,
    Always,
}

#[derive(Default, Clone, Copy)]
pub enum Usage {
    #[default]
    Write,
    Read,
    Copy,
}

#[derive(Copy, Clone)]
pub struct Buffer<'a, T: FlatData> {
    pub kind: Kind,
    pub access: Access,
    pub usage: Usage,
    pub data: Option<&'a [T]>,
}

impl<'a, T: FlatData> Buffer<'a, T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            kind: Kind::Vertex,
            access: Access::Once,
            usage: Usage::Write,
            data: None,
        }
    }

    #[must_use]
    pub const fn with_vertex_data(access: Access, usage: Usage, data: &'a [T]) -> Self {
        Self {
            data: Some(data),
            kind: Kind::Vertex,
            access,
            usage,
        }
    }
}

pub trait Native {
    /// Sets the data of the buffer
    ///
    /// # Errors
    /// Depends on the native implementation.
    ///
    /// `Error::ConversionError`: When the length of the containing data can't be converted into
    /// the native type without wrapping or overflowing.
    fn set_data<T: FlatData>(&mut self, data: &[T]) -> Result<(), Error>;
}

unsafe impl FlatData for f32 {}
unsafe impl FlatData for f64 {}
unsafe impl FlatData for u8 {}
unsafe impl FlatData for u16 {}
unsafe impl FlatData for u32 {}
unsafe impl FlatData for u64 {}
unsafe impl FlatData for i8 {}
unsafe impl FlatData for i16 {}
unsafe impl FlatData for i32 {}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertex => write!(f, "vertex"),
        }
    }
}
