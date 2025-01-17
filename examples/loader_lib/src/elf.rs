use core::fmt;

use axlog::debug;
use elf::{abi::{EM_RISCV, ET_EXEC}, endian::LittleEndian, ElfBytes};

pub fn verify_elf_header(elf: &ElfBytes<LittleEndian>) -> Result<(), LoadError> {
    let header = elf.ehdr;

    // 1. TODO: 验证 ELF 魔数

    // 2. 验证文件类型
    if header.e_type != ET_EXEC {
        debug!("Not an executable file: {:?}", header.e_type);
        return Err(LoadError::NotExecutable);
    }

    // 3. 验证目标架构
    if header.e_machine != EM_RISCV {
        debug!("Wrong architecture: expected RISC-V, got {:?}", header.e_machine);
        return Err(LoadError::WrongArchitecture);
    }

    // 4. 验证程序头表是否存在
    if header.e_phnum == 0 {
        debug!("No program headers found");
        return Err(LoadError::NoSegments);
    }

    // 5. 验证 ELF 版本
    if header.version != EV_CURRENT {
        debug!("Invalid ELF version");
        return Err(LoadError::InvalidVersion);
    }

    // 6. 验证入口点是否有效
    if header.e_entry == 0 {
        debug!("Invalid entry point");
        return Err(LoadError::InvalidEntryPoint);
    }

    // 7. 验证程序头表偏移
    if header.e_phoff == 0 {
        debug!("Invalid program header offset");
        return Err(LoadError::InvalidProgramHeaderOffset);
    }

    Ok(())
}

// 扩展错误类型
#[derive(Debug)]
pub enum LoadError {
    InvalidMagic,
    NotExecutable,
    WrongArchitecture,
    NoSegments,
    InvalidVersion,
    InvalidEntryPoint,
    InvalidProgramHeaderOffset,
    SegmentOutOfBounds,
    BadAlignment,
    RelocationError,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Invalid ELF magic number"),
            Self::NotExecutable => write!(f, "File is not executable"),
            Self::WrongArchitecture => write!(f, "Wrong target architecture"),
            Self::NoSegments => write!(f, "No program segments found"),
            Self::InvalidVersion => write!(f, "Invalid ELF version"),
            Self::InvalidEntryPoint => write!(f, "Invalid entry point"),
            Self::InvalidProgramHeaderOffset => write!(f, "Invalid program header offset"),
            Self::SegmentOutOfBounds => write!(f, "Segment out of bounds"),
            Self::BadAlignment => write!(f, "Bad segment alignment"),
            Self::RelocationError => write!(f, "Relocation error"),
        }
    }
}

// ELF 常量定义（如果需要）
const ELFMAG0: u8 = 0x7f;
const ELFMAG1: u8 = b'E';
const ELFMAG2: u8 = b'L';
const ELFMAG3: u8 = b'F';
const EV_CURRENT: u32 = 1;