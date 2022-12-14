#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod error;
pub mod opengl;
pub mod render_target;

pub mod gl43_core;

pub trait Context {
    fn update(&mut self);
}
