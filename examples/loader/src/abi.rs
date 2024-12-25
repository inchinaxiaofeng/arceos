use axhal::time::monotonic_time;
use axlog::{debug, info};
use axstd::{print, println, process::exit, string::ToString};
use core::{ffi::VaList, ptr::copy_nonoverlapping};
use cty::{c_char, c_int};
use printf_compat::output::display;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
pub const SYS_TERMINATE: usize = 3;
const SYS_TIMESPEC: usize = 4;
const SYS_VFPRINTF: usize = 5;
const SYS_VSPRINTF: usize = 6;

pub static mut ABI_TABLE: [usize; 16] = [0; 16];

// TODO: 将后来的函数添加进去
// `map func name to func addr`
pub static STR_TO_FUNC: [(&str, AbiFunction); 3] = [
    ("hello", AbiFunction::Hello(abi_hello)),
    ("putchar", AbiFunction::Putchar(abi_putchar)),
    ("exit", AbiFunction::Terminate(abi_terminate)),
];

pub fn init_abis() {
    info!("abi_hello: 0x{:x}", abi_hello as usize);
    register_abi(SYS_HELLO, abi_hello as usize);
    info!("abi_putchar: 0x{:x}", abi_putchar as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    info!("abi_exit: 0x{:x}", abi_terminate as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);
    info!("abi_timespec: 0x{:x}", abi_timespec as usize);
    register_abi(SYS_TIMESPEC, abi_timespec as usize);
    info!("vfprintf: 0x{:x}", vfprintf as usize);
    register_abi(SYS_VFPRINTF, vfprintf as usize);
    info!("vsprintf: 0x{:x}", vsprintf as usize);
    register_abi(SYS_VSPRINTF, vsprintf as usize);
}

#[derive(Clone, Copy, Debug)]
pub enum AbiFunction {
    Hello(fn() -> ()),
    Putchar(fn(char) -> ()),
    Terminate(fn() -> !),
}

impl AbiFunction {
    pub fn from_name(name: &str) -> Option<Self> {
        for (n, f) in STR_TO_FUNC.iter() {
            if n == &name {
                return Some(*f);
            }
        }
        None
    }

    pub fn addr(&self) -> usize {
        match self {
            AbiFunction::Hello(f) => *f as usize,
            AbiFunction::Putchar(f) => *f as usize,
            AbiFunction::Terminate(f) => *f as usize,
        }
    }
}

fn register_abi(num: usize, handle: usize) {
    unsafe {
        ABI_TABLE[num] = handle;
    }
}

/// `SYS_HELLO: 1`
fn abi_hello() {
    print!("\x1b[34m");
    println!("[ABI:Hello] Hello, Apps!");
    print!("\x1b[0m");
}

/// `SYS_PUTCHAR: 2`
fn abi_putchar(c: char) {
    print!("\x1b[34m");
    print!("{c}");
    print!("\x1b[0m");
}

/// `SYS_TERMINATE: 3`
fn abi_terminate() -> ! {
    print!("\x1b[34m");
    println!("Bye");
    print!("\x1b[0m");

    exit(0);
}

#[repr(C)]
#[derive(Debug)]
struct TimeSpec {
    tv_sec: usize,
    tv_nsec: usize,
}

/// `SYS_TIMESPEC: 4`
fn abi_timespec(ts: *mut TimeSpec) {
    unsafe {
        let ts = &mut *ts;
        let now = monotonic_time();
        ts.tv_nsec = now.as_nanos() as usize;
        ts.tv_sec = now.as_secs() as usize;
        debug!("{:?}", ts);
    }
}

/// `SYS_VFPRINTF: 5`
#[no_mangle]
unsafe extern "C" fn vfprintf(str: *const c_char, args: VaList) -> c_int {
    let format = display(str, args);
    println!("\x1b[34m{}\x1b[0m", format);
    format.bytes_written()
}

/// `SYS_VSPRINTF: 6`
unsafe extern "C" fn vsprintf(out: *mut c_char, str: *const c_char, args: VaList) -> c_int {
    // 检查str是否为null
    if str.is_null() {
        return -1; // 返回一个错误代码
    }
    let format = display(str, args);
    copy_nonoverlapping(format.to_string().as_ptr(), out, format.to_string().len());
    format.bytes_written()
}
