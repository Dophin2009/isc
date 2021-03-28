use crate::instructions::{Label, Temp};

use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Emitter {
    temp_allocator: Allocator<Temp>,
    label_allocator: Allocator<Label>,
}

impl Emitter {
    #[inline]
    pub fn new() -> Self {
        Self {
            temp_allocator: Allocator::new(),
            label_allocator: Allocator::new(),
        }
    }

    #[inline]
    pub fn alloc_temp(&mut self) -> Temp {
        self.temp_allocator.alloc()
    }

    #[inline]
    pub fn alloc_label(&mut self) -> Label {
        self.label_allocator.alloc()
    }
}

impl Default for Emitter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

pub trait Alloc {
    fn alloc(idx: usize) -> Self;
}

#[derive(Debug, Clone)]
pub struct Allocator<T>
where
    T: Alloc + Sized,
{
    count: usize,

    _phantom: PhantomData<T>,
}

impl<T> Allocator<T>
where
    T: Alloc + Sized,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            count: 0,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn alloc(&mut self) -> T {
        let t = T::alloc(self.count);
        self.count += 1;
        t
    }
}

impl<T> Default for Allocator<T>
where
    T: Alloc,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Alloc for Temp {
    #[inline]
    fn alloc(idx: usize) -> Self {
        Self { idx }
    }
}

impl Alloc for Label {
    #[inline]
    fn alloc(idx: usize) -> Self {
        Self { idx }
    }
}
