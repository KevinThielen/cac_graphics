mod buffer;
mod render_rarget;
mod shader;
mod stage;
mod vertex_layout;

mod gl43_core;

use crate::{
    buffer::FlatData, error::Error, handle, BufferHandle, RenderTargetHandle, ShaderHandle,
    StageHandle, VertexLayoutHandle,
};

use gl43_core as gl;

use core::gen_vec::GenVec;

pub trait GLContext {
    fn swap_buffers(&mut self);
    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void;
}

pub struct Context<C: GLContext> {
    gl_context: C,
    buffers: GenVec<handle::Buffer, buffer::Native>,
    layouts: GenVec<handle::VertexLayout, vertex_layout::Native>,
    stages: GenVec<handle::Stage, stage::Native>,
    shaders: GenVec<handle::Shader, shader::Native>,
    render_targets: GenVec<handle::RenderTarget, render_rarget::Native>,

    bound_layout: Option<VertexLayoutHandle>,
    bound_shader: Option<ShaderHandle>,
    bound_render_target: Option<RenderTargetHandle>,
}

impl<C: GLContext> Context<C> {
    /// Creates an OpenGL 4.3 context.
    ///
    /// # Errors
    /// `Error::InvalidContext`: When the context fails to load the function pointers or is using
    /// an unsupported version(< 4.3 or a non-existant version).
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
            buffers: GenVec::with_capacity(10),
            layouts: GenVec::with_capacity(10),
            stages: GenVec::with_capacity(10),
            shaders: GenVec::with_capacity(10),
            render_targets: GenVec::with_capacity(10),
            bound_layout: None,
            bound_shader: None,
            bound_render_target: None,
        })
    }

    pub fn raw_context(&mut self) -> &mut C {
        &mut self.gl_context
    }

    fn bind_render_target(&mut self, render_rarget: RenderTargetHandle) -> Result<(), Error> {
        if self.bound_render_target != Some(render_rarget) {
            self.bound_render_target = Some(render_rarget);
            if let Some(rt) = self.render_targets.get_mut(render_rarget) {
                rt.bind()?;
            } else {
                return Err(Error::ResourceNotFound);
            }
        }
        Ok(())
    }

    fn bind_shader(&mut self, shader: ShaderHandle) -> Result<(), Error> {
        if self.bound_shader != Some(shader) {
            self.bound_shader = Some(shader);
            if let Some(s) = self.shaders.get_mut(shader) {
                s.bind();
            } else {
                return Err(Error::ResourceNotFound);
            }
        }
        Ok(())
    }

    fn bind_layout(&mut self, layout: VertexLayoutHandle) -> Result<(), Error> {
        if self.bound_layout != Some(layout) {
            self.bound_layout = Some(layout);
            if let Some(l) = self.layouts.get_mut(layout) {
                l.bind();
            } else {
                return Err(Error::ResourceNotFound);
            }
        }
        Ok(())
    }
}

impl<C: GLContext> crate::Context for Context<C> {
    type Buffer = buffer::Native;
    type Layout = vertex_layout::Native;
    type Shader = shader::Native;
    type Stage = stage::Native;
    type RenderTarget = render_rarget::Native;

    fn update(&mut self) {
        self.gl_context.swap_buffers();
    }

    fn draw(
        &mut self,
        render_rarget: RenderTargetHandle,
        primitive: crate::Primitive,
        shader: ShaderHandle,
        layout: VertexLayoutHandle,
        start: usize,
        count: usize,
    ) -> std::result::Result<(), Error> {
        self.bind_render_target(render_rarget)?;
        self.bind_shader(shader)?;
        self.bind_layout(layout)?;

        let start = start
            .try_into()
            .map_err(|_| Error::ConversionFailed("start wraps around i32"))?;

        let count = count
            .try_into()
            .map_err(|_| Error::ConversionFailed("count wraps around i32"))?;

        unsafe {
            gl::DrawArrays(primitive.into(), start, count);
        }

        Ok(())
    }

