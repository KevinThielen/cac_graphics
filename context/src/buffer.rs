use std::fmt::Display;

use crate::error::Error;

#[derive(Copy, Clone)]
pub struct Buffer {
    pub(crate) handle: usize,
}

#[derive(Copy, Clone)]
pub enum Kind {
    Vertex,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertex => writeln!(f, "VBO"),
        }
    }
}

#[derive(Default)]
pub enum Access {
    #[default]
    Once,
    Frequent,
    Always,
}

#[derive(Default)]
pub enum Usage {
    #[default]
    Write,
    Read,
    Copy,
}

impl Buffer {
    pub fn new<C: Context>(ctx: &mut C, kind: Kind, access: Access, usage: Usage) -> Self {
        ctx.create(kind, access, usage)
    }

    pub fn new_vertex<C: Context>(ctx: &mut C) -> Self {
        ctx.create(Kind::Vertex, Access::default(), Usage::default())
    }

    pub fn with_vertex_data<T: FlatData, C: Context>(
        ctx: &mut C,
        data: &[T],
    ) -> Result<Self, Error> {
        let buffer = Self::new_vertex(ctx);
        buffer.set_data(ctx, data)?;
        Ok(buffer)
    }

    pub fn get_mut<'a, C: Context>(self, ctx: &'a mut C) -> Option<&'a mut C::NativeBuffer> {
        ctx.get_mut(self)
    }

    pub fn get<'a, C: Context>(self, ctx: &'a mut C) -> Option<&'a C::NativeBuffer> {
        ctx.get(self)
    }

    pub fn set_data<T: FlatData, C: Context>(self, ctx: &mut C, data: &[T]) -> Result<(), Error> {
        ctx.get_mut(self).map_or_else(
            || Err(Error::ResourceNotFound),
            |buffer| buffer.set_data(data),
        )
    }
}

pub trait Context {
    type NativeBuffer: NativeContext;

    fn create(&mut self, kind: Kind, freq: Access, usage: Usage) -> Buffer;
    fn get_mut(&mut self, handle: Buffer) -> Option<&mut Self::NativeBuffer>;
    fn get(&self, handle: Buffer) -> Option<&Self::NativeBuffer>;
}

pub trait NativeContext {
    fn set_data<T: FlatData>(&mut self, data: &[T]) -> Result<(), Error>;
}

pub unsafe trait FlatData {}

unsafe impl FlatData for f32 {}
unsafe impl FlatData for f64 {}
unsafe impl FlatData for u8 {}
unsafe impl FlatData for u16 {}
unsafe impl FlatData for u32 {}
unsafe impl FlatData for u64 {}
unsafe impl FlatData for i8 {}
unsafe impl FlatData for i16 {}
unsafe impl FlatData for i32 {}
