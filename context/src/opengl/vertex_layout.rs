use cac_core::gen_vec::GenVec;

use crate::{
    error::Error,
    handle,
    vertex_layout::{self, BufferAttributes},
};

use super::gl::{
    self,
    types::{GLenum, GLuint},
};

pub struct Native {
    pub(super) id: GLuint,
}

impl From<vertex_layout::AttributeKind> for GLenum {
    fn from(value: vertex_layout::AttributeKind) -> Self {
        use gl::{BYTE, FLOAT};

        match value {
            vertex_layout::AttributeKind::F32 => FLOAT,
            vertex_layout::AttributeKind::U8 => BYTE,
        }
    }
}

impl Native {
    pub(super) fn new(
        layout: &crate::VertexLayout,
        buffers: &GenVec<handle::Buffer, super::buffer::Native>,
    ) -> Result<Self, Error> {
        let mut vao = Self {
            id: unsafe {
                let mut vao = 0;
                gl::GenVertexArrays(1, &mut vao);
                vao
            },
        };

        vao.bind();
        Self::set_attributes(&layout.attributes)?;
        Self::set_buffers(&layout.attributes, buffers)?;

        Ok(vao)
    }

    pub(super) fn bind(&mut self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn set_attributes(attributes: &[BufferAttributes]) -> Result<(), Error> {
        attributes
            .iter()
            .enumerate()
            .try_for_each(|(index, attributes)| {
                for attr in &attributes.attributes {
                    let local_offset = attr
                        .local_offset
                        .try_into()
                        .map_err(|_| Error::ConversionFailed("local attribute offset to GLuint"))?;

                    let index = index
                        .try_into()
                        .map_err(|_| Error::ConversionFailed("attribute index to GLuint"))?;

                    unsafe {
                        gl::EnableVertexAttribArray(attr.location.into());
                        gl::VertexAttribFormat(
                            attr.location.into(),
                            attr.components.count().into(),
                            attr.kind.into(),
                            attr.normalized.into(),
                            local_offset,
                        );
                        gl::VertexAttribBinding(attr.location.into(), index);
                    }
                }
                Ok(())
            })
    }

    pub fn set_buffers(
        attributes: &[BufferAttributes],
        buffers: &GenVec<handle::Buffer, super::buffer::Native>,
    ) -> Result<(), Error> {
        for (location, buffer_attribute) in attributes.iter().enumerate() {
            if let Some(buffer) = buffer_attribute.buffer {
                let vbo = buffers.get(buffer).ok_or(Error::ResourceNotFound)?;

                let location = location
                    .try_into()
                    .map_err(|_| Error::ConversionFailed("buffer location wraps"))?;

                let offset = buffer_attribute
                    .offset
                    .try_into()
                    .map_err(|_| Error::ConversionFailed("buffer offset wraps"))?;

                let stride = buffer_attribute
                    .stride()
                    .try_into()
                    .map_err(|_| Error::ConversionFailed("buffer stride wraps"))?;

                unsafe {
                    gl::BindVertexBuffer(location, vbo.id, offset, stride);
                }
            }
        }
        Ok(())
    }
}

impl crate::vertex_layout::Native for Native {}

impl Drop for Native {
    fn drop(&mut self) {
        log::trace!("Dropped vertex layout {}.", self.id);
        unsafe { gl::DeleteVertexArrays(1, &self.id) }
    }
}
