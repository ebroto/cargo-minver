#![no_std]
#![feature(lang_items)]

use core::panic::PanicInfo;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

// NOTE: Since the panic_fmt lang item was removed, there is no
// way to test a no_std crate without having this attribute.
#[panic_handler]
fn lets_panic(_pi: &PanicInfo) -> ! {
    loop {}
}

fn main() {}
