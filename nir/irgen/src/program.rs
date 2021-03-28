use crate::hir::{Instruction, Start};
use crate::{CodegenState, Emit};

use ast::{Block, Function, Item};

impl Emit for ast::Program {
    type Output = hir::Program;
    type Error = ();

    #[inline]
    fn emit(val: &Self, codegen: &mut CodegenState) -> Result<Self::Output, Self::Error> {
        // Load all structs type information.
        for s in val.items.into_iter().filter_map(|item| match item {
            Item::Struct(s) => Some(s),
            _ => None,
        }) {
            codegen.type_tracker.register_struct(s);
        }

        let functions: Vec<_> = val
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(f) => Some(f),
                _ => None,
            })
            .collect();

        // TODO: error handling?
        let start_function = functions
            .iter()
            .find(|f| f.name.as_str() == "main")
            .unwrap();
        let start_instructions = Block::emit(start_function.body, codegen)?;
        let start = Start {
            instructions: start_instructions,
        };

        let functions = functions
            .into_iter()
            .filter(|f| f.name.as_str() != "main")
            .map(|f| Function::emit(f, codegen))
            .collect()?;

        Ok(hir::Program { start, functions })
    }
}

impl Emit for Function {
    type Output = ();
    type Error = ();

    #[inline]
    fn emit(val: &Self, codegen: &mut CodegenState) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}
