use std::ffi::{CStr, CString};

use crate::{
    error::Error,
    shader::{self, Stage},
};

use super::gl::{
    self,
    types::{GLenum, GLint, GLuint},
};

pub(super) struct Shader {
    pub(crate) id: GLuint,
    kind: shader::Kind,
}

pub(super) struct Program {
    pub(super) id: GLuint,
}

impl From<shader::Kind> for GLenum {
    fn from(value: shader::Kind) -> Self {
        match value {
            shader::Kind::Vertex => gl::VERTEX_SHADER,
            shader::Kind::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

impl Shader {
    fn new(kind: shader::Kind, sources: &[&str]) -> Result<Self, Error> {
        let shader = Self {
            id: unsafe { gl::CreateShader(kind.into()) },
            kind,
        };

        shader.compile(sources)?;
        Ok(shader)
    }
    fn compile(&self, sources: &[&str]) -> Result<(), Error> {
        ////convert Rust Str to CString

        let sources = sources
            .iter()
            .map(|s| {
                CString::new(*s)
                    .map_err(|e| Error::FailedToCompileShader(format!("conversion failed: {e}")))
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

            let mut error_string = vec![b'0'; error_length as usize];

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

impl Program {
    fn link(&self) -> Result<(), Error> {
        let mut link_status = 0;
        unsafe {
            gl::LinkProgram(self.id);
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut link_status);
        }

        //link_status == 0 means there is a link error
        if link_status == 1 {
            Ok(())
        } else {
            let mut error_length = 0;

            unsafe {
                gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut error_length);
            }

            let mut error_string: Vec<u8> =
                Vec::with_capacity(usize::try_from(error_length).unwrap() + 1);
            error_string.extend(
                std::iter::once(&b' ')
                    .cycle()
                    .take(error_length.try_into().unwrap()),
            );

            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
                    error_length,
                    std::ptr::null_mut(),
                    error_string.as_mut_ptr().cast(),
                );
            }

            let reason = String::from_utf8_lossy(&error_string).to_string();
            Err(Error::FailedToLinkShader(reason))
        }
    }
}

impl<C: super::GLContext> crate::shader::Context for super::Context<C> {
    fn new(&mut self, stages: &[crate::shader::Stage]) -> Result<shader::Shader, Error> {
        let program = Program {
            id: unsafe { gl::CreateProgram() },
        };

        let mut shaders = Vec::with_capacity(stages.len());

        for stage in stages {
            if let Some(shader) = self.stages.get(stage.handle) {
                shaders.push(shader.id);
            } else {
                return Err(Error::ResourceNotFound);
            }
        }

        for id in &shaders {
            unsafe { gl::AttachShader(program.id, *id) };
        }

        program.link()?;

        for id in shaders {
            unsafe { gl::DetachShader(program.id, id) };
        }

        let index = self.shaders.len();
        self.shaders.push(program);

        Ok(shader::Shader { handle: index })
    }

    fn with_sources(
        &mut self,
        vertex_shader: &[&str],
        fragment_shader: &[&str],
    ) -> Result<shader::Shader, crate::error::Error> {
        let program = Program {
            id: unsafe { gl::CreateProgram() },
        };

        let vertex_shader = Shader::new(shader::Kind::Vertex, vertex_shader)?;
        let fragment_shader = Shader::new(shader::Kind::Fragment, fragment_shader)?;

        unsafe {
            gl::AttachShader(program.id, vertex_shader.id);
            gl::AttachShader(program.id, fragment_shader.id);

            program.link()?;

            gl::DetachShader(program.id, vertex_shader.id);
            gl::DetachShader(program.id, fragment_shader.id);
        }

        let index = self.shaders.len();
        self.shaders.push(program);

        Ok(shader::Shader { handle: index })
    }

    fn new_stage(
        &mut self,
        kind: shader::Kind,
        sources: &[&str],
    ) -> Result<crate::shader::Stage, crate::error::Error> {
        let shader = Shader::new(kind, sources)?;

        let index = self.stages.len();
        self.stages.push(shader);

        Ok(Stage {
            handle: index,
            kind,
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        log::trace!("Dropped {} shader {}.", self.kind, self.id);
        unsafe { gl::DeleteShader(self.id) }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        log::trace!("Dropped shader program {}.", self.id);
        unsafe { gl::DeleteProgram(self.id) }
    }
}
