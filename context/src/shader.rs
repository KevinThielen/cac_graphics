use std::fmt::Display;

use crate::error::Error;

#[derive(Copy, Clone)]
pub enum Kind {
    Vertex,
    Fragment,
}

#[derive(Copy, Clone)]
pub struct Stage {
    pub(crate) handle: usize,
    pub kind: Kind,
}

impl Stage {
    pub fn new_vertex<C: Context>(ctx: &mut C, sources: &[&str]) -> Result<Self, Error> {
        ctx.new_stage(Kind::Vertex, sources)
    }
    pub fn new_fragment<C: Context>(ctx: &mut C, sources: &[&str]) -> Result<Self, Error> {
        ctx.new_stage(Kind::Fragment, sources)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Shader {
    pub(crate) handle: usize,
}

impl Shader {
    pub fn with_sources<C: Context>(
        ctx: &mut C,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Self, Error> {
        ctx.with_sources(&[vertex_shader], &[fragment_shader])
    }

    pub fn new<C: Context>(ctx: &mut C, stages: &[Stage]) -> Result<Shader, Error> {
        ctx.new(stages)
    }
}

pub trait Context {
    fn new(&mut self, stages: &[Stage]) -> Result<Shader, Error>;
    fn with_sources(
        &mut self,
        vertex_shader: &[&str],
        fragment_shader: &[&str],
    ) -> Result<Shader, Error>;
    fn new_stage(&mut self, kind: Kind, sources: &[&str]) -> Result<Stage, Error>;
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Vertex => write!(f, "vertex"),
            Kind::Fragment => write!(f, "fragment"),
        }
    }
}
