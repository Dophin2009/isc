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
    name: Ident,
    params: Vec<FunctionParam>,
    return_type: Type,
    is_method: bool,
}

#[derive(Clone, Debug)]
pub struct Function {
    visibility: Visibility,
    name: Ident,
    params: Vec<FunctionParam>,
}

#[derive(Clone, Debug)]
pub struct FunctionParam {
    name: Ident,
    ty: Type,
}

#[derive(Clone, Debug)]
pub struct Type {
    name: String,
}

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
}
