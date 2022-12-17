#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod buffer;
pub mod error;
pub mod opengl;
pub mod render_target;

pub trait Context: render_target::Context + buffer::Context {
    fn update(&mut self);
}
