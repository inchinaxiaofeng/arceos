use core::panic;
use core::{
    cmp::min,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use axlog::debug;

use axstd::println;

use elf::{
    abi::{ET_DYN, ET_EXEC},
    endian::LittleEndian,
    ElfBytes,
};

/// `bin`的开始位置
const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const MAX_APP_SIZE: usize = 0x100000;
pub const RUN_START: usize = 0xffff_ffc0_8010_0000;

pub fn load_elf() -> u64 {
    let elf_size = unsafe { *(PLASH_START as *const usize) };
    let elf_slice = unsafe { from_raw_parts((PLASH_START) as *const u8, elf_size) };
    let run_code = unsafe { from_raw_parts_mut(RUN_START as *mut u8, MAX_APP_SIZE) };

    let elf: ElfBytes<'_, LittleEndian> =
        ElfBytes::<LittleEndian>::minimal_parse(elf_slice).expect("Failed to parse ELF");
    let elf_hdr = elf.ehdr;

    load_exec(&elf, elf_slice, run_code);

    let entry: u64;

    if elf_hdr.e_type == ET_EXEC {
        // Static and position independent executable
        load_exec(&elf, elf_slice, run_code);
        entry = elf_hdr.e_entry;
    } else if elf_hdr.e_type == ET_DYN {
        load_dyn(&elf, elf_slice, run_code);
        entry = RUN_START as u64 + elf_hdr.e_entry;
    } else {
        panic!("Invalid ELF type");
    }

    // FIXME: 可能是出于编译的原因，在这里不进行打印或者任何一种使用的情况下，就不能正确执行。
    println!("Entry: 0x{:x}", entry);
    println!("ELF size: 0x{:x}", elf_size);
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

fn load_dyn(elf: &ElfBytes<LittleEndian>, elf_slice: &[u8], run_code: &mut [u8]) {
    let phdrs = elf.segments().expect("Failed to parse program headers");
    for phdr in phdrs {
        if phdr.p_type != elf::abi::PT_LOAD {
            continue;
        }
        load_segment(
            run_code,
            elf_slice,
            phdr.p_vaddr as usize,
            phdr.p_offset as usize,
            phdr.p_filesz as usize,
            phdr.p_memsz as usize,
        );
    }
    modify_plt(elf);
}

fn load_segment(
    run_code: &mut [u8],
    elf_slice: &[u8],
    p_vaddr: usize,
    p_offset: usize,
    p_filesz: usize,
    p_memsz: usize,
) {
    // Copy the segment into the executable zone
    // If `memsz` is larger than `filesz`, zero out the rest
    let run_code_offset = p_vaddr;
    run_code[run_code_offset..run_code_offset + p_filesz]
        .copy_from_slice(&elf_slice[p_offset..p_offset + p_filesz]);
    if p_memsz > p_filesz {
        let zero_size = min(run_code.len() - p_filesz, p_memsz - p_filesz);
        run_code[run_code_offset + p_filesz..run_code_offset + p_filesz + zero_size].fill(0);
    }
}

fn modify_plt(elf: &ElfBytes<LittleEndian>) {
    let (dynsym_table, dynstr_table) = elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");
    let rela_shdr = elf
        .section_header_by_name(".rela.plt")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.plt section");
    let relas = elf
        .section_data_as_relas(&rela_shdr)
        .expect("Failed to parse .rela.dyn section");

    for rela in relas {
        // Get the `r_sym'th` symbol from the dynamic symbol table
        let sym = dynsym_table
            .get(rela.r_sym as usize)
            .expect("Failed to get symbol");
        let rela_name = dynstr_table
            .get(sym.st_name as usize)
            .expect("Failed to get symbol name");
        let func =
            super::abi::AbiFunction::from_name(rela_name).expect("Failed to find abi function");
        unsafe {
            *((RUN_START as u64 + rela.r_offset) as *mut usize) = func.addr();
            debug!(
                "{} at : 0x{:x}",
                rela_name,
                *((RUN_START as u64 + rela.r_offset) as *const usize)
            );
        }
    }
}
