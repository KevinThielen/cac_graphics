use super::{
    gl::{self, types::GLenum},
    Context, GLContext,
};
use crate::{buffer, error::Error};

pub struct Buffer {
    id: gl::types::GLuint,
    kind: GLenum,
    usage: GLenum,
}

struct AccessUsage(buffer::Access, buffer::Usage);

impl buffer::NativeContext for Buffer {
    fn set_data<T>(&mut self, data: &[T]) -> Result<(), Error> {
        unsafe {
            gl::BindBuffer(self.kind, self.id);
            gl::BufferData(
                self.kind,
                (data.len() * std::mem::size_of::<T>()).try_into().unwrap(),
                data.as_ptr().cast(),
                self.usage,
            );
        }

        Ok(())
    }
}

impl<C: GLContext> buffer::Context for Context<C> {
    type NativeBuffer = Buffer;

    fn create(
        &mut self,
        kind: buffer::Kind,
        access: buffer::Access,
        usage: buffer::Usage,
    ) -> buffer::Buffer {
        let buffer = Self::NativeBuffer {
            id: unsafe {
                let mut buffer = 0;
                gl::GenBuffers(1, &mut buffer);
                buffer
            },
            kind: kind.into(),
            usage: AccessUsage(access, usage).into(),
        };

        let handle = buffer::Buffer {
            handle: self.buffers.len(),
        };
        self.buffers.push(buffer);

        handle
    }

    fn get_mut(&mut self, handle: usize) -> Option<&mut Self::NativeBuffer> {
        self.buffers.get_mut(handle)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        log::trace!("Dropped {} {}.", self.kind, self.id);
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}

#[allow(clippy::use_self)] //false positive because of trait implementation
impl From<buffer::Kind> for GLenum {
    fn from(value: buffer::Kind) -> Self {
        match value {
            buffer::Kind::Vertex => gl::ARRAY_BUFFER,
        }
    }
}

#[allow(clippy::use_self)] //false positive because of trait implementation
impl From<AccessUsage> for GLenum {
    fn from(value: AccessUsage) -> Self {
        use buffer::{
            Access::{Always, Frequent, Once},
            Usage::{Copy, Read, Write},
        };

        match (value.0, value.1) {
            (Once, Write) => gl::STATIC_DRAW,
            (Frequent, Write) => gl::DYNAMIC_DRAW,
            (Always, Write) => gl::STREAM_DRAW,

            (Once, Read) => gl::STATIC_READ,
            (Frequent, Read) => gl::DYNAMIC_READ,
            (Always, Read) => gl::STREAM_READ,

            (Once, Copy) => gl::STATIC_COPY,
            (Frequent, Copy) => gl::DYNAMIC_COPY,
            (Always, Copy) => gl::STREAM_COPY,
        }
    }
}