    /*******************************
     *          RENDER TARGET
     *******************************/
    fn create_render_target(
        &mut self,
        render_target: crate::RenderTarget,
    ) -> Result<crate::RenderTargetHandle, Error> {
        let rt = Self::RenderTarget::new(render_target);
        Ok(self.render_targets.insert(rt))
    }

    fn render_target(&self, handle: crate::RenderTargetHandle) -> Option<&Self::RenderTarget> {
        self.render_targets.get(handle)
    }

    fn render_target_mut(
        &mut self,
        handle: crate::RenderTargetHandle,
    ) -> Option<&mut Self::RenderTarget> {
        if self.bind_render_target(handle).is_ok() {
            self.render_targets.get_mut(handle)
        } else {
            None
        }
    }

    /*******************************
     *          BUFFER
     *******************************/
    fn create_buffer<T: FlatData>(
        &mut self,
        buffer: &crate::Buffer<T>,
    ) -> Result<BufferHandle, Error> {
        let buffer = Self::Buffer::new(buffer)?;
        Ok(self.buffers.insert(buffer))
    }

    fn buffer(&self, handle: BufferHandle) -> Option<&Self::Buffer> {
        self.buffers.get(handle)
    }

    fn buffer_mut(&mut self, handle: BufferHandle) -> Option<&mut Self::Buffer> {
        self.buffers.get_mut(handle)
    }

    /*******************************
     *          VertexLayout
     *******************************/
    fn create_layout(&mut self, layout: &crate::VertexLayout) -> Result<VertexLayoutHandle, Error> {
        let layout = vertex_layout::Native::new(layout, &self.buffers)?;
        let handle = self.layouts.insert(layout);

        self.bind_layout(handle)?;

        Ok(handle)
    }
    fn layout(&self, handle: VertexLayoutHandle) -> Option<&Self::Layout> {
        self.layouts.get(handle)
    }

    fn layout_mut(&mut self, handle: VertexLayoutHandle) -> Option<&mut Self::Layout> {
        if self.bind_layout(handle).is_ok() {
            self.layouts.get_mut(handle)
        } else {
            None
        }
    }

    fn layout_mut_and_buffers(
        &mut self,
        handle: VertexLayoutHandle,
        buffer_handles: &[BufferHandle],
    ) -> (Option<&mut Self::Layout>, Vec<Option<&Self::Buffer>>) {
        let layout = if self.bind_layout(handle).is_ok() {
            self.layouts.get_mut(handle)
        } else {
            None
        };

        let buffers = buffer_handles
            .iter()
            .map(|b| self.buffers.get(*b))
            .collect();

        (layout, buffers)
    }

    /*******************************
     *          Shader
     *******************************/

    fn create_stage(&mut self, shader: crate::shader::Stage) -> Result<StageHandle, Error> {
        let stage = Self::Stage::new(shader)?;
        Ok(self.stages.insert(stage))
    }

    fn stage(&self, handle: StageHandle) -> Option<&Self::Stage> {
        self.stages.get(handle)
    }

    fn create_shader(&mut self, shader: crate::shader::Shader) -> Result<ShaderHandle, Error> {
        let shader = Self::Shader::new(shader, &self.stages)?;

        Ok(self.shaders.insert(shader))
    }

    fn shader(&self, handle: ShaderHandle) -> Option<&Self::Shader> {
        self.shaders.get(handle)
    }

    fn shader_mut(&mut self, handle: ShaderHandle) -> Option<&mut Self::Shader> {
        if self.bind_shader(handle).is_ok() {
            self.shaders.get_mut(handle)
        } else {
            None
        }
    }
}

impl From<crate::Primitive> for gl::types::GLenum {
    fn from(value: crate::Primitive) -> Self {
        use gl::{TRIANGLES, TRIANGLE_STRIP};

        match value {
            crate::Primitive::Triangles => TRIANGLES,
            crate::Primitive::TriangleStrip => TRIANGLE_STRIP,
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
