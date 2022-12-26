use crate::{
    error::Error,
    vertex_layout::{self, Stride},
};

use super::{
    gl::{
        self,
        types::{GLenum, GLint, GLuint},
    },
    Context, GLContext,
};

pub struct VertexLayout {
    pub(super) id: gl::types::GLuint,
}

impl From<vertex_layout::AttributeKind> for GLenum {
    fn from(value: vertex_layout::AttributeKind) -> Self {
        match value {
            vertex_layout::AttributeKind::F32 => gl::FLOAT,
            vertex_layout::AttributeKind::U8 => gl::BYTE,
        }
    }
}

impl crate::vertex_layout::NativeContext for VertexLayout {
    type NativeBuffer = super::buffer::Buffer;

    fn set_attribute_buffer(&mut self, buffer: &Self::NativeBuffer) -> Result<(), Error> {
        Err(Error::ResourceNotFound)
    }
}

impl<C: GLContext> crate::vertex_layout::Context for Context<C> {
    type NativeLayout = VertexLayout;

    fn get_mut_with_buffers<const N: usize>(
        &mut self,
        handle: vertex_layout::VertexLayout,
        buffers: [crate::buffer::Buffer; N],
    ) -> (&mut Self::NativeLayout, [Option<&Self::NativeBuffer>; N]) {
        let layout = self.layouts.get_mut(handle.handle).unwrap();

        let mut b = [None; N];

        for i in 0..N {
            b[i] = self.buffers.get(buffers[i].handle);
        }

        (layout, b)
    }

    fn create(&mut self) -> crate::vertex_layout::VertexLayout {
        let vao = VertexLayout {
            id: unsafe {
                let mut vao = 0;
                gl::GenVertexArrays(1, &mut vao);
                vao
            },
        };

        let handle = vertex_layout::VertexLayout {
            handle: self.layouts.len(),
        };
        self.layouts.push(vao);

        handle
    }

    fn set_attributes(
        &mut self,
        handle: vertex_layout::VertexLayout,
        index: u8,
        attributes: &[vertex_layout::VertexAttribute],
    ) -> Result<(), crate::error::Error> {
        self.bind_vao(Some(handle))?;

        for attr in attributes {
            unsafe {
                gl::EnableVertexAttribArray(attr.location.into());
                gl::VertexAttribFormat(
                    attr.location.into(),
                    attr.components.count().into(),
                    attr.kind.into(),
                    attr.normalized.into(),
                    attr.local_offset as GLuint,
                );
                gl::VertexAttribBinding(attr.location.into(), index.into());
            }
        }

        Ok(())
    }

    fn set_attribute_buffer(
        &mut self,
        location: u8,
        handle: vertex_layout::VertexLayout,
        buffer: crate::buffer::Buffer,
        offset: usize,
        stride: Stride,
    ) -> Result<(), Error> {
        self.bind_vao(Some(handle))?;

        let vbo = self
            .buffers
            .get(buffer.handle)
            .ok_or(Error::ResourceNotFound)?;

        unsafe {
            gl::BindVertexBuffer(
                location.into(),
                vbo.id,
                offset.try_into().unwrap(),
                stride.stride_bytes() as GLint,
            );
        }
        Ok(())
    }
}

impl Drop for VertexLayout {
    fn drop(&mut self) {
        log::trace!("Dropped vertex array object {}.", self.id);
        unsafe { gl::DeleteVertexArrays(1, &self.id) }
    }
}
