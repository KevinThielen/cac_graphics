mod buffer;
mod render_target;
mod shader;
mod stage;
mod vertex_layout;

mod gl43_core;

use crate::{
    buffer::FlatData, error::Error, handle, BufferHandle, RenderTargetHandle, ShaderHandle,
    StageHandle, VertexLayoutHandle,
};

use gl43_core as gl;

use cac_core::{gen_vec::GenVec, math::URect};

thread_local! {
    static ERROR_LOGS: Vec<String> = Vec::new();
}

pub trait GLContext {
    fn swap_buffers(&mut self);
    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void;
}

struct Resources {
    buffers: GenVec<handle::Buffer, buffer::Native>,
    layouts: GenVec<handle::VertexLayout, vertex_layout::Native>,
    stages: GenVec<handle::Stage, stage::Native>,
    shaders: GenVec<handle::Shader, shader::Native>,
    render_targets: GenVec<handle::RenderTarget, render_target::Native>,
}

impl Resources {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffers: GenVec::with_capacity(capacity),
            layouts: GenVec::with_capacity(capacity),
            stages: GenVec::with_capacity(capacity),
            shaders: GenVec::with_capacity(capacity),
            render_targets: GenVec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::with_capacity(10);
    }
}

#[derive(Default)]
struct State {
    pub bound_layout: Option<VertexLayoutHandle>,
    pub bound_shader: Option<ShaderHandle>,
    pub bound_render_target: Option<RenderTargetHandle>,
}

impl State {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn bind_render_target(
        &mut self,
        resources: &mut Resources,
        render_rarget: RenderTargetHandle,
    ) -> Result<(), Error> {
        if self.bound_render_target != Some(render_rarget) {
            self.bound_render_target = Some(render_rarget);
            if let Some(rt) = resources.render_targets.get_mut(render_rarget) {
                rt.bind()?;
            } else {
                return Err(Error::ResourceNotFound);
            }
        }
        Ok(())
    }

    pub fn bind_shader(
        &mut self,
        resources: &mut Resources,
        shader: ShaderHandle,
    ) -> Result<(), Error> {
        if self.bound_shader != Some(shader) {
            self.bound_shader = Some(shader);
            if let Some(s) = resources.shaders.get_mut(shader) {
                s.bind();
            } else {
                return Err(Error::ResourceNotFound);
            }
        }
        Ok(())
    }

    pub fn bind_layout(
        &mut self,
        resources: &mut Resources,
        layout: VertexLayoutHandle,
    ) -> Result<(), Error> {
        if self.bound_layout != Some(layout) {
            self.bound_layout = Some(layout);
            if let Some(l) = resources.layouts.get_mut(layout) {
                l.bind();
            } else {
                return Err(Error::ResourceNotFound);
            }
        }
        Ok(())
    }

    pub fn bind_draw_state(
        &mut self,
        resources: &mut Resources,
        render_rarget: RenderTargetHandle,
        layout: VertexLayoutHandle,
        shader: ShaderHandle,
    ) -> Result<(), Error> {
        self.bind_render_target(resources, render_rarget)?;
        self.bind_layout(resources, layout)?;
        self.bind_shader(resources, shader)?;
        Ok(())
    }
}

pub struct Context<C: GLContext> {
    gl_context: C,
    resources: Resources,
    state: State,

    //Boxing the collection is fine in this case, because it provides a stable adress to the
    //collection, that can be send over FFI.
    #[allow(clippy::box_collection)]
    error_log: Box<Vec<String>>,

    viewport: URect,
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

        let viewport = unsafe {
            let mut data = [0; 4];
            gl::GetIntegerv(gl::VIEWPORT, data.as_mut_ptr());

            let x = data[0]
                .try_into()
                .map_err(|_| Error::ConversionFailed("viewport x to u32"))?;

            let y = data[1]
                .try_into()
                .map_err(|_| Error::ConversionFailed("viewport y to u32"))?;

            let width = data[2]
                .try_into()
                .map_err(|_| Error::ConversionFailed("viewport width to u32"))?;

            let height = data[3]
                .try_into()
                .map_err(|_| Error::ConversionFailed("viewport height to u32"))?;

            URect {
                x,
                y,
                width,
                height,
            }
        };

        let mut ctx = Self {
            gl_context: context,
            error_log: Box::default(),
            viewport,
            resources: Resources::with_capacity(10),
            state: State::default(),
        };

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(
                Some(debug_callback),
                std::ptr::addr_of_mut!(*ctx.error_log).cast(),
            );

            gl::Enable(gl::SCISSOR_TEST);
        }

        Ok(ctx)
    }

    pub fn raw_context(&mut self) -> &mut C {
        &mut self.gl_context
    }
}

impl<C: GLContext> crate::Context for Context<C> {
    type Buffer = buffer::Native;
    type Layout = vertex_layout::Native;
    type Shader = shader::Native;
    type Stage = stage::Native;
    type RenderTarget = render_target::Native;

    fn update(&mut self) {
        self.gl_context.swap_buffers();
    }

