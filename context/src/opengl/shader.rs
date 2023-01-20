use cac_core::gen_vec::GenVec;

use crate::{error::Error, shader::Shader};

use super::{
    gl::{self, types::GLuint},
    stage,
};

pub struct Native {
    pub(super) id: GLuint,
}

impl Native {
    pub(super) fn new(
        shader: Shader,
        stages: &GenVec<crate::handle::Stage, super::stage::Native>,
    ) -> Result<Self, Error> {
        let program = Self {
            id: unsafe { gl::CreateProgram() },
        };
        let temp_stages = shader
            .stage_sources
            .iter()
            .map(|stage| stage::Native::new(*stage))
            .collect::<Result<Vec<stage::Native>, Error>>()?;

        let stages = shader
            .stages
            .iter()
            .map(|stage| stages.get(*stage))
            .collect::<Option<Vec<&stage::Native>>>()
            .ok_or(Error::ResourceNotFound)?;

        temp_stages
            .iter()
            .chain(stages.iter().copied())
            .for_each(|stage| {
                unsafe { gl::AttachShader(program.id, stage.id) };
            });

        program.link()?;

        temp_stages
            .iter()
            .chain(stages.into_iter())
            .for_each(|stage| {
                unsafe { gl::DetachShader(program.id, stage.id) };
            });

        Ok(program)
    }

    pub(super) fn bind(&mut self) {
        unsafe { gl::UseProgram(self.id) }
    }

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

impl crate::shader::Native for Native {}

impl Drop for Native {
    fn drop(&mut self) {
        log::trace!("Dropped shader program {}.", self.id);
        unsafe { gl::DeleteProgram(self.id) }
    }
}
