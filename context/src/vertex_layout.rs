use crate::{buffer::Buffer, error::Error};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct VertexLayout {
    pub(crate) handle: usize,
}

impl VertexLayout {
    pub fn new<C: Context>(ctx: &mut C) -> Self {
        Context::create(ctx)
    }

    pub fn get_mut<C: Context>(self, ctx: &mut C) -> Option<&mut C::NativeLayout> {
        Context::get_mut(ctx, self)
    }

    pub fn get_mut_with_buffers<const N: usize, C: Context>(
        self,
        ctx: &mut C,
        buffers: [Buffer; N],
    ) -> (Option<&mut C::NativeLayout>, [Option<&C::NativeBuffer>; N]) {
        Context::get_mut_and_buffers(ctx, self, buffers)
    }

    pub fn with_attributes<C: Context>(
        ctx: &mut C,
        attributes: &[&[VertexAttribute]],
    ) -> Result<Self, Error> {
        let vao = Self::new(ctx);

        for attr in attributes.iter().enumerate() {
            ctx.set_attributes(vao, attr.0.try_into().unwrap(), attr.1)?;
        }

        Ok(vao)
    }

    pub fn set_attributes<C: Context>(
        self,
        ctx: &mut C,
        index: u8,
        attributes: &[VertexAttribute],
    ) {
        ctx.set_attributes(self, index, attributes);
    }

    pub fn set_buffer<C: Context>(
        self,
        ctx: &mut C,
        index: u8,
        buffer: Buffer,
        offset: usize,
        stride: Stride,
    ) -> Result<(), Error> {
        ctx.set_attribute_buffer(index, self, buffer, offset, stride)
    }
}

#[derive(Copy, Clone)]
pub struct VertexAttribute {
    pub location: u8,
    pub components: Components,
    pub kind: AttributeKind,
    pub normalized: bool,
    pub local_offset: usize,
}

impl VertexAttribute {
    #[must_use]
    pub const fn with_f32(location: u8, components: Components, local_offset: usize) -> Self {
        Self {
            location,
            components,
            kind: AttributeKind::F32,
            normalized: false,
            local_offset,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Stride<'a> {
    Interleaved(&'a [VertexAttribute]),
    Bytes(usize),
}

impl<'a> Stride<'a> {
    pub fn stride_bytes(&self) -> usize {
        match self {
            Stride::Bytes(bytes) => *bytes,
            Stride::Interleaved(attr) => attr
                .iter()
                .map(|attr| attr.kind.size() * attr.components.count() as usize)
                .sum(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Components {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
}

impl Components {
    pub fn count(&self) -> u8 {
        match self {
            Self::Scalar => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }
}

#[derive(Copy, Clone)]
pub enum AttributeKind {
    F32,
    U8,
}

impl AttributeKind {
    pub fn size(&self) -> usize {
        match self {
            Self::F32 => std::mem::size_of::<f32>(),
            Self::U8 => std::mem::size_of::<u8>(),
        }
    }
}

pub trait Context: crate::buffer::Context {
    type NativeLayout: NativeContext;

    fn get_mut_and_buffers<const N: usize>(
        &mut self,
        handle: VertexLayout,
        buffers: [Buffer; N],
    ) -> (
        Option<&mut Self::NativeLayout>,
        [Option<&Self::NativeBuffer>; N],
    );

    fn get_mut(&mut self, handle: VertexLayout) -> Option<&mut Self::NativeLayout>;

    fn create(&mut self) -> VertexLayout;

    fn set_attributes(
        &mut self,
        handle: VertexLayout,
        index: u8,
        attribute: &[VertexAttribute],
    ) -> Result<(), Error>;

    fn set_attribute_buffer(
        &mut self,
        location: u8,
        handle: VertexLayout,
        buffer: Buffer,
        offset: usize,
        stride: Stride,
    ) -> Result<(), Error>;
}

pub trait NativeContext {
    type NativeBuffer;
    fn set_attribute_buffer(&mut self, buffer: &Self::NativeBuffer) -> Result<(), Error>;
}
