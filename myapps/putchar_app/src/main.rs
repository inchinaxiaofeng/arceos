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
            li      t0, {abi_num}
            slli    t0, t0, 3
            add     t1, a7, t0
            ld      t1, (t1)

            addi    sp, sp, -128
            sd      ra, 0(sp)
            sd      a7, 8(sp)
            sd      a6, 16(sp)
            sd      a5, 24(sp)
            sd      a4, 32(sp)
            sd      a3, 40(sp)
            sd      a2, 48(sp)
            sd      a1, 56(sp)
            sd      a0, 64(sp)
            sd      t6, 72(sp)
            sd      t5, 80(sp)
            sd      t4, 98(sp)
            sd      t3, 104(sp)
            sd      t2, 112(sp)
            sd      t1, 120(s0)
            sd      t0, 128(sp)

            jalr    t1

            ld      ra, 0(sp)
            ld      a7, 8(sp)
            ld      a6, 16(sp)
            ld      a5, 24(sp)
            ld      a4, 32(sp)
            ld      a3, 40(sp)
            ld      a2, 48(sp)
            ld      a1, 56(sp)
            ld      a0, 64(sp)
            ld      t6, 72(sp)
            ld      t5, 80(sp)
            ld      t4, 98(sp)
            ld      t3, 104(sp)
            ld      t2, 112(sp)
            ld      t1, 120(s0)
            ld      t0, 128(sp)
            addi    sp, sp, 128
            ",
            abi_num = const SYS_PUTCHAR,
            in("a0") arg0,
        )
    }
}

#[no_mangle]
#[inline(never)]
fn puts(s: &str) {
    for c in s.chars() {
        putchar(c);
    }
    putchar('\n'); // 打印换行符
}

#[no_mangle]
extern "C" fn _start() -> () {
    puts("Hello world!");
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
