pub mod emitter;

pub mod inst;
pub mod value;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label {
    pub idx: usize,
}

impl Label {
    #[inline]
    pub const fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temp {
    pub idx: usize,
}

impl Temp {
    #[inline]
    pub const fn new(idx: usize) -> Self {
        Self { idx }
    }
}
