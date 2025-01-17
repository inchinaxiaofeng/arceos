use core::{
    cmp::min,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use axstd::println;

use axlog::debug;

use elf::{abi::PT_LOAD, endian::LittleEndian, ElfBytes};

use crate::{
    abi::ABI_TABLE,
    elf::{verify_elf_header, LoadError},
};

/// `bin`的开始位置
const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const MAX_APP_SIZE: usize = 0x10_0000;
pub const APP_START: usize = 0xffff_ffc0_8010_0000;
const MAX_LIB_SIZE: usize = 0x10_0000;
const LIB_START: usize = 0xffff_ffc0_8030_0000;

pub fn load_elf() -> u64 {
    debug!("Load payload ...");
    // Load X out file
    let app_elf_size = unsafe { *(PLASH_START as *const usize) };
    let app_elf_slice = unsafe { from_raw_parts((PLASH_START + 0x8) as *const u8, app_elf_size) };
    let app_code = unsafe { from_raw_parts_mut((APP_START) as *mut u8, MAX_APP_SIZE) };

    let app_elf: ElfBytes<'_, LittleEndian> =
        ElfBytes::<LittleEndian>::minimal_parse(app_elf_slice).expect("Failed to parse ELF");

    let is_need_interp = {
        if let Some(segments) = app_elf.segments() {
            let mut is_pie = false;
            for segment in segments {
                debug!("Segment type: {}", segment.p_type);
                if segment.p_type == elf::abi::PT_INTERP {
                    is_pie = true;
                }
            }
            is_pie
        } else {
            false
        }
    };
    debug!(
        "Dynamic interpreter (.interp section) exists: {}",
        is_need_interp
    );

    let entry: u64 = {
        if is_need_interp == false {
            // Static and position independent executable
            debug!("Static and position independent app");
            let _ = load_exec(&app_elf, app_elf_slice, app_code);
            app_elf.ehdr.e_entry
        } else {
            debug!("Dynamic link app");
            let lib_elf_size = unsafe { *((PLASH_START + app_elf_size + 0x8) as *const usize) };
            let lib_elf_slice = unsafe {
                from_raw_parts(
                    (PLASH_START + app_elf_size + 0x10) as *const u8,
                    lib_elf_size,
                )
            };
            let lib_code = unsafe { from_raw_parts_mut((LIB_START) as *mut u8, MAX_LIB_SIZE) };

            let lib_elf: ElfBytes<'_, LittleEndian> =
                ElfBytes::<LittleEndian>::minimal_parse(lib_elf_slice)
                    .expect("Failed to parse ELF at SO file");

            debug!(
                "ELF Headers App: 0x{:x}, Lib: 0x{:x}",
                app_elf.ehdr.e_ehsize, lib_elf.ehdr.e_ehsize
            );

            debug!("Load lib to mem space");
            load_dyn(&lib_elf, lib_elf_slice, lib_code, 0);
            debug!("Load app to mem space");
            load_dyn(&app_elf, app_elf_slice, app_code, 0);

            modify_plt(&app_elf, &lib_elf);
            modify_plt_for_lib(&app_elf, &lib_elf);

            println!("Lib elf size: 0x{:x}", lib_elf_size);
            LIB_START as u64 + lib_elf.ehdr.e_entry
        }
    };

    println!("App elf size: 0x{:x}", app_elf_size);
    return entry;
}

fn load_exec(
    app_elf: &ElfBytes<LittleEndian>,
    app_elf_slice: &[u8],
    app_code: &mut [u8],
) -> Result<(), LoadError> {
    // 检查 ELF 头
    verify_elf_header(app_elf).expect("Failed to verify ELF header");

    if let Some(phs) = app_elf.segments() {
        for ph in phs {
            if ph.p_type != PT_LOAD {
                debug!("skipping segment type: {}", ph.p_type);
                continue;
            }

            let offset = ph.p_offset as usize;
            let filesz = ph.p_filesz as usize;
            let memsz = ph.p_memsz as usize;

            // 计算在内存中的实际地址
            let vaddr = ph.p_vaddr as usize;
            let dest_addr = vaddr - APP_START;

            debug!(
                "Loading segment: offset=0x{:x}, filesz=0x{:x}, memsz=0x{:x}, vaddr=0x{:x}",
                offset, filesz, memsz, vaddr
            );

            debug!("dest_addr: {}", dest_addr);

            // 复制段内容
            if filesz > 0 {
                let src: &[u8] = &app_elf_slice[offset..offset + filesz];
                let dest = &mut app_code[dest_addr..dest_addr + filesz];
                dest.copy_from_slice(src);
            }

            // 处理`.bss`等需要零初始化的部分
            if memsz > filesz {
                let dest = &mut app_code[dest_addr + filesz..dest_addr + memsz];
                dest.fill(0);
            }
        }
    }
    /* ```
        let text_shdr = app_elf
            .section_header_by_name(".text")
            .expect("section table should be parseable")
            .expect("elf should have a .text section");
        let text_slice = app_elf_slice
            .get(text_shdr.sh_offset as usize..)
            .expect("text section should be in bounds");
        let copy_size = min(app_code.len(), text_slice.len());
        app_code[..copy_size].copy_from_slice(&text_slice[..copy_size]);
    ```*/
    Ok(())
}

fn load_dyn(
    elf: &ElfBytes<LittleEndian>,
    elf_slice: &[u8],
    run_code: &mut [u8],
    address_bios: usize,
) {
    let phdrs = elf.segments().expect("Failed to parse program headers");
    for phdr in phdrs {
        if phdr.p_type != elf::abi::PT_LOAD {
            continue;
        }
        debug!(
            "Load Segment vaddr 0x{:x} offset 0x{:x} filesz 0x{:x} memsz 0x{:x} address_bios 0x{:x}",
            phdr.p_vaddr as usize,
            phdr.p_offset as usize,
            phdr.p_offset as usize,
            phdr.p_memsz as u64,
            address_bios
        );
        load_segment(
            run_code,
            elf_slice,
            phdr.p_vaddr as usize,
            phdr.p_offset as usize,
            phdr.p_filesz as usize,
            phdr.p_memsz as usize,
            address_bios,
        );
    }
}

fn load_segment(
    run_code: &mut [u8],
    elf_slice: &[u8],
    p_vaddr: usize,
    p_offset: usize,
    p_filesz: usize,
    p_memsz: usize,
    address_bios: usize,
) {
    // Copy the segment into the executable zone
    // If `memsz` is larger than `filesz`, zero out the rest
    let run_code_offset = p_vaddr - address_bios;
    run_code[run_code_offset..run_code_offset + p_filesz]
        .copy_from_slice(&elf_slice[p_offset..p_offset + p_filesz]);
    if p_memsz > p_filesz {
        let zero_size = min(run_code.len() - p_filesz, p_memsz - p_filesz);
        run_code[run_code_offset + p_filesz..run_code_offset + p_filesz + zero_size].fill(0);
    }
}

fn modify_plt_for_lib(elf: &ElfBytes<LittleEndian>, so_elf: &ElfBytes<LittleEndian>) {
    debug!("modify for solib");
    let (so_dynsym_table, so_dynstr_table) = so_elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");
    let so_rela_shdr = so_elf
        .section_header_by_name(".rela.plt")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.plt section");
    let so_relas = so_elf
        .section_data_as_relas(&so_rela_shdr)
        .expect("Failed to parse .rela.plt section");

    let run_code_entry_name = "main";

    for rela in so_relas {
        let sym = so_dynsym_table
            .get(rela.r_sym as usize)
            .expect("Failed to get symbol");
        let rela_name = so_dynstr_table
            .get(sym.st_name as usize)
            .expect("Failed to get symbol name");
        if rela_name == run_code_entry_name {
            unsafe {
                *((LIB_START as u64 + rela.r_offset) as *mut usize) =
                    APP_START + elf.ehdr.e_entry as usize;
                debug!(
                    "[plt] @0x{:x} value 0x{:x} st_name {}",
                    LIB_START as u64 + rela.r_offset,
                    APP_START + elf.ehdr.e_entry as usize,
                    rela_name
                );
            }
        } else {
            unsafe {
                *((LIB_START as u64 + rela.r_offset) as *mut usize) =
                    LIB_START + sym.st_value as usize;
                debug!(
                    "[plt] @0x{:x} value 0x{:x} st_name {}",
                    LIB_START as u64 + rela.r_offset,
                    LIB_START + sym.st_value as usize,
                    rela_name
                );
            }
        }
    }

    let so_rela_shdr = so_elf
        .section_header_by_name(".rela.dyn")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.dyn section");
    let so_relas = so_elf
        .section_data_as_relas(&so_rela_shdr)
        .expect("Failed to parse .rela.dyn section");

    let symbols_to_find = ["abi_entry"];

    for rela in so_relas {
        let sym = so_dynsym_table
            .get(rela.r_sym as usize)
            .expect("Failed to get symbol");
        let rela_name = so_dynstr_table
            .get(sym.st_name as usize)
            .expect("Failed to get symbol name");
        for symbol_name in symbols_to_find.iter() {
            if rela_name == *symbol_name {
                unsafe {
                    *((LIB_START as u64 + rela.r_offset) as *mut usize) =
                        ABI_TABLE.as_ptr() as usize;
                    debug!(
                        "[got] @0x{:x} value 0x{:x} st_name {}",
                        LIB_START as u64 + rela.r_offset,
                        ABI_TABLE.as_ptr() as usize,
                        rela_name
                    );
                }
                break;
            }
        }
    }
}

fn modify_plt(elf: &ElfBytes<LittleEndian>, so_elf: &ElfBytes<LittleEndian>) {
    debug!("modify for run code");
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
    // Get SO lib symbol table
    let (so_dynsym_table, so_dynstr_table) = so_elf
        .dynamic_symbol_table()
        .expect("Failed to parse SO dynamic symbol table")
        .expect("SO ELF should have a dynamic symbol table");

    for rela in relas {
        // Get the `r_sym'th` symbol from the dynamic symbol table
        let sym = dynsym_table
            .get(rela.r_sym as usize)
            .expect("Failed to get symbol");
        let rela_name = dynstr_table
            .get(sym.st_name as usize)
            .expect("Failed to get symbol name");
        // Find symbol in SO ELF
        let so_sym = so_dynsym_table
            .iter()
            .find(|s| {
                let name = so_dynstr_table.get(s.st_name as usize).unwrap_or(&"");
                name == rela_name
            })
            .expect("Failed to find symbol in SO dynamic symbol table");

        unsafe {
            *((APP_START as u64 + rela.r_offset) as *mut usize) =
                LIB_START + so_sym.st_value as usize;
            debug!(
                "[plt] @0x{:x} value 0x{:x} st_name {}",
                APP_START as u64 + rela.r_offset,
                LIB_START + so_sym.st_value as usize,
                rela_name
            );
        }
    }
}
