use std::ffi::CString;

use super::gl::{
    self,
    types::{GLenum, GLuint},
};
use crate::{
    shader::{self, Stage},
    Error,
};

pub struct Native {
    pub(crate) id: GLuint,
    kind: shader::Kind,
}

impl From<shader::Kind> for GLenum {
    fn from(value: shader::Kind) -> Self {
        use gl::{FRAGMENT_SHADER, VERTEX_SHADER};

        match value {
            shader::Kind::Vertex => VERTEX_SHADER,
            shader::Kind::Fragment => FRAGMENT_SHADER,
        }
    }
}

impl Native {
    pub(super) fn new(stage: Stage) -> Result<Self, Error> {
        let shader = Self {
            id: unsafe { gl::CreateShader(stage.kind.into()) },
            kind: stage.kind,
        };

        shader.compile(stage.sources)?;
        Ok(shader)
    }

    fn compile(&self, sources: &[&str]) -> Result<(), Error> {
        ////convert Rust Str to CString

        let sources = sources
            .iter()
            .map(|s| {
                CString::new(*s).map_err(|_e| {
                    Error::FailedToCompileShader(String::from("source is not a valid CString"))
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let sources_ptrs: Vec<_> = sources.iter().map(|s| s.as_ptr()).collect();
        //
        //let sources_ptr: Vec<_> = sources.iter().map(|s| s.as_ptr()).collect();
        //let sources_lengths: Vec<_> = sources.iter().map(|s| s.len() as GLint).collect();

        let mut compile_status = 0;
        unsafe {
            gl::ShaderSource(
                self.id,
                sources
                    .len()
                    .try_into()
                    .expect("failed to convert len to i32"),
                sources_ptrs.as_ptr().cast(),
                std::ptr::null(),
            );
            gl::CompileShader(self.id);
            gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut compile_status);
        }

        // 0 means error
        if compile_status == 1 {
            Ok(())
        } else {
            //need to do some annoying dance to get the actual compile error for the shader through
            //the ffi nonsense
            let mut error_length = 0;
            unsafe {
                gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut error_length);
            }

            let error_length_size = error_length
                .try_into()
                .map_err(|_| Error::ConversionFailed("failed to convert stage error length"))?;

            let mut error_string = vec![b'0'; error_length_size];

            unsafe {
                gl::GetShaderInfoLog(
                    self.id,
                    error_length,
                    std::ptr::null_mut(),
                    error_string.as_mut_ptr().cast(),
                );
            }
            let reason = String::from_utf8_lossy(&error_string).to_string();

            Err(Error::FailedToCompileShader(reason))
        }
    }
}

impl Drop for Native {
    fn drop(&mut self) {
        log::trace!("Dropped {} shader {}.", self.kind, self.id);
        unsafe { gl::DeleteShader(self.id) }
    }
}
