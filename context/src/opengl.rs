use crate::{error::Error, gl43_core as gl, render_target};

pub trait GLContext {
    fn swap_buffers(&mut self);
    fn get_proc_address(&mut self, name: &'static str) -> *const std::ffi::c_void;
}

pub struct Context<C: GLContext> {
    gl_context: C,
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
        })
    }

    pub fn raw_context(&mut self) -> &mut C {
        &mut self.gl_context
    }
}

impl<C: GLContext> crate::Context for Context<C> {
    fn update(&mut self) {
        self.gl_context.swap_buffers();
    }
}

impl<C: GLContext> render_target::Context for Context<C> {
    fn clear(&mut self, target: &render_target::RenderTarget) {
        if let Some(color) = target.clear_color {
            let (r, g, b, a) = color.as_rgba();
            unsafe {
                gl::ClearColor(r, g, b, a);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
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
