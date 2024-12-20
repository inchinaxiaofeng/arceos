#![feature(asm_const)]
#![no_std]
#![no_main]

use axlog::debug;
use axstd::{print, println, process::exit};
use core::{
    cmp::min,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use elf::{endian::LittleEndian, ElfBytes};

/// `bin`的开始位置
const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const MAX_APP_SIZE: usize = 0x100000;
const RUN_START: usize = 0xffff_ffc0_8010_0000;

pub fn load_elf() -> u64 {
    let elf_size = unsafe { *(PLASH_START as *const usize) };
    debug!("ELF size: 0x{:x}", elf_size);
    let elf_slice = unsafe { from_raw_parts((PLASH_START) as *const u8, elf_size) };
    let elf: ElfBytes<'_, LittleEndian> =
        ElfBytes::<LittleEndian>::minimal_parse(elf_slice).expect("Failed to parse ELF");
    let elf_hdr = elf.ehdr;

    let run_code = unsafe { from_raw_parts_mut(RUN_START as *mut u8, MAX_APP_SIZE) };

    load_exec(&elf, elf_slice, run_code);
    let entry = elf_hdr.e_entry;
    debug!("Entry: 0x{:x}", entry);
    return entry;
}

fn load_exec(elf: &ElfBytes<LittleEndian>, elf_slice: &[u8], run_code: &mut [u8]) {
    let text_shdr = elf
        .section_header_by_name(".text")
        .expect("section table should be parseable")
        .expect("elf should have a .text section");
    let text_slice = elf_slice
        .get(text_shdr.sh_offset as usize..)
        .expect("text section should be in bounds");
    let copy_size = min(run_code.len(), text_slice.len());
    run_code[..copy_size].copy_from_slice(&text_slice[..copy_size]);
}

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe {
        ABI_TABLE[num] = handle;
    }
}

fn abi_hello() {
    print!("\x1b[34m");
    println!("[ABI:Hello] Hello, Apps!");
    print!("\x1b[0m");
}

fn abi_putchar(c: char) {
    print!("\x1b[34m");
    print!("{c}");
    print!("\x1b[0m");
}

fn abi_terminate() {
    print!("\x1b[34m");
    println!("Bye");
    print!("\x1b[0m");

    exit(0);
}

unsafe fn bye() -> () {
    core::arch::asm!("
          li      t0, {abi_num}
          slli    t0, t0, 3
          la      t1, {abi_table}
          add     t1, t1, t0
          ld      t1, (t1)
          jalr    t1
          li      t2, {run_start}
          jalr    t2
          j       .",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
        //abi_num = const SYS_HELLO,
        abi_num = const SYS_TERMINATE,
    )
}

#[no_mangle]
fn main() {
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);

    let entry = load_elf();

    unsafe {
        core::arch::asm!("
                la      a7, {abi_table}

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

                jalr    t2

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
            abi_table = sym ABI_TABLE,
            in("t2") entry,
        )
    }

    unsafe {
        bye();
    }
}
