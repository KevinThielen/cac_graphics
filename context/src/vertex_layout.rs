use crate::error::Error;

pub trait Native {}

#[derive(Clone, Default)]
pub struct BufferAttributes {
    pub attributes: Vec<VertexAttribute>,
    pub buffer: Option<crate::BufferHandle>,
    pub stride: Stride,
    pub offset: usize,
}

impl BufferAttributes {
    #[must_use]
    pub fn stride(&self) -> usize {
        match self.stride {
            Stride::Bytes(bytes) => bytes,
            Stride::Interleaved => self
                .attributes
                .iter()
                .map(|a| a.kind.size() * a.components.count() as usize)
                .sum(),
        }
    }
}

#[derive(Clone)]
pub struct VertexLayout {
    pub attributes: Vec<BufferAttributes>,
}

impl VertexLayout {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn push_attributes<const N: usize>(&mut self, attributes: [VertexAttribute; N]) {
        self.attributes.push(BufferAttributes {
            attributes: attributes.into(),
            ..BufferAttributes::default()
        });
    }

    /// Sets a buffer for a specific attribute set, refered to via the index of that set.
    ///
    /// # Errors
    /// `Error::ResourceNotFound`: When the index doesn't refer to any attribute set. In that case,
    /// you should check the index or create/push the attributes first.
    pub fn set_buffer(
        &mut self,
        index: usize,
        buffer: crate::BufferHandle,
        stride: Stride,
        offset: usize,
    ) -> Result<(), Error> {
        if let Some(attributes) = self.attributes.get_mut(index) {
            attributes.buffer = Some(buffer);
            attributes.stride = stride;
            attributes.offset = offset;
            Ok(())
        } else {
            Err(Error::ResourceNotFound)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
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

#[derive(Copy, Default, Clone, PartialEq, Eq)]
pub enum Stride {
    #[default]
    Interleaved,
    Bytes(usize),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Components {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
}

impl Components {
    #[must_use]
    pub const fn count(&self) -> u8 {
        match self {
            Self::Scalar => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AttributeKind {
    F32,
    U8,
}

impl AttributeKind {
    #[must_use]
    pub const fn size(&self) -> usize {
        match self {
            Self::F32 => std::mem::size_of::<f32>(),
            Self::U8 => std::mem::size_of::<u8>(),
        }
    }
}