    fn poll_errors(&mut self) -> Option<Vec<String>> {
        if self.error_log.is_empty() {
            None
        } else {
            let log = self.error_log.to_vec();
            self.error_log.clear();
            Some(log)
        }
    }
    fn reset(&mut self) {
        self.resources.clear();
        self.state.reset();
        self.error_log.clear();
    }

    fn viewport(&self) -> URect {
        self.viewport
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
        self.state
            .bind_draw_state(&mut self.resources, render_rarget, layout, shader)?;

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
        Ok(self.resources.render_targets.insert(rt))
    }

    fn render_target(&self, handle: crate::RenderTargetHandle) -> Option<&Self::RenderTarget> {
        self.resources.render_targets.get(handle)
    }

    fn render_target_mut(
        &mut self,
        handle: crate::RenderTargetHandle,
    ) -> Option<&mut Self::RenderTarget> {
        if self
            .state
            .bind_render_target(&mut self.resources, handle)
            .is_ok()
        {
            self.resources.render_targets.get_mut(handle)
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
        Ok(self.resources.buffers.insert(buffer))
    }

    fn buffer(&self, handle: BufferHandle) -> Option<&Self::Buffer> {
        self.resources.buffers.get(handle)
    }

    fn buffer_mut(&mut self, handle: BufferHandle) -> Option<&mut Self::Buffer> {
        self.resources.buffers.get_mut(handle)
    }

    /*******************************
     *          VertexLayout
     *******************************/
    fn create_layout(&mut self, layout: &crate::VertexLayout) -> Result<VertexLayoutHandle, Error> {
        let layout = vertex_layout::Native::new(layout, &self.resources.buffers)?;
        let handle = self.resources.layouts.insert(layout);

        self.state.bind_layout(&mut self.resources, handle)?;

        Ok(handle)
    }
    fn layout(&self, handle: VertexLayoutHandle) -> Option<&Self::Layout> {
        self.resources.layouts.get(handle)
    }

    fn layout_mut(&mut self, handle: VertexLayoutHandle) -> Option<&mut Self::Layout> {
        if self.state.bind_layout(&mut self.resources, handle).is_ok() {
            self.resources.layouts.get_mut(handle)
        } else {
            None
        }
    }

    fn layout_mut_and_buffers(
        &mut self,
        handle: VertexLayoutHandle,
        buffer_handles: &[BufferHandle],
    ) -> (Option<&mut Self::Layout>, Vec<Option<&Self::Buffer>>) {
        let layout = if self.state.bind_layout(&mut self.resources, handle).is_ok() {
            self.resources.layouts.get_mut(handle)
        } else {
            None
        };

        let buffers = buffer_handles
            .iter()
            .map(|b| self.resources.buffers.get(*b))
            .collect();

        (layout, buffers)
    }

    /*******************************
     *          Shader
     *******************************/

    fn create_stage(&mut self, shader: crate::shader::Stage) -> Result<StageHandle, Error> {
        let stage = Self::Stage::new(shader)?;
        Ok(self.resources.stages.insert(stage))
    }

    fn stage(&self, handle: StageHandle) -> Option<&Self::Stage> {
        self.resources.stages.get(handle)
    }

    fn create_shader(&mut self, shader: crate::shader::Shader) -> Result<ShaderHandle, Error> {
        let shader = Self::Shader::new(shader, &self.resources.stages)?;

        Ok(self.resources.shaders.insert(shader))
    }

    fn shader(&self, handle: ShaderHandle) -> Option<&Self::Shader> {
        self.resources.shaders.get(handle)
    }

    fn shader_mut(&mut self, handle: ShaderHandle) -> Option<&mut Self::Shader> {
        if self.state.bind_shader(&mut self.resources, handle).is_ok() {
            self.resources.shaders.get_mut(handle)
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
    user_param: *mut std::ffi::c_void,
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

    let kind_str = match kind {
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

    let error_message = format!("{id}: {kind_str} from {source}: {error_message}");

    match severity {
        gl::DEBUG_SEVERITY_HIGH => log::error!("{error_message}"),
        gl::DEBUG_SEVERITY_MEDIUM => log::warn!("{error_message}"),
        gl::DEBUG_SEVERITY_LOW => log::info!("{error_message}"),
        gl::DEBUG_SEVERITY_NOTIFICATION => {
            log::trace!("{error_message}");
        }
        _ => log::trace!("{error_message}"),
    };

    if !user_param.is_null() {
        let vec_ptr: *mut Vec<String> = user_param.cast();

        unsafe {
            if let Some(v) = vec_ptr.as_mut() {
                if v.len() >= 20 {
                    log::warn!("graphics context error log filled, discarding new logs");
                } else if let gl::DEBUG_TYPE_ERROR
                | gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR
                | gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR
                | gl::DEBUG_TYPE_PORTABILITY
                | gl::DEBUG_TYPE_PERFORMANCE = kind
                {
                    v.push(error_message);
                }
            }
        }
    }
}
