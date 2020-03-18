#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn lets_panic(_pi: &PanicInfo) -> ! {
    loop {}
}

fn main() {}
