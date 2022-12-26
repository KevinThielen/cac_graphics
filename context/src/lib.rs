#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod error;
pub mod opengl;

pub mod buffer;
pub mod render_target;
pub mod shader;
pub mod vertex_layout;

pub use error::Error;
pub trait Context: render_target::Context + buffer::Context + vertex_layout::Context {
    fn update(&mut self);
    fn draw(
        &mut self,
        target: &render_target::RenderTarget,
        primitive: Primitive,
        shader: shader::Shader,
        layout: vertex_layout::VertexLayout,
        start: usize,
        count: usize,
    ) -> Result<(), Error>;
}

pub trait ResourceStorage<R, H> {
    fn insert(&mut self, resource: R);
    fn get(&self, handle: H) -> Option<&R>;
    fn get_mut(&self, handle: H) -> Option<&mut R>;
    fn remove(&mut self, handle: H) -> Option<R>;
}

pub trait Resource<H> {
    type NativeResource;
    type Storage: ResourceStorage<Self::NativeResource, H>;

    fn storage(&self) -> &Self::Storage;
    fn storage_mut(&self) -> &mut Self::Storage;
}

pub enum Primitive {
    Triangles,
    TriangleStrip,
}
