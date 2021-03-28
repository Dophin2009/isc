pub use hir;

mod program;

use ast::{Path, Program};
use hir::emitter::Emitter;

use std::collections::BTreeMap;

// Trait implemented by nodes to produce the corresponding IR chunk.
pub(crate) trait Emit {
    /// Output type for passing back useful values.
    type Output;
    type Error;

    fn emit(val: &Self, codegen: &mut CodegenState) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug)]
pub struct Codegen {}

impl Codegen {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }

    #[inline]
    pub fn emit(&self, program: &Program) -> Result<hir::Program, ()> {
        let mut state = CodegenState::new();
        state.emit(program)
    }
}

#[derive(Debug)]
pub(crate) struct CodegenState {
    pub ir_emitter: Emitter,
    pub type_tracker: TypeTracker,
}

impl CodegenState {
    #[inline]
    pub fn new() -> Self {
        Self {
            ir_emitter: Emitter::new(),
            type_tracker: TypeTracker::new(),
        }
    }

    #[inline]
    pub fn emit<T>(&mut self, val: &T) -> Result<T::Output, T::Error>
    where
        T: Emit,
    {
        T::emit(val, self)
    }
}

#[derive(Debug)]
pub(crate) struct TypeTracker {
    pub types: BTreeMap<Path, TypeData>,
}

impl TypeTracker {
    #[inline]
    pub fn new() -> Self {
        Self {
            types: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn register_struct(&mut self, s: ast::Struct) {
        self.types.insert(TypeData::Struct(s))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeData {
    Struct(ast::Struct),
}

impl TypeData {
    #[inline]
    pub fn size(&self) -> usize {
        match self {
            Self::Struct(s) => s.fields.items.iter().map(|field| ast_type_size(&field.ty)),
        }
    }
}

#[inline]
fn ast_type_size(ty: &ast::Type) -> usize {
    match ty {
        ast::Type::Array(array_type) => {}
    }
}
