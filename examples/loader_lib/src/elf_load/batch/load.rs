use core::cmp::min;

use axstd::format;

use axlog::{debug, info};

use elf::{
    ElfBytes,
    abi::{PT_LOAD, R_RISCV_64, R_RISCV_JUMP_SLOT, R_RISCV_RELATIVE},
    endian::LittleEndian,
};

pub fn load_exec(
    app_elf: &ElfBytes<LittleEndian>,
    app_elf_slice: &[u8],
    app_code: &mut [u8],
    app_start: usize,
) {
    if let Some(phs) = app_elf.segments() {
        for ph in phs {
            if ph.p_type != PT_LOAD {
                continue;
            }

            let offset = ph.p_offset as usize;
            let filesz = ph.p_filesz as usize;
            let memsz = ph.p_memsz as usize;

            // 计算在内存中的实际地址
            let vaddr = ph.p_vaddr as usize;
            vaddr
                .lt(&app_start)
                .then(|| panic!("segments vaddr less then app_start"));
            let dest_addr = vaddr - app_start;

            debug!(
                "Loading segment: offset=0x{:x}, filesz=0x{:x}, memsz=0x{:x}, vaddr=0x{:x}",
                offset, filesz, memsz, vaddr
            );

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
}

pub fn load_dyn(
    elf: &ElfBytes<LittleEndian>,
    elf_slice: &[u8],
    run_code: &mut [u8],
    address_bios: usize,
) {
    let phdrs = elf.segments().expect("Failed to parse program headers");
    for phdr in phdrs {
        if phdr.p_type != PT_LOAD {
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

pub fn load_segment(
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

/// 加载 `APP` 库
///
/// # 参数
/// * `app_elf_slice` - `APP` 基于在 `PLASH` 中的入口地址与在 `PLASH` 中的大小，
/// 获取其在 `PLASH` 中的切片。
/// * `app_code` - `APP` 基于在 `MEM` 中的入口地址与在 `MEM` 中的大小，
/// 获取其在 `MEM` 中的切片。
/// * `app_size` - `APP` 在 `PLASH` 中的大小
/// * `app_start` - `APP` 在 `MEM` 中的入口地址
/// * `lib_elf` - `LIB` 基于在 `PLASH` 的入口地址和大小，获得其切片，转化为 `ElfBytes`
/// * `lib_start` - `LIB` 在 `MEM` 中的入口地址
///
/// # 返回值
/// (app entry, 函数入口地址(main entry))
pub fn load_app_dyn(
    app_elf_slice: &[u8],
    app_elf: &ElfBytes<LittleEndian>,
    app_code: &mut [u8],
    app_size: usize,
    app_start: usize,
    lib_elf: &ElfBytes<LittleEndian>,
    lib_start: usize,
    main_name: &str,
) -> (usize, usize) {
    debug!("Load app to mem space");
    load_dyn(&app_elf, app_elf_slice, app_code, 0);

    debug!("Modify lib to mem space");
    modify_app(&app_elf, app_start, &lib_elf, lib_start);

    info!("App elf size: 0x{:x}", app_size);
    (
        app_start + app_elf.ehdr.e_entry as usize,
        find_app_func_entry(&app_elf, app_start, main_name),
    )
}

/// 加载 `LIB` 库
///
/// # 参数
/// * `lib_size` - `LIB` 在 `PLASH` 中的大小
/// * `lib_entry` - `LIB` 在 `PLASH` 中的入口地址
/// * `lib_start` - `LIB` 在 `MEM` 中的入口地址
/// * `max_lib_size` - `LIB` 在 `MEM` 中的最大大小
///
/// # 返回值
/// `Lib` 库的入口函数
pub fn load_lib(
    lib_elf_slice: &[u8],
    lib_code: &mut [u8],
    lib_elf: &ElfBytes<LittleEndian>,
    lib_start: usize,
    run_code_entry_name: &str,
    run_code_main_name: &str,
) -> usize {
    debug!("Load lib to mem space");
    load_dyn(&lib_elf, lib_elf_slice, lib_code, 0);

    debug!("Modify lib to mem space");
    modify_lib(&lib_elf, lib_start, run_code_entry_name, run_code_main_name);

    // 正常情况下，应该是由 `APP` 内的 `start` 函数开始执行，
    // 但是我们在 `Batch Mode` 下需要重新写 `start` 函数，因此从这里开始
    return lib_start + lib_elf.ehdr.e_entry as usize;
}

/// 传递一个 `APP` 的 `ElfBytes`，查询某一个函数的入口地址，返回
/// # 参数
/// * `elf` - `APP` 基于在 `PLASH` 的入口地址和大小，获得其切片，转化为 `ElfBytes`
/// * `app_start` - `APP` 在 `MEM` 中的入口地址
/// * `entry_name` - 待查询的入口地址，通常为 `main`
fn find_app_func_entry(elf: &ElfBytes<LittleEndian>, app_start: usize, func_name: &str) -> usize {
    let (dynsym_table, dynstr_table) = elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");

    debug!("Finding func entry {}", func_name);

    let sym = dynsym_table
        .iter()
        .find(|sym| {
            let name = dynstr_table
                .get(sym.st_name as usize)
                .expect("Failed to get name in dynstr_table");
            name == func_name
        })
        .expect("Failed to find symbol in APP dynamic symbol table");
    sym.st_value.eq(&0).then(|| panic!("Bad func entry"));
    app_start + sym.st_value as usize
}

/// 这个是用来修改 `LIB` 库的 `PLT` 表格的，但是不会修改 `main` 函数。
///
/// # 参数
/// * `elf` -
/// * `lib_start` - `LIB` 在内存中地址
fn modify_lib(
    elf: &ElfBytes<LittleEndian>,
    lib_start: usize,
    run_code_entry_name: &str,
    run_code_main_name: &str,
) {
    let (dynsym_table, dynstr_table) = elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");

    info!("modify for LIB-RELA.PLT so lib");
    let rela_plt_shdr = elf
        .section_header_by_name(".rela.plt")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.plt section");
    let rela_plts = elf
        .section_data_as_relas(&rela_plt_shdr)
        .expect("Failed to parse .rela.plt section");

    for rela_plt in rela_plts {
        let sym = dynsym_table.get(rela_plt.r_sym as usize).expect(&format!(
            "Failed to get symbol for index: {}",
            rela_plt.r_sym
        ));
        let rela_name = dynstr_table.get(sym.st_name as usize).expect(&format!(
            "Failed to get symbol name for index: {}",
            sym.st_name
        ));

        match rela_plt.r_type {
            // Indicates the symbol associated with a `PLT` entry: `S`
            R_RISCV_JUMP_SLOT => {
                // 在遇到特殊函数的时候，跳过。
                if rela_name != run_code_entry_name && rela_name != run_code_main_name {
                    let relative_offset = lib_start + rela_plt.r_offset as usize;
                    let new_value = lib_start + sym.st_value as usize;
                    debug!(
                        "[Lib-rela.plt R_RISCV_JUMP_SLOT] @0x{:x}=0x{:x} st_name {}",
                        relative_offset, new_value, rela_name,
                    );
                    sym.st_value.eq(&0).then(|| panic!("Bad st_value"));
                    unsafe { *(relative_offset as *mut usize) = new_value };
                }
            }
            _ => {
                panic!("Unknown relocation type: {}", rela_plt.r_type);
            }
        }
    }

    info!("modify for LIB-RELA.DYN so lib");
    let rela_dyn_shdr = elf
        .section_header_by_name(".rela.dyn")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.dyn section");
    let rela_dyns = elf
        .section_data_as_relas(&rela_dyn_shdr)
        .expect("Failed to parse .rela.dyn section");
    for rela_dyn in rela_dyns {
        let sym = dynsym_table.get(rela_dyn.r_sym as usize).expect(&format!(
            "Failed to get symbol for index: {}",
            rela_dyn.r_sym
        ));
        let rela_name = dynstr_table.get(sym.st_name as usize).expect(&format!(
            "Failed to get symbol name for index: {}",
            sym.st_name
        ));

        match rela_dyn.r_type {
            // Adjust a link address (A) to its load address: `(B + A)`.
            R_RISCV_RELATIVE => {
                let relative_offset = lib_start + rela_dyn.r_offset as usize;
                let new_value = lib_start + rela_dyn.r_addend as usize;
                debug!(
                    "[Lib-rela.dyn R_RISCV_RELATIVE] @0x{:x}=0x{:x}",
                    relative_offset, new_value,
                );
                rela_dyn.r_addend.eq(&0).then(|| panic!("Bad st_value"));
                unsafe { *(relative_offset as *mut usize) = new_value };
            }
            // 64-bit relocation: `S + A`.
            R_RISCV_64 => {
                // 在遇到特殊符号的时候，跳过。
                if rela_name != run_code_entry_name && rela_name != run_code_main_name {
                    let relative_offset = lib_start + rela_dyn.r_offset as usize;
                    let new_value = lib_start + sym.st_value as usize;
                    debug!(
                        "[Lib-rela.dyn R_RISCV_64] @0x{:x}=0x{:x} name {}",
                        relative_offset, new_value, rela_name,
                    );
                    sym.st_value.eq(&0).then(|| panic!("Bad st_value"));
                    unsafe { *(relative_offset as *mut usize) = new_value };
                }
            }
            _ => {
                panic!("Unknown relocation type: {}", rela_dyn.r_type);
            }
        }
    }
}

/// 修改 `Lib` 库中的 指定符号 的地址
///
/// # 参数
/// * `elf` - `LIB` 的 `ElfBytes`
/// * `lib_start` - `LIB` 在内存中地址
/// * `new_value` - 需要修改的指定符号地址
/// * `name` - 需要修改的指定符号的名字
pub fn modify_lib_dyn(
    elf: &ElfBytes<LittleEndian>,
    lib_start: usize,
    new_value: usize,
    name: &str,
) -> Option<()> {
    let (dynsym_table, dynstr_table) = elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");

    info!("Modify `main` function entry in lib");
    // 获取 `.rela.plt` section
    let rela_dyn_shdr = elf
        .section_header_by_name(".rela.dyn")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.dyn section");
    // 解析 `.rela.plt` 中的重定位条目
    let rela_dyns = elf
        .section_data_as_relas(&rela_dyn_shdr)
        .expect("Failed to parse .rela.plt section");

    let rela_dyn = rela_dyns.into_iter().find(|rela_dyn| {
        let sym = dynsym_table.get(rela_dyn.r_sym as usize).expect(&format!(
            "Failed to get symbol for index: {}",
            rela_dyn.r_sym
        ));
        let rela_name = dynstr_table.get(sym.st_name as usize).expect(&format!(
            "Failed to get symbol name for index: {}",
            sym.st_name
        ));

        name == rela_name
    })?; // 如果找不到，就在这里返回一个None

    match rela_dyn.r_type {
        // Indicates the symbol associated with a `PLT` entry: `S`
        R_RISCV_64 => {
            let relative_offset = lib_start + rela_dyn.r_offset as usize;
            info!(
                "[Lib-rela.plt dyn:{}] @0x{:x}=0x{:x}",
                name, relative_offset, new_value
            );
            unsafe { *(relative_offset as *mut usize) = new_value };
        }
        _ => {
            panic!("Unknown relocation type: {}", rela_dyn.r_type);
        }
    }
    Some(())
}

/// 修改 `Lib` 库中的 指定函数 的入口地址
///
/// # 参数
/// * `elf` - `LIB` 的 `ElfBytes`
/// * `lib_start` - `LIB` 在内存中地址
/// * `new_value` - 需要修改的特殊函数函数入口地址
/// * `name` - 需要修改的特殊函数函数的名字
pub fn modify_lib_plt(
    elf: &ElfBytes<LittleEndian>,
    lib_start: usize,
    new_value: usize,
    name: &str,
) -> Option<()> {
    let (dynsym_table, dynstr_table) = elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");

    info!("Modify `main` function entry in lib");
    // 获取 `.rela.plt` section
    let rela_plt_shdr = elf
        .section_header_by_name(".rela.plt")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.plt section");
    // 解析 `.rela.plt` 中的重定位条目
    let rela_plts = elf
        .section_data_as_relas(&rela_plt_shdr)
        .expect("Failed to parse .rela.plt section");

    let rela_plt = rela_plts.into_iter().find(|rela_plt| {
        let sym = dynsym_table.get(rela_plt.r_sym as usize).expect(&format!(
            "Failed to get symbol for index: {}",
            rela_plt.r_sym
        ));
        let rela_name = dynstr_table.get(sym.st_name as usize).expect(&format!(
            "Failed to get symbol name for index: {}",
            sym.st_name
        ));

        name == rela_name
    })?; // 如果找不到，就在这里返回一个None

    match rela_plt.r_type {
        // Indicates the symbol associated with a `PLT` entry: `S`
        R_RISCV_JUMP_SLOT => {
            let relative_offset = lib_start + rela_plt.r_offset as usize;
            info!(
                "[Lib-rela.plt plt:{}] @0x{:x}=0x{:x}",
                name, relative_offset, new_value
            );
            unsafe { *(relative_offset as *mut usize) = new_value };
        }
        _ => {
            panic!("Unknown relocation type: {}", rela_plt.r_type);
        }
    }
    Some(())
}

/// 这个是用来修改 `APP` 库的 `PLT` 表格的
///
/// # 参数
/// * `app_elf` - `APP` 文件本身
/// * `app_start` - 内存中 `APP` 的起始地址
/// * `lib_elf` - `LIB` 文件本身
/// * `lib_start` - 内存中 `LIB` 的起始地址
fn modify_app(
    app_elf: &ElfBytes<LittleEndian>,
    app_start: usize,
    lib_elf: &ElfBytes<LittleEndian>,
    lib_start: usize,
) {
    let (app_dynsym_table, app_dynstr_table) = app_elf
        .dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");
    let (lib_dynsym_table, lib_dynstr_table) = lib_elf
        .dynamic_symbol_table()
        .expect("Failed to parse LIB dynamic symbol table")
        .expect("LIB ELF should have a dynamic symbol table");

    info!("modify for APP-RELA.PLT run code");
    let app_rela_plt_shdr = app_elf
        .section_header_by_name(".rela.plt")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.plt section");
    let app_rela_plts = app_elf
        .section_data_as_relas(&app_rela_plt_shdr)
        .expect("Failed to parse .rela.dyn section");

    for app_rela_plt in app_rela_plts {
        // Get the `r_sym'th` symbol from the dynamic symbol table
        let app_sym = app_dynsym_table
            .get(app_rela_plt.r_sym as usize)
            .expect("Failed to get symbol");
        let app_rela_name = app_dynstr_table
            .get(app_sym.st_name as usize)
            .expect("Failed to get symbol name");

        match app_rela_plt.r_type {
            // Indicates the symbol associated with a `PLT` entry: `S`
            R_RISCV_JUMP_SLOT => {
                debug!("Finding symbol name {}", app_rela_name);
                // Find symbol in LIB ELF
                let lib_sym = lib_dynsym_table
                    .iter()
                    .find(|s| {
                        let name = lib_dynstr_table.get(s.st_name as usize).unwrap_or(&"");
                        // 如果开始寻找`__libc_start_main`，就替换成我们的实现
                        name == app_rela_name
                    })
                    .expect("Failed to find symbol in LIB dynamic symbol table");

                let relative_offset = app_start + app_rela_plt.r_offset as usize;
                let new_value = lib_start + lib_sym.st_value as usize;
                debug!(
                    "[App-rela.plt R_RISCV_JUMP_SLOT] @0x{:x}=0x{:x} st_name {}",
                    relative_offset, new_value, app_rela_name,
                );
                lib_sym.st_value.eq(&0).then(|| panic!("Bad st_value"));
                unsafe { *(relative_offset as *mut usize) = new_value };
            }
            _ => {
                panic!("Unknown relocation type: {}", app_rela_plt.r_type);
            }
        }
    }

    info!("modify for APP-RELA.DYN run code");
    let app_rela_dyn_shdr = app_elf
        .section_header_by_name(".rela.dyn")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.dyn section");
    let app_rela_dyns = app_elf
        .section_data_as_relas(&app_rela_dyn_shdr)
        .expect("Failed to parse .rela.dyn section");

    for app_rela_dyn in app_rela_dyns {
        let app_sym = app_dynsym_table
            .get(app_rela_dyn.r_sym as usize)
            .expect(&format!(
                "Failed to get symbol for index: {}",
                app_rela_dyn.r_sym
            ));
        let app_rela_name = app_dynstr_table
            .get(app_sym.st_name as usize)
            .expect(&format!(
                "Failed to get symbol name for index: {}",
                app_sym.st_name
            ));

        match app_rela_dyn.r_type {
            // Adjust a link address (A) to its load address: `(B + A)`.
            R_RISCV_RELATIVE => {
                let relative_offset = app_start + app_rela_dyn.r_offset as usize;
                let new_value = app_start + app_rela_dyn.r_addend as usize;
                debug!(
                    "[App-rela.dyn R_RISCV_RELATIVE] @0x{:x}=0x{:x}",
                    relative_offset, new_value,
                );
                app_rela_dyn.r_addend.eq(&0).then(|| panic!("Bad st_value"));
                unsafe { *(relative_offset as *mut usize) = new_value };
            }
            // 64-bit relocation: `S + A`.
            R_RISCV_64 => {
                // Find symbol in LIB ELF
                let lib_sym = lib_dynsym_table
                    .iter()
                    .find(|s| {
                        let name = lib_dynstr_table.get(s.st_name as usize).unwrap_or(&"");
                        name == app_rela_name
                    })
                    .expect("Failed to find symbol in LIB dynamic symbol table");

                let relative_offset = app_start + app_rela_dyn.r_offset as usize;
                let new_value = lib_start + lib_sym.st_value as usize;
                debug!(
                    "[App-rela.dyn R_RISCV_64] @0x{:x}=0x{:x} name {}",
                    relative_offset, new_value, app_rela_name,
                );
                lib_sym.st_value.eq(&0).then(|| panic!("Bad lib st_value"));
                app_sym.st_value.ne(&0).then(|| panic!("Bad app st_value"));
                unsafe { *(relative_offset as *mut usize) = new_value };
            }
            _ => {
                panic!("Unknown relocation type: {}", app_rela_dyn.r_type);
            }
        }
    }
}
