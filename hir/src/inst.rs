use crate::value::{SymbolRef, Value};

#[derive(Clone, Debug)]
pub enum Instruction<B>
where
    B: BinaryOp,
{
    Copy(CopyInstruction),
    CopyFromRef(CopyFromRefInstruction),
    CopyFromDeref(CopyFromDerefInstruction),
    CopyToDeref(CopyToDerefInstruction),

    CopyFromArray(CopyFromArrayInstruction),
    CopyToArray(CopyToArrayInstruction),

    Indexing(IndexingInstruction),
    Call(CallInstruction),

    UnaryOp(UnaryOpInstruction),
    BinaryOp(BinaryOpInstruction<B>),

    If(IfInstruction),

    /// Jump to some label.
    Jump(JumpInstruction),
    /// Indicate the location of some label.
    Label(LabelInstruction),
}

#[derive(Clone, Debug)]
pub struct IndexingInstruction {
    pub lhs: LValue,
    pub array: RValue,
    pub idx: RValue,
}

#[derive(Clone, Debug)]
pub struct CallInstruction {
    pub function: RValue,
}

#[derive(Clone, Debug)]
pub struct JumpInstruction {
    pub label: Label,
}

#[derive(Clone, Debug)]
pub struct LabelInstruction {
    pub label: Label,
}

// TODO: Devise a more strongly typed solution?
#[derive(Clone, Debug)]
pub struct IfInstruction {
    pub condition: RValue,
    pub jump: JumpInstruction,
}

#[derive(Clone, Debug)]
pub struct UnaryOpInstruction<T>
where
    T: UnaryOp,
{
    pub lhs: LValue,
    pub op: T,
    pub rhs: RValue,
}

pub trait UnaryOp {}

#[derive(Clone, Debug)]
pub struct BinaryOpInstruction<T>
where
    T: BinaryOp,
{
    pub lhs: LValue,
    pub op: T,
    pub o1: RValue,
    pub o2: RValue,
}

pub trait BinaryOp {}

pub type AddInstruction = BinaryOpInstruction<Add>;
pub type SubInstruction = BinaryOpInstruction<Sub>;
pub type MulInstruction = BinaryOpInstruction<Mul>;
pub type DivInstruction = BinaryOpInstruction<Div>;
pub type EquInstruction = BinaryOpInstruction<Equ>;
pub type NequInstruction = BinaryOpInstruction<Nequ>;
pub type GtInstruction = BinaryOpInstruction<Gt>;
pub type GtEquInstruction = BinaryOpInstruction<GtEqu>;
pub type LtInstruction = BinaryOpInstruction<Lt>;
pub type LtEquInstruction = BinaryOpInstruction<LtEqu>;

macro_rules! binaryop {
    ($name:ident) => {
        #[derive(Clone, Debug)]
        pub struct $name;

        impl BinaryOp for $name {}
    };
    ($($name:ident),*) => {
        $(binaryop!($name);)*
    };
}

binaryop! { Add, Sub, Mul, Div, Equ, Nequ, Gt, GtEqu, Lt, LtEqu }
