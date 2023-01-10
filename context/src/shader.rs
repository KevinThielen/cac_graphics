use std::fmt::Display;

#[derive(Copy, Clone)]
pub enum Kind {
    Vertex,
    Fragment,
}

#[derive(Copy, Clone)]
pub struct Stage<'a> {
    pub kind: Kind,
    pub sources: &'a [&'a str],
}

impl<'a> Stage<'a> {
    #[must_use]
    pub const fn new_vertex(sources: &'a [&'a str]) -> Self {
        Self {
            kind: Kind::Vertex,
            sources,
        }
    }
    #[must_use]
    pub const fn new_fragment(sources: &'a [&'a str]) -> Self {
        Self {
            kind: Kind::Fragment,
            sources,
        }
    }
}

pub trait Native {}

#[derive(Clone, Copy)]
pub struct Shader<'a> {
    pub stages: &'a [crate::StageHandle],
    pub stage_sources: &'a [Stage<'a>],
}

impl<'a> Shader<'a> {
    #[must_use]
    pub const fn with_stages(stages: &'a [Stage<'a>]) -> Self {
        Self {
            stages: &[],
            stage_sources: stages,
        }
    }

    #[must_use]
    pub const fn with_handles(handles: &'a [crate::StageHandle]) -> Self {
        Self {
            stages: handles,
            stage_sources: &[],
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertex => write!(f, "vertex"),
            Self::Fragment => write!(f, "fragment"),
        }
    }
}
