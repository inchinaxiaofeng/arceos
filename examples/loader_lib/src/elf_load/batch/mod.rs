pub mod head_decoder;
pub mod load;
pub mod script_decoder;

use core::slice::{from_raw_parts, from_raw_parts_mut};

use axlog::{debug, info};

use elf::{
    ElfBytes,
    abi::{ET_DYN, ET_EXEC},
    endian::LittleEndian,
};
use head_decoder::head_decoded;
use load::{load_app_dyn, load_exec, load_lib, modify_lib_dyn, modify_lib_plt};
use script_decoder::script_decoded;

use crate::{
    abi::{ABI_TABLE, mem::cleanup_memory},
    config::{
        APP_START, GLOBAL_SOTRE, LIB_START, MAX_APP_SIZE, MAX_LIB_SIZE, PLASH_SIZE, PLASH_START,
    },
    elf_load::verify::verify_elf_header,
};

struct LibInfo<'a> {
    pub elf: ElfBytes<'a, LittleEndian>,
    pub entry: usize,
}

pub fn run_loop() {
    info!("Load payload ...");
    debug!("Decode head and script...");
    let head_decoded = head_decoded(PLASH_START, PLASH_SIZE);
    info!("HeadDecoded {:?}", head_decoded);
    let script_decoded = script_decoded(
        PLASH_START + head_decoded.script.1 as usize,
        head_decoded.script.0 as usize,
    );

    let lib = if head_decoded.lib.0 != 0 && head_decoded.lib.1 != 0 {
        if head_decoded.lib.0 as usize > MAX_LIB_SIZE {
            panic!("Lib size > max lib size");
        }
        let lib_elf_slice = unsafe {
            from_raw_parts(
                (PLASH_START + (head_decoded.lib.1 as usize)) as *const u8,
                head_decoded.lib.0 as usize,
            )
        };
        let lib_code = unsafe { from_raw_parts_mut((LIB_START) as *mut u8, MAX_LIB_SIZE) };
        let lib_elf: ElfBytes<'_, LittleEndian> =
            ElfBytes::<LittleEndian>::minimal_parse(lib_elf_slice)
                .expect("Failed to parse ELF at LIB file");
        verify_elf_header(&lib_elf)
            .is_err()
            .then(|| panic!("Failed to verify_elf_header for Lib ELF"));

        let lib_entry = load_lib(
            lib_elf_slice,
            lib_code,
            &lib_elf,
            LIB_START,
            "__entry",
            "main",
        );

        info!(
            "Load lib done, entry 0x{:x} size 0x{:x}",
            lib_entry, head_decoded.lib.0
        );
        Some(LibInfo {
            elf: lib_elf,
            entry: lib_entry,
        })
    } else if head_decoded.lib.0 == 0 && head_decoded.lib.1 == 0 {
        None
    } else {
        panic!("Error in Lib part");
    };

    // 基于 `script_decoded` 进行加载与执行
    for i in 0..script_decoded.line_num as usize {
        info!("Read script");
        let app_name = script_decoded.lines_meta[i].0.clone();
        let arg_entry = script_decoded.lines_meta[i].1.argc_ptr();

        let app = head_decoded
            .apps
            .iter()
            .find(|app| app.1 == app_name)
            .expect("Failed to find app");

        info!("Load APP");
        if app.0 as usize > MAX_APP_SIZE {
            panic!("App size > max app size");
        }
        let app_elf_slice = unsafe {
            from_raw_parts(
                (PLASH_START + (app.2 as usize)) as *const u8,
                app.0 as usize,
            )
        };
        let app_code = unsafe { from_raw_parts_mut((APP_START) as *mut u8, MAX_APP_SIZE) };
        let app_elf: ElfBytes<'_, LittleEndian> =
            ElfBytes::<LittleEndian>::minimal_parse(app_elf_slice)
                .expect("Failed to parse ELF at APP file");
        verify_elf_header(&app_elf)
            .is_err()
            .then(|| panic!("Failed to verify_elf_header for App ELF"));

        // 判断是否需要动态链接器
        let entry = match app_elf.ehdr.e_type {
            ET_EXEC => {
                // 在需要动态链接器的时候（静态加载），就可以直接使用App的Entry
                load_exec(&app_elf, app_elf_slice, app_code, APP_START);

                app_elf.ehdr.e_entry as usize
            }
            ET_DYN => {
                let lib = lib
                    .as_ref()
                    .expect("ERROR: A dynamic app loaded but cant find lib.");
                // 动态连接加载
                let (app_entry, _main_entry) = load_app_dyn(
                    app_elf_slice,
                    &app_elf,
                    app_code,
                    app.0 as usize,
                    APP_START,
                    &lib.elf, // ✅ 这里用 &lib.elf 避免 Move
                    LIB_START,
                    "main",
                );
                modify_lib_plt(&lib.elf, LIB_START, app_entry, "__entry")
                    .is_none()
                    .then(|| {
                        modify_lib_dyn(&lib.elf, LIB_START, app_entry, "__entry")
                            .expect("Failed to modify in lib")
                    });
                // modify_lib_func_entry(&lib.elf, LIB_START, main_entry, "main");

                lib.entry // ✅ 这里还是可用
            }
            _ => panic!("Unsupported ehdr type {:?}", app_elf.ehdr.e_type),
        };

        info!(
            "Entry @0x{:x} arg @0x{:x}",
            entry as usize, arg_entry as usize
        );

        let global_store = GLOBAL_SOTRE;

        unsafe {
            core::arch::asm!("
            // 除了通用寄存器，还需要保存其他内容
            addi    sp, sp, -240

            // 保存通用寄存器
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
            sd      t4, 88(sp)
            sd      t3, 96(sp)
            sd      t2, 104(sp)
            sd      t1, 112(sp)
            sd      t0, 120(sp)
            sd      s11,128(sp)
            sd      s10,136(sp)
            sd      s9, 144(sp)
            sd      s8, 152(sp)
            sd      s7, 160(sp)
            sd      s6, 168(sp)
            sd      s5, 176(sp)
            sd      s4, 184(sp)
            sd      s3, 192(sp)
            sd      s2, 200(sp)
            sd      s1, 208(sp)
            sd      s0, 216(sp)
            sd      gp, 224(sp)
            sd      tp, 232(sp)

            mv      t1, {global_store}
            sd      sp, 0(t1)

            // 使用临时寄存器保存栈指针和返回地址，避免竞争
            auipc   t2, 0                  // 当前指令的地址（基址）
            addi    t2, t2, 14
            sd      t2, 8(t1)

            // 重进入跳板段
            jal     t2, 8                  // 写入t2没有什么特别的考量
            jal     t2, 18                  // 在执行重进入函数前，这个语句不应当被调用

            mv      t2, {entry}
            mv      a0, {param}             // 将参数p的地址加载到a0
            la      a7, {abi_table}
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
            ld      t4, 88(sp)
            ld      t3, 96(sp)
            ld      t2, 104(sp)
            ld      t1, 112(sp)
            ld      t0, 120(sp)
            ld      s11,128(sp)
            ld      s10,136(sp)
            ld      s9, 144(sp)
            ld      s8, 152(sp)
            ld      s7, 160(sp)
            ld      s6, 168(sp)
            ld      s5, 176(sp)
            ld      s4, 184(sp)
            ld      s3, 192(sp)
            ld      s2, 200(sp)
            ld      s1, 208(sp)
            ld      s0, 216(sp)
            ld      gp, 224(sp)
            ld      tp, 232(sp)

            addi    sp, sp, 240
            ",
                abi_table = sym ABI_TABLE,
                global_store = in(reg) global_store,
                param = in(reg) arg_entry,
                entry = in(reg) entry,
                options(nostack, nomem)
            )
        }

        cleanup_memory();
        info!("Done app");
    }
}

// 汇编重新进入的循环位置
pub extern "C" fn reentry_label() -> ! {
    cleanup_memory();
    let store0 = unsafe { *(GLOBAL_SOTRE as *const usize) }.clone();
    let store8 = unsafe { *((GLOBAL_SOTRE + 8) as *const usize) }.clone();
    info!("GLOBAL store0 0x{:x}, store8 0x{:x}", store0, store8);
    unsafe {
        core::arch::asm!(
            "
            mv      sp, {global_store0}
            mv      t2, {global_store8}
            jalr    t2
            ",
            global_store0 = in(reg) store0,
            global_store8 = in(reg) store8,
            options(nostack, nomem, noreturn)
        );
    }
}
