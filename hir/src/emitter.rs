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

impl Default for Emitter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct LabelAllocator {
    count: usize,
}

impl LabelAllocator {
    #[inline]
    pub const fn new() -> Self {
        Self { count: 0 }
    }

    #[inline]
    pub fn alloc(&mut self) -> Label {
        let t = Label::new(self.count);
        self.count += 1;
        t
    }
}

impl Default for LabelAllocator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct TempAllocator {
    count: usize,
}

impl TempAllocator {
    #[inline]
    pub const fn new() -> Self {
        Self { count: 0 }
    }

    #[inline]
    pub fn alloc(&mut self) -> Temp {
        let t = Temp::new(self.count);
        self.count += 1;
        t
    }
}

impl Default for TempAllocator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
