use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type SymbolRef = Rc<Symbol>;

#[derive(Clone, Debug)]
pub enum Symbol {
    Var(Var),
    Type(Type),
    Func(FnRef),
}

#[derive(Clone, Debug)]
pub struct Var {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct FnRef {
    pub name: String,
    pub scope: Rc<Scope>,
    pub param: Vec<SymbolRef>,
    pub ret: Type,
}

#[derive(Clone, Debug)]
pub enum Value {
    Var(SymbolRef),
    Const(Const),
}

#[derive(Clone, Debug)]
pub enum Const {
    Bool(bool),
}

#[derive(Clone, Debug)]
pub enum Type {
    Void,
    Fn(FnType),
}

#[derive(Clone, Debug)]
pub struct FnType {
    pub param: Vec<Type>,
    pub ret: Type,
}

#[derive(Clone, Debug)]
pub struct Ptr {
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Array {
    pub ty: Type,
    pub len: usize,
}

#[derive(Clone, Debug)]
pub struct Scope {
    map: RefCell<HashMap<String, SymbolRef>>,
}

impl Scope {
    /// Create a new scope.
    pub fn new() -> Scope {
        Scope {
            map: RefCell::new(HashMap::new()),
        }
    }

    /// Add a symbol to the scope, and return if this symbol was successfully added.
    pub fn insert(&self, sym: SymbolRef) -> bool {
        let id = sym.name();
        self.map.borrow_mut().insert(id.to_string(), sym).is_none()
    }

    /// Append a collection of symbols to Scope
    pub fn append<I>(&self, iter: I)
    where
        I: Iterator<Item = SymbolRef>,
    {
        iter.for_each(|sym| {
            self.insert(sym);
        })
    }

    /// Lookup a symbol with given `id`.
    pub fn find(&self, id: &str) -> Option<SymbolRef> {
        self.map.borrow_mut().get(id).cloned()
    }

    /// Remove symbol with `id` from scope.
    pub fn remove(&self, id: &str) {
        self.map.borrow_mut().remove(id);
    }

    /// Clear all the symbols in the scope
    pub fn clear(&self) {
        self.map.borrow_mut().clear()
    }

    /// Return vector containing all the symbols in the scope.
    pub fn collect(&self) -> Vec<SymbolRef> {
        self.map.borrow().values().cloned().collect()
    }

    /// Run the given function on each symbol in this scope
    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(SymbolRef),
    {
        self.map.borrow().values().cloned().for_each(f)
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
