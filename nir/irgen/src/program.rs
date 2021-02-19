use crate::{EmitIR, Emitter, Temp};

use ast::Program;

impl EmitIR for Program {
    type Output = ();
    type Error = ();

    #[inline]
    fn emit_ir(val: &Self, emitter: &mut Emitter) -> Result<Self::Output, Self::Error> {
        for item in &val.items {
            

        }
    }
}
