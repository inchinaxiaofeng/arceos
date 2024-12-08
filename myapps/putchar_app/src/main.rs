#![feature(asm_const)]
#![no_std]
#![no_main]

// `const SYS_HELLO: usize = 1;`
const SYS_PUTCHAR: usize = 2;

#[no_mangle]
#[inline(never)]
fn putchar(c: char) {
    let arg0: u8 = c as u8;
    unsafe {
        core::arch::asm!("
            addi    sp, sp, -8
            sd      ra, 0(sp)

            li      a7, 0xffffffc080249000
            li      t0, {abi_num}
            slli    t0, t0, 3
            add     t1, a7, t0
            ld      t1, (t1)
            jalr    t1

            ld      ra, 0(sp)
            addi    sp, sp, 8
            ",
            abi_num = const SYS_PUTCHAR,
            in("a0") arg0,
        )
    }
}

#[no_mangle]
#[inline(never)]
fn puts(s: &str) {
    let mut i: u8 = 1;
    for c in s.chars() {
        putchar(c);
    }
    putchar('\n'); // 打印换行符
}

#[no_mangle]
extern "C" fn _start() -> () {
    puts("Hello world!");
    return;
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
