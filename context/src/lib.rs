#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]

pub mod error;
pub mod opengl;

pub mod buffer;
pub mod render_target;
pub mod shader;
pub mod vertex_layout;

use cac_core::{gen_vec::Handle, math::URect};

pub use buffer::Buffer;
pub use render_target::RenderTarget;
pub use vertex_layout::VertexLayout;

pub use error::Error;

pub mod handle {
    pub struct Buffer;
    pub struct VertexLayout;
    pub struct Shader;
    pub struct Stage;
    pub struct RenderTarget;
}

pub type BufferHandle = Handle<handle::Buffer>;
pub type VertexLayoutHandle = Handle<handle::VertexLayout>;
pub type StageHandle = Handle<handle::Stage>;
pub type ShaderHandle = Handle<handle::Shader>;
pub type RenderTargetHandle = Handle<handle::RenderTarget>;

pub trait Context {
    type Buffer: buffer::Native;
    type Layout: vertex_layout::Native;
    type Shader: shader::Native;
    type Stage;
    type RenderTarget: render_target::Native;

    fn reset(&mut self);
    fn update(&mut self);

    fn poll_errors(&mut self) -> Option<Vec<String>>;
    fn viewport(&self) -> URect;

    /// Invokes a drawcall, binding the shader, layout and rendetarget
    ///
    /// # Errors
    /// `Error::ResourceNotFound`: When the handles are invalid and are not pointing to actual resources
    /// `Error::ConversionError`: When start or count can't be converted into the native graphics API
    /// value, like i32, without wrapping or overflowing.
    fn draw(
        &mut self,
        target: RenderTargetHandle,
        primitive: Primitive,
        shader: ShaderHandle,
        layout: VertexLayoutHandle,
        start: usize,
        count: usize,
    ) -> Result<(), Error>;

    /// Creates a render target, a surface to draw onto
    ///
    /// # Errors
    /// Depends on the native implementation.
    ///
    /// `Error::ConversionError`: When the viewport inside the passed struct can't be converted
    /// into the required graphics value without wrapping or overflowing.
    fn create_render_target(
        &mut self,
        render_target: RenderTarget,
    ) -> Result<RenderTargetHandle, Error>;

    fn render_target(&self, handle: RenderTargetHandle) -> Option<&Self::RenderTarget>;
    fn render_target_mut(&mut self, handle: RenderTargetHandle) -> Option<&mut Self::RenderTarget>;

    /// Creates a buffer, data that is stored on the graphics context.
    /// It doesn't neccessarily mean that the data is stored on the GPU, but is dependent on the
    /// actual graphics implementation.
    ///
    /// # Errors
    /// Depends on the native implementation.
    ///
    /// `Error::ConversionError`: When the length of the containing data can't be converted into
    /// the native type without wrapping or overflowing.
    fn create_buffer<T: buffer::FlatData>(
        &mut self,
        buffer: &Buffer<T>,
    ) -> Result<BufferHandle, Error>;
    fn buffer(&self, handle: BufferHandle) -> Option<&Self::Buffer>;
    fn buffer_mut(&mut self, handle: BufferHandle) -> Option<&mut Self::Buffer>;

    /// Creates the vertex layout
    ///
    ///
    /// # Errors
    /// Depends on the native implementation.
    ///
    /// `Error::ConversionError`: When the attributes can't be converted into the native types
    /// without wrapping or overflowing.
    ///
    /// `Error::ResourceNotFound`: When a buffer handle is invalid and not pointing to a graphics
    /// object.
    fn create_layout(&mut self, layout: &VertexLayout) -> Result<VertexLayoutHandle, Error>;
    fn layout(&self, handle: VertexLayoutHandle) -> Option<&Self::Layout>;
    fn layout_mut(&mut self, handle: VertexLayoutHandle) -> Option<&mut Self::Layout>;

    /// Creates a new shader stage
    ///
    /// # Errors
    /// Depends on the native implementation.
    ///
    /// `Error::ConversionError`: When a type can't be converted into the native types.
    /// `FailedToCompileShader`: When the shader source contains a compilation error or is not a
    /// valid `CString`,
    fn create_stage(&mut self, shader: shader::Stage) -> Result<StageHandle, Error>;
    fn stage(&self, handle: StageHandle) -> Option<&Self::Stage>;

    /// Creates a new shader program
    ///
    /// # Errors
    /// Depends on the native implementation.
    ///
    /// `Error::ConversionError`: When a type can't be converted into the native types.
    /// `FailedToLinkShader`: When the shader stages are invalid or a linking error occurs.
    /// `ResourceNotFound`: When a shader stage handle is invalid and doesn't point to a graphics
    /// object.
    fn create_shader(&mut self, shader: shader::Shader) -> Result<ShaderHandle, Error>;
    fn shader(&self, handle: ShaderHandle) -> Option<&Self::Shader>;
    fn shader_mut(&mut self, handle: ShaderHandle) -> Option<&mut Self::Shader>;

    fn layout_mut_and_buffers(
        &mut self,
        handle: VertexLayoutHandle,
        buffer_handles: &[BufferHandle],
    ) -> (Option<&mut Self::Layout>, Vec<Option<&Self::Buffer>>);
}

pub enum Primitive {
    Triangles,
    TriangleStrip,
}
