use spin::{Mutex, MutexGuard};

/// Syscall id for brk().
static SYS_BRK: usize = 12;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Wrapper around [`spin::Mutex`].
pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}

pub struct Block {
    next: Option<&'static mut Block>,
}

impl Block {
    pub const fn new() -> Self {
        Self { next: None }
    }
}

pub struct BrkState {
    current: usize,
}

impl BrkState {
    pub const fn new() -> Self {
        Self { current: 0 }
    }
}

pub struct Allocator {
    /// Bins of free blocks available to allocate.
    bins: [Option<&'static mut Block>; BLOCK_SIZES.len()],
    brk_state: BrkState,
}

impl Allocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut Block> = None;
        Self {
            bins: [EMPTY; BLOCK_SIZES.len()],
            brk_state: BrkState::new(),
        }
    }

    pub fn fallback_alloc(&mut self, size: usize) -> *mut u8 {
        0 as *mut u8
    }

    pub const fn current_brk(&self) -> usize {
        self.brk_state.current
    }

    /// Increase the brk pointer by `incr`. Returns a nullptr if OOM.
    #[inline]
    pub unsafe fn sbrk(&mut self, incr: usize) -> Result<usize, ()> {
        let old_brk = self.current_brk();
        if old_brk == 0 {
            self.brk(0)?;
        }

        let new_brk = old_brk + incr;
        self.brk(new_brk)
    }

    /// Set the brk pointer to the given position and return the new brk position.
    #[inline]
    pub unsafe fn brk(&mut self, ptr: usize) -> Result<usize, ()> {
        let new = x86::syscall!(SYS_BRK, ptr) as usize;

        self.brk_state.current = new;
        if new < ptr {
            Err(())
        } else {
            Ok(new)
        }
    }
}

impl Locked<Allocator> {
    pub fn alloc(&self, size: usize) -> *mut u8 {
        let mut allocator = self.lock();

        match bin_index(size) {
            Some(index) => match allocator.bins[index].take() {
                Some(block) => {
                    allocator.bins[index] = block.next.take();
                    block as *mut Block as *mut u8
                }
                // If no block exists in the list, allocate a new one.
                None => {
                    let block_size = BLOCK_SIZES[index];
                    allocator.fallback_alloc(block_size)
                }
            },
            None => allocator.fallback_alloc(size),
        }
    }

    pub fn dealloc(&self, ptr: *mut u8) {}
}

fn bin_index(size: usize) -> Option<usize> {
    BLOCK_SIZES.iter().position(|&s| s >= size)
}

/// Return the correct amount of bytes for alignment.
#[inline]
fn align(n: usize, align: usize) -> usize {
    (n + align - 1) & !(align - 1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(8, align(3, 8));
        assert_eq!(8, align(5, 8));
        assert_eq!(8, align(8, 8));
        assert_eq!(16, align(9, 8));
        assert_eq!(16, align(12, 8));
        assert_eq!(16, align(13, 8));
        assert_eq!(16, align(16, 8));
    }
}
