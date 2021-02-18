use crate::{EmitIR, Temp};

use ast::Expr;

impl EmitIR for Expr {
    type Output = (Temp,);
    type Error = ();

    fn emit_ir(val: &Self, emitter: &mut irr::Emitter) -> Result<Self::Output, Self::Error> {
        let output = match val {
            Expr::Var(ident) => {
            }
            Expr::Literal(_) => {}
            Expr::ArrayLiteral(_) => {}
            Expr::FunctionCall(_) => {}
            Expr::BinOp(_) => {}
            Expr::UnaryOp(_) => {}
            Expr::ArrayIndex(_) => {}
        };

        Ok(output)
    }
}
