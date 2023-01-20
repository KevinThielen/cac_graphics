use super::gl::{self, types::GLenum};
use crate::{
    buffer::{self, FlatData},
    error::Error,
};

pub struct Native {
    pub(crate) id: gl::types::GLuint,
    kind: GLenum,
    usage: GLenum,
}

struct AccessUsage(buffer::Access, buffer::Usage);

impl Native {
    pub(super) fn new<T: FlatData>(buffer: &crate::Buffer<T>) -> Result<Self, Error> {
        let mut b = Self {
            id: unsafe {
                let mut buffer = 0;
                gl::GenBuffers(1, &mut buffer);
                buffer
            },
            kind: buffer.kind.into(),
            usage: AccessUsage(buffer.access, buffer.usage).into(),
        };

        if let Some(data) = buffer.data {
            b.set_data(data)?;
        }

        Ok(b)
    }

    fn set_data<T: buffer::FlatData>(&mut self, data: &[T]) -> Result<(), Error> {
        let size = (data.len() * std::mem::size_of::<T>())
            .try_into()
            .map_err(|_| Error::ConversionFailed("buffer length into i32"))?;

        unsafe {
            gl::BindBuffer(self.kind, self.id);
            gl::BufferData(self.kind, size, data.as_ptr().cast(), self.usage);
        }

        Ok(())
    }
}

impl crate::buffer::Native for Native {
    fn set_data<T: buffer::FlatData>(&mut self, data: &[T]) -> Result<(), Error> {
        self.set_data(data)
    }
}

impl Drop for Native {
    fn drop(&mut self) {
        log::trace!(
            "Dropped {kind} buffer {id}.",
            id = self.id,
            kind = buffer::Kind::try_from(self.kind)
                .map_or_else(|_| String::from("unknown"), |k| k.to_string()),
        );
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

impl TryFrom<GLenum> for buffer::Kind {
    type Error = crate::Error;
    fn try_from(value: GLenum) -> Result<Self, Self::Error> {
        match value {
            gl::ARRAY_BUFFER => Ok(Self::Vertex),
            _ => Err(Error::ConversionFailed("glenum to bufferkind")),
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
