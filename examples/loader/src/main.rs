#![feature(asm_const)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg(feature = "axstd")]
use axstd::print;

/// `bin`的开始位置
const PLASH_START: usize = 0xffff_ffc0_2200_0000;

/// 可执行代码的位置
/// ```
/// app running aspace
/// SBI(0x80000000) -> App <- Kernel(0x80200000)
/// va_pa_offset: 0xffff_ffc0_0000_0000
/// ```
const RUN_START: usize = 0xffff_ffc0_8010_0000;

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
    //`println!("[ABI:Print] {c}");`
    print!("\x1b[34m");
    print!("{c}");
    print!("\x1b[0m");
}

fn abi_terminate() {
    print!("\x1b[34m");
    println!("Bye");
    print!("\x1b[0m");

    #[cfg(feature = "axstd")]
    axstd::process::exit(0);
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

/// 需要注意的是，由于我的操作系统与8字节对齐，所以在构建结构体的时候，就会要求8字节对齐。
/// 注意在命令行中构建结构体时，要考虑到对齐的要求，
/// 建议直接使用usize，而非core::mem::size_of::<>();来获得大小
#[repr(C)]
struct AppHeader {
    magic_number: usize, // 用于识别 4bytes
    app_size: usize,     // 应用程序大小 8bytes
    entry_point: usize,  // 入口函数地址
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);

    let mut file_ptr = PLASH_START as usize;
    // 不建议使用
    let header_size = core::mem::size_of::<AppHeader>();
    println!("Load payload ...");

    loop {
        println!("Loading applications...");
        // Now, point to header.
        let app_header_ptr = file_ptr as *const AppHeader;
        let app_header = unsafe { &*app_header_ptr };
        if app_header.magic_number != 0x1234567812345678 {
            println!("Exiting loop. Bad magic number.");
            break;
        }

        // Move to code 不建议使用
        file_ptr += header_size;

        // 根据头部信息获取每个应用程序的起始地址和大小
        let apps_start = file_ptr as *const u8;
        let apps_size = app_header.app_size;

        println!(
            "header {:?} start {:?} entry {:?} size {}",
            app_header_ptr, apps_start, app_header.entry_point as *const u8, apps_size
        );

        // 读取应用程序代码
        let app_code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };

        let run_code = unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, apps_size) };
        run_code.copy_from_slice(app_code);

        println!("Execute app ...");

        // Execute app
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
                in("t2") app_header.entry_point,
            )
        }

        file_ptr += apps_size;
        println!("Loading complete!");
    }

    println!("Load payload ok!");

    unsafe {
        bye();
    }
}
