pub use crate::token::Type as PrimitiveType;

#[derive(Clone, Debug)]
pub struct Spanned<T>(pub T, pub Span);

#[derive(Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub enum Item {
    Struct(Struct),
    Function(Function),
}

#[derive(Clone, Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Vec<StructField>,
    pub functions: Vec<StructFunction>,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub vis: Visibility,
    pub name: Ident,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct StructFunction {
    pub vis: Visibility,
    pub name: Ident,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub is_method: bool,
    pub body: Body,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub vis: Visibility,
    pub name: Ident,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub body: Body,
}

#[derive(Clone, Debug)]
pub struct FunctionParam {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Body {}

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Path(Path),
    None,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub segs: Vec<Ident>,
}
