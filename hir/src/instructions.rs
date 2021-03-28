use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Program {
    pub start: Start,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Start {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Function {
    pub name: FunctionName,
    pub params: Vec<Rc<LValue>>,
    pub body: Vec<Instruction>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LValue {
    Temp(Temp),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RValue {
    Temp(Temp),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LabelMark {
    pub idx: usize,
}

impl LabelMark {
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
    Ref(Ref),
    Deref(Deref),

    Indexing(Indexing),
    Call(Call),

    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    Equ(Equ),
    Nequ(Nequ),
    Gt(Gt),
    GtEqu(LtEqu),
    Lt(Lt),
    LtEqu(LtEqu),

    If(If),

    /// Jump to some label.
    Jump(Jump),
    /// Indicate the location of some label.
    Label(Label),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ref {
    pub val: Rc<RValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deref {
    pub ptr: Rc<RValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Indexing {
    pub lhs: Rc<LValue>,
    pub array: Rc<RValue>,
    pub idx: Rc<RValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Call {
    pub function: FunctionName,
    pub args: Vec<Rc<RValue>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionName {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jump {
    pub label: Label,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label {
    pub label: LabelMark,
}

// TODO: Devise a more strongly typed solution?
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct If {
    pub condition: Rc<RValue>,
    pub jump: Jump,
}

pub type Add = BinaryOperation<AddOp>;
pub type Sub = BinaryOperation<SubOp>;
pub type Mul = BinaryOperation<MulOp>;
pub type Div = BinaryOperation<DivOp>;
pub type Equ = BinaryOperation<EquOp>;
pub type Nequ = BinaryOperation<NequOp>;
pub type Gt = BinaryOperation<GtOp>;
pub type GtEqu = BinaryOperation<GtEquOp>;
pub type Lt = BinaryOperation<LtOp>;
pub type LtEqu = BinaryOperation<LtEquOp>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryOperation<T>
where
    T: BinaryOperator,
{
    pub lhs: Rc<LValue>,
    pub op: T,
    pub o1: Rc<RValue>,
    pub o2: Rc<RValue>,
}

pub trait BinaryOperator {}

macro_rules! binaryop {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name;

        impl BinaryOperator for $name {}
    };
    ($($name:ident),*) => {
        $(binaryop!($name);)*
    };
}

binaryop! { AddOp, SubOp, MulOp, DivOp, EquOp, NequOp, GtOp, GtEquOp, LtOp, LtEquOp }
