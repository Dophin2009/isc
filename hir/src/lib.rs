pub mod emitter;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LValue {
    Temp(Temp),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RValue {
    Temp(Temp),
}

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instruction {
    Indexing(IndexingInstruction),
    Call(CallInstruction),

    Add(AddInstruction),
    Sub(SubInstruction),
    Mul(MulInstruction),
    Div(DivInstruction),
    Equ(EquInstruction),
    Nequ(NequInstruction),
    Gt(GtInstruction),
    GtEqu(LtEquInstruction),
    Lt(LtInstruction),
    LtEqu(LtEquInstruction),

    If(IfInstruction),

    /// Jump to some label.
    Jump(JumpInstruction),
    /// Indicate the location of some label.
    Label(LabelInstruction),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexingInstruction {
    pub lhs: LValue,
    pub array: RValue,
    pub idx: RValue,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallInstruction {
    pub function: RValue,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JumpInstruction {
    pub label: Label,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LabelInstruction {
    pub label: Label,
}

// TODO: Devise a more strongly typed solution?
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IfInstruction {
    pub condition: RValue,
    pub jump: JumpInstruction,
}

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

macro_rules! binaryop {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name;

        impl BinaryOp for $name {}
    };
    ($($name:ident),*) => {
        $(binaryop!($name);)*
    };
}

binaryop! { Add, Sub, Mul, Div, Equ, Nequ, Gt, GtEqu, Lt, LtEqu }
