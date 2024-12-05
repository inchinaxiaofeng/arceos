#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
unsafe extern "C" fn _start() -> () {
    return;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
