mod buffer;
mod render_rarget;
mod shader;
mod vertex_layout;

mod gl43_core;
use gl43_core as gl;

use crate::error::Error;

use self::gl43_core::types::GLint;

pub trait GLContext {
    fn swap_buffers(&mut self);
    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void;
}

pub struct Context<C: GLContext> {
    gl_context: C,
    buffers: Vec<buffer::Buffer>,
    layouts: Vec<vertex_layout::VertexLayout>,
    stages: Vec<shader::Shader>,
    shaders: Vec<shader::Program>,

    bound_vao: Option<crate::vertex_layout::VertexLayout>,
    bound_shader: Option<crate::shader::Shader>,
}

impl<C: GLContext> Context<C> {
    pub fn new(mut context: C) -> Result<Self, Error> {
        gl::load_with(|name| context.get_proc_address(name));

        if !gl::GetIntegerv::is_loaded() {
            return Err(Error::InvalidContext(String::from(
                "failed to load OpenGL fn pointers",
            )));
        }

        let mut version = (0, 0);
        unsafe {
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut version.0);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut version.1);
        }

        if version.0 < 4 || (version.0 == 4 && version.1 < 3) {
            return Err(Error::InvalidContext(format!(
                "version 4.3 required, received {}.{}",
                version.0, version.1
            )));
        }

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());
        }

        Ok(Self {
            gl_context: context,
            buffers: Vec::with_capacity(10),
            layouts: Vec::with_capacity(10),
            stages: Vec::with_capacity(10),
            shaders: Vec::with_capacity(10),
            bound_vao: None,
            bound_shader: None,
        })
    }

    pub fn raw_context(&mut self) -> &mut C {
        &mut self.gl_context
    }

    fn bind_vao(
        &mut self,
        handle: Option<crate::vertex_layout::VertexLayout>,
    ) -> Result<(), Error> {
        if handle != self.bound_vao {
            if let Some(layout) = handle {
                let vao = self
                    .layouts
                    .get(layout.handle)
                    .ok_or(Error::ResourceNotFound)?;

                unsafe {
                    gl::BindVertexArray(vao.id);
                }

                self.bound_vao = handle;
            }
        }

        Ok(())
    }

    fn bind_shader(&mut self, handle: Option<crate::shader::Shader>) -> Result<(), Error> {
        if handle != self.bound_shader {
            if let Some(shader) = handle {
                let shader = self
                    .shaders
                    .get(shader.handle)
                    .ok_or(Error::ResourceNotFound)?;

                unsafe {
                    gl::UseProgram(shader.id);
                }

                self.bound_shader = handle;
            }
        }

        Ok(())
    }
}

impl<C: GLContext> crate::Context for Context<C> {
    fn update(&mut self) {
        self.gl_context.swap_buffers();
    }

    fn draw(
        &mut self,
        _target: &crate::render_target::RenderTarget,
        primitive: crate::Primitive,
        shader: crate::shader::Shader,
        layout: crate::vertex_layout::VertexLayout,
        start: usize,
        count: usize,
    ) -> std::result::Result<(), Error> {
        self.bind_shader(Some(shader))?;
        self.bind_vao(Some(layout))?;

        unsafe { gl::DrawArrays(primitive.into(), start as GLint, count as GLint) }

        Ok(())
    }
}

impl From<crate::Primitive> for gl::types::GLenum {
    fn from(value: crate::Primitive) -> Self {
        match value {
            crate::Primitive::Triangles => gl::TRIANGLES,
            crate::Primitive::TriangleStrip => gl::TRIANGLE_STRIP,
        }
    }
}

extern "system" fn debug_callback(
    source: u32,
    kind: u32,
    id: u32,
    severity: u32,
    _length: i32,
    message: *const i8,
    _user_param: *mut std::ffi::c_void,
) {
    let source = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER COMPILER",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "WINDOW SYSTEM",
        gl::DEBUG_SOURCE_OTHER => "OTHER",
        gl::DEBUG_SOURCE_APPLICATION => "APPLICATION",
        gl::DEBUG_SOURCE_THIRD_PARTY => "THIRD PARTY",
        _ => "UNKNOWN",
    };

    let kind = match kind {
        gl::DEBUG_TYPE_ERROR => "ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED BEHAVIOUR",
        gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
        _ => "UNKNOWN",
    };

    let error_message = unsafe {
        std::ffi::CStr::from_ptr(message)
            .to_str()
            .unwrap_or("[FAILED TO READ GL ERROR MESSAGE]")
    };

    match severity {
        gl::DEBUG_SEVERITY_HIGH => log::error!("{id}: {kind} from {source}: {error_message}"),
        gl::DEBUG_SEVERITY_MEDIUM => log::warn!("{id}: {kind} from {source}: {error_message}"),
        gl::DEBUG_SEVERITY_LOW => log::info!("{id}: {kind} from {source}: {error_message}"),
        gl::DEBUG_SEVERITY_NOTIFICATION => {
            log::trace!("{id}: {kind} from {source}: {error_message}");
        }
        _ => log::trace!("{id}: {kind} from {source}: {error_message}"),
    };
}
