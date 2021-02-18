pub use irr;

pub use irr::{Emitter, Instruction, Temp};

// Trait implemented by nodes to produce the corresponding IR chunk.
pub trait EmitIR {
    /// Output type for passing back useful values.
    type Output;
    type Error;

    fn emit_ir(val: &Self, emitter: &mut Emitter) -> Result<Self::Output, Self::Error>;
}

mod expr;
