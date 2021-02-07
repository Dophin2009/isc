#![no_std]
#![feature(lang_items)]
#![feature(const_mut_refs)]

mod alloc;

use core::panic::PanicInfo;

use alloc::{Allocator, Locked};
use spin::Once;

static ALLOCATOR: Once<Locked<Allocator>> = Once::initialized(Locked::new(Allocator::new()));

#[no_mangle]
pub extern "C" fn memalloc(size: usize) -> *mut u8 {
    ALLOCATOR.get().unwrap().alloc(size)
}

#[no_mangle]
pub extern "C" fn memfree(ptr: *mut u8) {
    ALLOCATOR.get().unwrap().dealloc(ptr);
}

#[cfg(not(test))]
#[lang = "eh_personality"]
fn eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
