use axlog::info;
use axstd::{print, println, process::exit};

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
pub const SYS_TERMINATE: usize = 3;

pub static mut ABI_TABLE: [usize; 16] = [0; 16];

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

fn abi_terminate() -> ! {
    print!("\x1b[34m");
    println!("Bye");
    print!("\x1b[0m");

    exit(0);
}
