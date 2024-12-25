use axhal::time::monotonic_time;
use axlog::{debug, info};
use axstd::{
    io::stdin,
    print, println,
    process::exit,
    string::{String, ToString},
};
use core::{ffi::VaList, ptr::copy_nonoverlapping};
use cty::{c_char, c_int, size_t};
use printf_compat::output::display;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
pub const SYS_TERMINATE: usize = 3;
const SYS_TIMESPEC: usize = 4;
const SYS_VFPRINTF: usize = 5;
const SYS_VSNPRINTF: usize = 6;
const SYS_VFSCANF: usize = 7;

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
    info!("vsprintf: 0x{:x}", vsnprintf as usize);
    register_abi(SYS_VSNPRINTF, vsnprintf as usize);
    info!("vfscanf: 0x{:x}", vfscanf as usize);
    register_abi(SYS_VFSCANF, vfscanf as usize);
}

// TODO: 将后来的函数添加进去
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

/// `SYS_VSNPRINTF: 6`
unsafe extern "C" fn vsnprintf(
    out: *mut c_char,
    maxlen: size_t,
    str: *const c_char,
    args: VaList,
) -> c_int {
    // 检查str是否为null
    if str.is_null() {
        return -1; // 返回一个错误代码
    }
    // 创建格式化字符串
    let format = display(str, args);
    let output_string = format.to_string();
    let bytes_written = output_string.len();

    // 限制写入的字节数
    let len_to_copy = bytes_written.min(maxlen - 1); // 保留一个字节用于Null终止符
    copy_nonoverlapping(output_string.as_ptr(), out, len_to_copy);

    // 添加null终止符
    *out.add(len_to_copy) = 0;

    bytes_written as c_int
}

/// `SYS_VFSCANF: 7`
unsafe extern "C" fn vfscanf(str: *mut c_char, args: VaList) -> c_int {
    if str.is_null() {
        return -1;
    }

    let mut output: String = String::new();
    stdin()
        .read_line(&mut output)
        .expect("Failed to real line!");

    let output_string = output.to_string();

    // 读取
    copy_nonoverlapping(output_string.as_ptr(), str, output_string.len());
    0
}
