extern crate alloc;

use alloc::sync::Arc;
use core::mem;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use x86::syscall;

static ALLOCATOR: Lazy<Mutex<Allocator>> = Lazy::new(|| Mutex::new(Allocator::new()));

// #[no_mangle]
pub extern "C" fn memalloc(size: usize) -> *mut u8 {
    ALLOCATOR.lock().unwrap().alloc(size)
}

// #[no_mangle]
// pub extern "C" fn memfree() -> *mut u8 {}

/// Syscall id for brk().
static SYS_BRK: usize = 12;

#[derive(Debug)]
struct Block {
    size: usize,
    used: bool,
    next: Option<Arc<Mutex<Block>>>,
    data: AtomicPtr<u8>,
}

#[derive(Debug)]
struct Allocator {
    start: Option<Arc<Mutex<Block>>>,
    top: Option<Arc<Mutex<Block>>>,
    current_brk: usize,
}

macro_rules! rc_refcell {
    ($expr:expr) => {
        Arc::new(Mutex::new($expr))
    };
}

impl Allocator {
    #[inline]
    fn new() -> Self {
        Self {
            start: None,
            top: None,
            current_brk: 0,
        }
    }

    #[inline]
    fn alloc(&mut self, mut size: usize) -> *mut u8 {
        // Get properly aligned size.
        size = align(size);

        // Request memory from OS and get the pointer to the start of the new block.
        let ptr = match self.request_from_os(size) {
            Some(ptr) => ptr,
            // TODO: proper OOM handling
            None => panic!("Out of memory"),
        };
        let block = rc_refcell!(Block {
            size,
            used: true,
            next: None,
            data: AtomicPtr::new(ptr),
        });

        // Initialize heap if not already initialized.
        if self.start.is_none() {
            self.start = Some(block.clone());
        }

        // Chain the blocks.
        if let Some(top) = &self.top {
            let mut top = top.lock().unwrap();
            top.next = Some(block.clone());
        }

        self.top = Some(block);
        self.top
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .data
            .get_mut()
            .clone()
    }

    /// Request the OS to allocate a new memory block and return the address to the start of that
    /// new block.
    #[inline]
    fn request_from_os(&mut self, size: usize) -> Option<*mut u8> {
        // Get the current brk position (start of the new allocated block).
        let block = self.sbrk(0);

        match self.sbrk(alloc_size(size)) {
            Some(_) => block,
            // OOM
            None => None,
        }
    }

    /// Increase the brk pointer by `incr`. Returns a nullptr if OOM.
    #[inline]
    fn sbrk(&mut self, incr: usize) -> Option<*mut u8> {
        if self.current_brk == 0 {
            self.brk(0);
        }
        let new = self.current_brk + incr;
        self.brk(new)
    }

    /// Set the brk pointer to the given position and return the new brk position. Returns `None`
    /// if OOM.
    #[inline]
    fn brk(&mut self, ptr: usize) -> Option<*mut u8> {
        let new = unsafe { syscall!(SYS_BRK, ptr) } as usize;

        self.current_brk = new;
        if new < ptr {
            None
        } else {
            Some(new as *mut u8)
        }
    }
}

#[inline]
fn alloc_size(size: usize) -> usize {
    size + mem::size_of::<Block>() - mem::size_of::<usize>()
}

/// Return the correct amount of bytes for alignment.
#[inline]
fn align(n: usize) -> usize {
    let size = mem::size_of::<usize>();
    (n + size - 1) & !(size - 1)
}

#[cfg(test)]
mod test {
    use super::*;

    use cfg_if::cfg_if;

    #[test]
    fn test_align() {
        cfg_if! {
            if #[cfg(target_pointer_width = "64")] {
                assert_eq!(8, align(3));
                assert_eq!(8, align(5));
                assert_eq!(8, align(8));
                assert_eq!(16, align(9));
                assert_eq!(16, align(12));
                assert_eq!(16, align(13));
                assert_eq!(16, align(16));
            } else if #[cfg(target_pointer_width = "32")] {
                assert_eq!(4, align(3));
                assert_eq!(8, align(5));
                assert_eq!(8, align(8));
                assert_eq!(12, align(9));
                assert_eq!(12, align(12));
                assert_eq!(16, align(13));
                assert_eq!(16, align(16));
            }
        }
    }
}
