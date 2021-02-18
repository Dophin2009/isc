#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,

    /// Jump to some label.
    Jump,
    /// Indicate the location of some label.
    Label,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {}

#[derive(Clone, Debug)]
pub struct Emitter {
    instructions: Vec<Instruction>,

    temp_allocator: TempAllocator,
}

impl Emitter {
    pub const fn new() -> Self {
        Self {
            instructions: vec![],
            temp_allocator: TempAllocator::new(),
        }
    }

    pub fn emit_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn alloc_temp(&mut self) -> Temp {
        self.temp_allocator.alloc()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temp {
    pub idx: usize,
}

impl Temp {
    pub const fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Clone, Debug)]
pub struct TempAllocator {
    count: usize,
}

impl TempAllocator {
    pub const fn new() -> Self {
        Self { count: 0 }
    }

    pub fn alloc(&mut self) -> Temp {
        let t = Temp::new(self.count);
        self.count += 1;
        t
    }
}
