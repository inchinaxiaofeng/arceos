use core::ptr::copy_nonoverlapping;

use crate::{
    linux_env::{
        axfs_ext::api::OpenFlags,
        linux_api::{
            api::process_api,
            link::{AT_FDCWD, FilePath, deal_with_path},
            utils::{UtilsError, deal_path},
        },
    },
    syscall::{
        DirEnt, DirEntType, SyscallError, SyscallResult, TimeSecs,
        ctypes::{Fcntl64Cmd, RenameFlags},
        syscall_fs::ctype::file::{FileDesc, new_fd},
    },
};
use alloc::string::ToString;
use axerrno::AxError;
use axfs::api::{
    Permissions, create_dir, metadata, remove_dir, remove_file, rename, set_current_dir,
};
use axlog::{debug, error, info, warn};
use axtask::current;

/// 功能:获取当前工作目录；
/// # Arguments
/// * `buf: *mut u8`, 一块缓存区,用于保存当前工作目录的字符串。当`buf`设为`NULL`,由系统来分配缓存区。
/// * `len: usize`, `buf`缓存区的大小。
/// # Return
/// 成功执行,则返回当前工作目录的字符串的指针。失败,则返回`NULL`。
/// 暂时:成功执行,则返回当前工作目录的字符串的指针 `as isize`。失败,返回0。
///
/// TODO: 当前写法存在问题,`cwd`应当是各个进程独立的,而这里修改的是整个`fs`的目录
pub fn syscall_getcwd(args: [usize; 6]) -> SyscallResult {
    let buf = args[0] as *mut u8;
    let len = args[1];
    info!("Into syscall_getcwd. buf: {}, len: {}", buf as usize, len);
    let cwd = axfs::api::current_dir().unwrap();

    // TODO: 如果buf为NULL,则系统分配缓存区
    // ```
    // let process = current_process();
    // let process_inner = process.inner.lock();
    // if buf.is_null() {
    //     buf = allocate_buffer(cwd.len());   // 分配缓存区 allocate_buffer
    // }

    let cwd = cwd.as_bytes();

    if len >= cwd.len() {
        // FIX: 这里需要判断能不能访问
        unsafe {
            core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.len());
        }
        Ok(buf as isize)
    } else {
        info!("getcwd: buf size is too small");
        Err(SyscallError::ERANGE)
    }
}

/// 功能:创建目录；
/// # Arguments
/// * `dirfd: usize`, 要创建的目录所在的目录的文件描述符。
/// * `path: *const u8`, 要创建的目录的名称。如果`path`是相对路径,则它是相对于`dirfd`目录而言的。如果`path`是相对路径,且`dirfd`的值为`AT_FDCWD`,则它是相对于当前路径而言的。如果`path`是绝对路径,则`dirfd`被忽略。
/// * `mode: u32`, 文件的所有权描述。详见`man 7 inode`。
/// 返回值:成功执行,返回0。失败,返回-1。
// FIX: 不建议对一个目录使用判断是否存在的方法。
// 在对目录进行查找的时候，会报错: `[  1.266962 0:2 fatfs::dir:139] Is a directory`
// 这个查找之后似乎不对`ArceOS`代码和`fatfs`的代码进行修改是没有办法解决的。
// 需要找到新的方法用来判断目录的存在与否。
// 这里可以通过直接判断create_dir的返回值进行判断。
// 同时也可以对已经存在的情况进行判断。
// 当一个目录被lookup函数使用时，就会发生报错。
// 我们没有办法排除在已经有了对应目录的情况下再次创建目录导致的报错。
// 本质上是只要使用到lookup就会出现问题
pub fn syscall_mkdirat(args: [usize; 6]) -> SyscallResult {
    let dir_fd = args[0];
    let path = args[1] as *const u8;
    let mode = args[2] as u32;

    let path = match deal_path(dir_fd, Some(path), true) {
        Ok(ans) => ans,
        Err(e) => match e {
            UtilsError::StrTooLong => return Err(SyscallError::ENAMETOOLONG),
            UtilsError::CannotAcce | UtilsError::NULL => return Err(SyscallError::EFAULT),
            UtilsError::OutOfTable | UtilsError::NoEntryInTable => return Err(SyscallError::EBADF),
            UtilsError::InvalidArg => return Err(SyscallError::ENOTDIR),
            UtilsError::PanicMe => {
                panic!("{:?}", e);
            }
            _ => {
                panic!("{:?}", e);
            }
        },
    };

    debug!(
        "Into syscall_mkdirat. dirfd: {}, path: {:?}, mode: {}",
        dir_fd,
        path.path(),
        mode
    );

    match metadata(path.path()) {
        Ok(_) => {
            debug!("Err: EEXIST pathname already exists (not necessarily as a directory).");
            return Err(SyscallError::EEXIST);
        }
        Err(e) => match e {
            AxError::NotFound => {}
            _ => {
                warn!("Unexpect errno catched {:?}", e);
                return Err(SyscallError::EPERM);
            }
        },
    }

    // 只要文件夹存在就返回0
    match create_dir(path.path()) {
        Ok(_) => Ok(0),
        Err(e) => match e {
            AxError::AlreadyExists => {
                debug!("Err: EEXIST pathname already exists (not necessarily as a directory).");
                Err(SyscallError::EEXIST)
            }
            AxError::NotFound => {
                debug!(
                    "Err: ENOENT A directory component in pathname does not exist or is a dangling symbolic link."
                );
                Err(SyscallError::ENOENT)
            }
            _ => {
                warn!("Unexpect errno catched {:?}", e);
                Err(SyscallError::EPERM)
            }
        },
    }
}

/// 功能:切换工作目录；
/// # Arguments
/// * `path: *const u8`, 需要切换到的目录。
/// # Return
/// 成功执行:返回0。失败, 返回-1。
pub fn syscall_chdir(args: [usize; 6]) -> SyscallResult {
    let path = args[0] as *const u8;
    // 从path中读取字符串
    let path = match deal_path(AT_FDCWD, Some(path), true) {
        Ok(path) => path,
        Err(e) => match e {
            UtilsError::StrTooLong => return Err(SyscallError::ENAMETOOLONG),
            UtilsError::CannotAcce | UtilsError::NULL => return Err(SyscallError::EFAULT),
            UtilsError::OutOfTable => return Err(SyscallError::EBADF),
            UtilsError::PanicMe => {
                panic!("{:?}", e);
            }
            _ => {
                panic!("{:?}", e);
            }
        },
    };

    debug!("Into syscall_chdir. path: {:?}", path.path());
    match metadata(path.path()) {
        Ok(metadata) => {
            let perm = metadata.permissions();
            if !perm.contains(Permissions::OWNER_READ) {
                return Err(SyscallError::EACCES);
            }
            if !metadata.is_dir() {
                return Err(SyscallError::ENOTDIR);
            }
            set_current_dir(path.path()).unwrap();
            Ok(0)
        }
        Err(e) => match e {
            AxError::NotFound => Err(SyscallError::ENOENT),
            AxError::NotADirectory => Err(SyscallError::ENOTDIR),
            _ => {
                panic!("{:?}", e);
            }
        },
    }
}

/// To get the `dirent` structures from the directory referred to by the open file descriptor `fd` into the buffer
/// # Arguments
/// * `fd: usize`, the file descriptor of the directory to be read
/// * `buf: *mut u8`, the buffer to store the `dirent` structures
/// * `len: usize`, the size of the buffer
///
/// # Return
/// * On success, the number of bytes read is returned. On end of directory, 0 is returned.
/// * On error, -1 is returned.
pub fn syscall_getdents64(args: [usize; 6]) -> SyscallResult {
    let fd = args[0];
    let buf = args[1] as *mut u8;
    let len = args[2];
    let path = match deal_path(fd, None, false) {
        Ok(path) => path,
        Err(e) => match e {
            UtilsError::CannotAcce | UtilsError::NULL => return Err(SyscallError::EFAULT),
            UtilsError::StrTooLong => return Err(SyscallError::ENAMETOOLONG),
            UtilsError::OutOfTable => return Err(SyscallError::EBADF),
            UtilsError::StrEmpty | UtilsError::NoEntryInTable => return Err(SyscallError::ENOENT),
            UtilsError::PanicMe => {
                panic!("{:?}", e);
            }
            _ => {
                panic!("{:?}", e);
            }
        },
    };

    // ```
    // let process = process_api();
    // // 注意是否分配地址
    // let start: VirtAddr = (buf as usize).into();
    // let end = start + len;
    //     if process.manual_alloc_range_for_lazy(start, end).is_err() {
    //         return Err(SyscallError::EFAULT);
    //     }

    if len < DirEnt::fixed_size() {
        return Err(SyscallError::EINVAL);
    }
    // ```
    // let entry_id_from = unsafe { (*(buf as *const DirEnt)).d_off };
    // error!("entry_id_from: {}", entry_id_from);
    // 先获取buffer里面最后一个长度
    let mut all_offset = 0; // 记录上一次调用时进行到的目录项距离文件夹开始时的偏移量
    let mut buf_offset = 0; // 记录当前buf里面的目录项的指针偏移量
    loop {
        if buf_offset + DirEnt::fixed_size() >= len {
            break;
        }
        let dir_ent = unsafe { *(buf.add(buf_offset) as *const DirEnt) };
        if dir_ent.d_reclen == 0 {
            break;
        }
        buf_offset += dir_ent.d_reclen as usize;
        // all_offset = dir_ent.d_off; // 记录最新的 offset
        if all_offset < dir_ent.d_off {
            all_offset = dir_ent.d_off;
        } else {
            break;
        }
    }

    let buf = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let dir_iter = axfs::api::read_dir(path.path()).unwrap();
    let mut count = 0; // buf中已经写入的字节数
    let mut offset: u64 = 0; // 当前目录项在文件夹中的偏移
    for entry in dir_iter {
        let entry = entry.unwrap();
        let mut name = entry.file_name();
        name.push('\0');
        let name = name.as_bytes();
        let name_len = name.len();
        let file_type = entry.file_type();
        let entry_size = DirEnt::fixed_size() + name_len + 1;

        // buf不够大，写不下新的entry
        if count + entry_size + DirEnt::fixed_size() + 1 > len {
            debug!("buf not big enough");
            break;
        }
        offset += entry_size as u64;
        if offset <= all_offset {
            continue;
        }
        // 转换为DirEnt
        let dirent: &mut DirEnt = unsafe { &mut *(buf.as_mut_ptr().add(count) as *mut DirEnt) };
        // 设置定长部分
        if file_type.is_dir() {
            dirent.set_fixed_part(1, offset, entry_size, DirEntType::Dir);
        } else if file_type.is_file() {
            dirent.set_fixed_part(1, offset, entry_size, DirEntType::Reg);
        } else {
            dirent.set_fixed_part(1, offset, entry_size, DirEntType::Unknown);
        }

        // 写入文件名
        unsafe { copy_nonoverlapping(name.as_ptr(), dirent.d_name.as_mut_ptr(), name_len) };

        count += entry_size;
    }

    // 为了保证下一次访问的时候边界是存在的，因此需要手动写入一个空的目录项
    if count != 0 && count + DirEnt::fixed_size() <= len {
        // 转换为DirEnt
        let dirent: &mut DirEnt = unsafe { &mut *(buf.as_mut_ptr().add(count) as *mut DirEnt) };
        // 设置定长部分
        dirent.set_fixed_part(1, offset, DirEnt::fixed_size(), DirEntType::Reg);
        count += DirEnt::fixed_size();
        return Ok((count - DirEnt::fixed_size()) as isize);
    }
    Ok(count as isize)
}

/// 276
/// 重命名文件或目录
// `todo!`
// 1. 权限检查
// 调用进程必须对源目录和目标目录都有写权限,才能完成重命名。
// 2. 目录和文件在同一个文件系统
// 如果目录和文件不在同一个文件系统,重命名会失败。`renameat2`不能跨文件系统重命名。
// 3. 源文件不是目标目录的子目录
// 如果源文件是目标目录的子孙目录,也会导致重命名失败。不能将目录重命名到自己的子目录中。
// 4. 目标名称不存在
// 目标文件名在目标目录下必须不存在,否则会失败。
// 5. 源文件被打开
// 如果源文件正被进程打开,默认情况下重命名也会失败。可以通过添加`RENAME_EXCHANGE`标志位实现原子交换。
// 6. 目录不是挂载点
// 如果源目录是一个挂载点,也不允许重命名。
/// # Arguments
/// * `old_dirfd: usize`, 旧文件所在的目录的文件描述符。
/// * `old_path: *const u8`, 旧文件的名称。如果`old_path`是相对路径,则它是相对于`old_dirfd`目录而言的。如果`old_path`是相对路径,且`old_dirfd`的值为`AT_FDCWD`,则它是相对于当前路径而言的。如果`old_path`是绝对路径,则`old_dirfd`被忽略。
/// * `new_dirfd: usize`, 新文件所在的目录的文件描述符。
/// * `new_path: *const u8`, 新文件的名称。如果`new_path`是相对路径,则它是相对于`new_dirfd`目录而言的。如果`new_path`是相对路径,且`new_dirfd`的值为`AT_FDCWD`,则它是相对于当前路径而言的。如果`new_path`是绝对路径,则`new_dirfd`被忽略。
/// * `flags: usize`, 重命名的标志位。目前只支持`RENAME_NOREPLACE`、`RENAME_EXCHANGE`和`RENAME_WHITEOUT`。
pub fn syscall_renameat2(args: [usize; 6]) -> SyscallResult {
    let old_dirfd = args[0];
    let _old_path = args[1] as *const u8;
    let new_dirfd = args[2];
    let _new_path = args[3] as *const u8;
    let flags = args[4];
    let old_path = deal_with_path(old_dirfd, Some(_old_path), false).unwrap();
    let new_path = deal_with_path(new_dirfd, Some(_new_path), false).unwrap();

    let proc_path = FilePath::new("/proc").unwrap();
    if old_path.start_with(&proc_path) || new_path.start_with(&proc_path) {
        debug!(
            "EPERM  RENAME_WHITEOUT was specified in flags, but the caller does not have the CAP_MKNOD capability."
        );
        return Err(SyscallError::EPERM);
    }

    let flags = if let Some(ans) = RenameFlags::from_bits(flags as u32) {
        ans
    } else {
        debug!("EINVAL An invalid flag was specified in flags.");
        return Err(SyscallError::EINVAL);
    };

    if flags.contains(RenameFlags::NOREPLACE) {
        if flags.contains(RenameFlags::EXCHANGE) {
            debug!("EINVAL Both RENAME_NOREPLACE and RENAME_EXCHANGE were specified in flags.");
            return Err(SyscallError::EINVAL);
        }
        match metadata(new_path.path()) {
            Ok(_) => {
                debug!("EEXIST flags contains RENAME_NOREPLACE and newpath already exists.");
                return Err(SyscallError::EEXIST);
            }
            Err(e) => {
                debug!("{}", e);
            }
        }
    }

    if !flags.contains(RenameFlags::EXCHANGE) {
        // 此时不是交换，而是移动，那么需要
        match (metadata(new_path.path()), metadata(old_path.path())) {
            (Ok(new_metadata), Ok(old_metadata)) => {
                if old_metadata.is_dir() ^ new_metadata.is_dir() {
                    if old_metadata.is_dir() {
                        debug!(
                            "ENOTDIR oldpath is relative and olddirfd is a file descriptor referring to a file other than a directory; or similar for newpath and newdirfd"
                        );
                        return Err(SyscallError::ENOTDIR);
                    }
                    debug!(
                        "EISDIR newpath is an existing directory, but oldpath is not a directory."
                    );
                    return Err(SyscallError::EISDIR);
                }
            }
            (Err(new_err), _) => match new_err {
                AxError::NotFound => {}
                _ => {
                    panic!("{}", new_err)
                }
            },
            (Ok(_), Err(old_err)) => match old_err {
                AxError::NotFound => {
                    // FIXME: 检查这里，重构整个函数
                    // panic!("CHECKME");
                }
                _ => {
                    panic!("{}", old_err);
                }
            },
        }
    } else if flags.contains(RenameFlags::WHITEOUT) {
        debug!("EINVAL Both RENAME_WHITEOUT and RENAME_EXCHANGE were specified in flags.");
        return Err(SyscallError::EINVAL);
    }

    if flags.contains(RenameFlags::EXCHANGE) {
        if let Err(e) = metadata(new_path.path()) {
            match e {
                AxError::NotFound => {
                    debug!("ENOENT flags contains RENAME_EXCHANGE and newpath does not exist.");
                    return Err(SyscallError::ENOENT);
                }
                _ => {
                    panic!("{}", e);
                }
            }
        }
    }

    // 做实际重命名操作
    match metadata(old_path.path()) {
        Ok(_) => {}
        Err(e) => match e {
            AxError::NotFound => {
                debug!(
                    "ENOENT The link named by oldpath does not exist; or, a directory component in newpath does not exist; or, oldpath or newpath is an empty string."
                );
                return Err(SyscallError::ENOENT);
            }
            _ => {
                panic!("{}", e)
            }
        },
    }

    if old_path.path() == new_path.path() {
        // 相同文件不用改
        return Ok(0);
    }
    if !flags.contains(RenameFlags::EXCHANGE) {
        // 此时若存在新文件，默认是没有 NOREPLACE 的
        match metadata(new_path.path()) {
            // 当新文件已经存在，先删掉新文件
            Ok(new_metadata) => {
                if new_metadata.is_dir() {
                    if let Err(err) = remove_dir(new_path.path()) {
                        error!("error: {:?}", err);
                        return Err(SyscallError::EPERM);
                    }
                } else if new_metadata.is_file() {
                    if let Err(err) = remove_file(new_path.path()) {
                        error!("error: {:?}", err);
                        return Err(SyscallError::EPERM);
                    }
                }
            }
            Err(e) => match e {
                // 新文件不存在时，直接执行后续操作
                AxError::NotFound => {}
                _ => panic!("{}", e),
            },
        };

        if let Err(err) = rename(old_path.path(), new_path.path()) {
            error!("error: {:?}", err);
            return Err(SyscallError::EPERM);
        }
    } else {
        // 当前不支持交换
        axlog::warn!("renameat2 exchange not implemented");
        return Err(SyscallError::EPERM);
    }
    Ok(0)
}

/// # Arguments
/// * `fd: usize`
/// * `cmd: usize`
/// * `arg: usize`
pub fn syscall_fcntl64(args: [usize; 6]) -> SyscallResult {
    let fd = args[0];
    let cmd = args[1];
    let arg = args[2];
    let process = process_api();
    let mut fd_table = process.fd_manager.fd_table.lock();

    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return Err(SyscallError::EBADF);
    }
    let file = fd_table[fd].clone().unwrap();
    info!("fd: {}, cmd: {}", fd, cmd);
    match Fcntl64Cmd::try_from(cmd) {
        Ok(Fcntl64Cmd::F_DUPFD) => {
            let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
                fd
            } else {
                // 文件描述符达到上限了
                return Err(SyscallError::EMFILE);
            };
            fd_table[new_fd] = fd_table[fd].clone();
            Ok(new_fd as isize)
        }
        Ok(Fcntl64Cmd::F_GETFD) => {
            if file.get_status().contains(OpenFlags::CLOEXEC) {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Ok(Fcntl64Cmd::F_SETFD) => {
            if file.set_close_on_exec((arg & 1) != 0) {
                Ok(0)
            } else {
                Err(SyscallError::EINVAL)
            }
        }
        Ok(Fcntl64Cmd::F_GETFL) => Ok(file.get_status().bits() as isize),
        Ok(Fcntl64Cmd::F_SETFL) => {
            if let Some(flags) = OpenFlags::from_bits(arg as u32) {
                if file.set_status(flags) {
                    return Ok(0);
                }
            }
            Err(SyscallError::EINVAL)
        }
        Ok(Fcntl64Cmd::F_DUPFD_CLOEXEC) => {
            let new_fd = if let Ok(fd) = process.alloc_fd(&mut fd_table) {
                fd
            } else {
                // 文件描述符达到上限了
                return Err(SyscallError::EMFILE);
            };

            if file.set_close_on_exec((arg & 1) != 0) {
                fd_table[new_fd] = fd_table[fd].clone();
                Ok(new_fd as isize)
            } else {
                Err(SyscallError::EINVAL)
            }
        }
        _ => Err(SyscallError::EINVAL),
    }
}

// FIX: `fatfs`文件系统不支持设置权限，会直接当作`0o755`返回。
/// 53
/// 修改文件权限
/// mode: `0o777`, 3位八进制数字
/// path为相对路径:
///     1. 若`dir_fd`为`AT_FDCWD`,则相对于当前工作目录
///     2. 若`dir_fd`为`AT_FDCWD`以外的值,则相对于`dir_fd`所指的目录
/// path为绝对路径:
///     忽视`dir_fd`,直接根据`path`访问
/// # Arguments
/// * `dir_fd: usize`, 目录的文件描述符
/// * `path: *const u8`, 文件的路径
/// * `mode: usize`, 文件的权限
pub fn syscall_fchmodat(args: [usize; 6]) -> SyscallResult {
    let dir_fd = args[0];
    let path = args[1] as *const u8;
    let mode = args[2];

    let mode = if let Some(ans) = Permissions::from_bits(mode as u16) {
        ans
    } else {
        return Err(SyscallError::EINVAL);
    };

    let file_path = match deal_path(dir_fd, Some(path), false) {
        Ok(path) => path,
        Err(e) => match e {
            UtilsError::CannotAcce | UtilsError::NULL => return Err(SyscallError::EFAULT),
            UtilsError::StrTooLong => return Err(SyscallError::ENAMETOOLONG),
            UtilsError::OutOfTable => return Err(SyscallError::EBADF),
            UtilsError::StrEmpty | UtilsError::NoEntryInTable => return Err(SyscallError::ENOENT),
            UtilsError::PanicMe => {
                panic!("{:?}", e);
            }
            _ => {
                panic!("{:?}", e);
            }
        },
    };

    //```
    // FIXME: 这里是有问题的，因为现有的文件系统尚不支持设置权限，在这里实验了一下
    //    let file_io = match FDM.fd_table.lock().get(dir_fd) {
    //        Some(Some(f)) => f.clone(),
    //        _ => return Err(SyscallError::EBADF),
    //    };
    //    if let Some(file_desc) = file_io.as_any().downcast_ref::<FileDesc>() {
    //        warn!("Get perm, before {:?}", file_desc.get_perm());
    //        file_desc.set_perm(mode);
    //        warn!("Get perm, after  {:?}", file_desc.get_perm());
    //    } else {
    //        warn!("fdsaf");
    //    }

    match metadata(file_path.path()) {
        Ok(mut meta) => {
            meta.set_permissions(mode);
            Ok(0)
        }
        Err(e) => match e {
            AxError::NotFound => return Err(SyscallError::ENOENT),
            _ => {
                panic!("{:?}", e);
            }
        },
    }
}

/// 修改指定文件描述符的文件权限
/// mode: 0o777, 3位八进制数字
/// # Arguments
/// * `fd: usize`, 文件的文件描述符
/// * `mode: usize`, 文件的权限
pub fn syscall_fchmod(args: [usize; 6]) -> SyscallResult {
    let fd = args[0];
    let mode = args[1];
    let process = process_api();

    // 将模式转换为 Permissions 类型
    let mode = if let Some(ans) = Permissions::from_bits(mode as u16) {
        ans
    } else {
        return Err(SyscallError::EINVAL);
    };

    // 获取文件描述符对应的文件
    let file_io = match process.fd_manager.fd_table.lock().get(fd) {
        Some(Some(f)) => f.clone(),
        _ => return Err(SyscallError::EBADF), // 文件描述符无效
    };

    if let Some(file_desc) = file_io.as_any().downcast_ref::<FileDesc>() {
        // 修改文件权限
        match file_desc.file.lock().metadata() {
            Ok(mut metadata) => {
                metadata.set_permissions(mode);
                Ok(0)
            }
            Err(e) => {
                warn!("Unexpected err catched. {:?}", e);
                Err(SyscallError::EINVAL)
            }
        }
    } else {
        Err(SyscallError::EINVAL) // 文件描述福类型错误
    }
}

// TODO: 添加支持与文档
pub fn syscall_fchownat(_args: [usize; 6]) -> SyscallResult {
    unimplemented!();
}

// TODO: 添加支持与文档
pub fn syscall_fchown(_args: [usize; 6]) -> SyscallResult {
    unimplemented!();
}

/// 48
/// 获取文件权限
/// 类似上面的`fchmodat`
///        The mode specifies the accessibility `check(s)` to be performed,
///        and is either the value F_OK, or a mask consisting of the `bitwise`
///        OR of one or more of R_OK, W_OK, and X_OK. F_OK tests for the
///        existence of the file. R_OK, W_OK, and X_OK test whether the
///        file exists and grants read, write, and execute permissions,
///        respectively.
/// 0: F_OK, 1: X_OK, 2: W_OK, 4: R_OK
/// # Arguments
/// * `dir_fd: usize`, 目录的文件描述符
/// * `path: *const u8`, 文件的路径
/// * `mode: usize`, 文件的权限
pub fn syscall_faccessat(args: [usize; 6]) -> SyscallResult {
    let dir_fd = args[0];
    let path = args[1] as *const u8;
    let mode = args[2];
    // `todo`: 有问题,实际上需要考虑当前进程对应的用户`UID`和文件拥有者之间的关系
    // 现在一律当作root用户处理
    let file_path = match deal_path(dir_fd, Some(path), false) {
        Ok(path) => path,
        Err(e) => match e {
            UtilsError::StrTooLong => return Err(SyscallError::ENAMETOOLONG),
            UtilsError::CannotAcce | UtilsError::NULL => return Err(SyscallError::EFAULT),
            UtilsError::OutOfTable | UtilsError::NoEntryInTable => return Err(SyscallError::EBADF),
            UtilsError::PanicMe => {
                panic!("panic {:?}", e);
            }
            _ => {
                panic!("{:?}", e);
            }
        },
    };

    let mode = if let Some(ans) = Permissions::from_bits(mode as u16) {
        ans
    } else {
        debug!("Err: An invalid flag was specified in mode");
        return Err(SyscallError::EINVAL);
    };

    match metadata(file_path.path()) {
        Ok(metadata) => {
            if mode.is_empty() {
                // F_OK
                Ok(0)
            } else {
                let perm = metadata.permissions();
                // 逐位对比
                if (mode.contains(Permissions::OWNER_EXEC)
                    && !perm.contains(Permissions::OWNER_EXEC))
                    || (mode.contains(Permissions::OWNER_WRITE)
                        && !perm.contains(Permissions::OWNER_WRITE))
                    || (mode.contains(Permissions::OWNER_READ)
                        && !perm.contains(Permissions::OWNER_READ))
                {
                    return Err(SyscallError::EACCES);
                }
                Ok(0)
            }
        }
        Err(e) => match e {
            AxError::NotFound => return Err(SyscallError::ENOENT),
            _ => {
                panic!("{:?}", e);
            }
        },
    }
}

/// 29
/// 执行各种设备相关的控制功能
/// `todo`: 未实现
/// # Arguments
/// * `fd: usize`, 文件描述符
/// * `request: usize`, 控制命令
/// * `argp: *mut usize`, 参数
pub fn syscall_ioctl(args: [usize; 6]) -> SyscallResult {
    let fd = args[0];
    let request = args[1];
    let argp = args[2];
    let process = process_api();
    info!("try the current");
    current();
    info!("pass the current");
    let fd_table = process.fd_manager.fd_table.lock();
    debug!("fd: {}, request: {}, argp: {}", fd, request, argp);
    if fd >= fd_table.len() {
        debug!("fd {} is out of range", fd);
        return Err(SyscallError::EBADF);
    }
    if fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return Err(SyscallError::EBADF);
    }
    //    if process.manual_alloc_for_lazy(argp.into()).is_err() {
    //        return Err(SyscallError::EFAULT); // 地址不合法
    //    }

    let file = fd_table[fd].clone().unwrap();
    match file.ioctl(request, argp) {
        Ok(ret) => Ok(ret),
        Err(_) => Ok(0),
    }
}

// FIX: 待测试
/// 88
/// 用于修改文件或目录的时间戳(timestamp)
/// 如果 `fir_fd < 0`,它和 `path` 共同决定要找的文件；
/// 如果 `fir_fd >=0`,它就是文件对应的 `fd`
/// # Arguments
/// * `dir_fd: usize`, 目录的文件描述符
/// * `path: *const u8`, 文件的路径
/// * `times: *const TimeSecs`, 时间戳
/// * `flags: usize`, 选项
pub fn syscall_utimensat(args: [usize; 6]) -> SyscallResult {
    let dir_fd = args[0];
    let path = args[1] as *const u8;
    let times = args[2] as *const TimeSecs;
    let _flags = args[3];
    let process = process_api();
    info!("dir_fd: {}, path: {}", dir_fd as usize, path as usize);
    if dir_fd != AT_FDCWD && (dir_fd as isize) < 0 {
        return Err(SyscallError::EBADF); // 错误的文件描述符
    }

    if dir_fd == AT_FDCWD
    //        && process
    //            .manual_alloc_for_lazy((path as usize).into())
    //            .is_err()
    {
        return Err(SyscallError::EFAULT); // 地址不合法
    }
    // 需要设置的时间
    let (new_atime, new_mtime) = if times.is_null() {
        (TimeSecs::now(), TimeSecs::now())
    } else {
        //        if process.manual_alloc_type_for_lazy(times).is_err() {
        //            return Err(SyscallError::EFAULT);
        //        }
        unsafe { (*times, *(times.add(1))) } //  注意传入的TimeVal中 sec和nsec都是usize, 但TimeValue中nsec是u32
    };
    // 感觉以下仿照maturin的实现不太合理,并没有真的把时间写给文件,只是写给了一个新建的临时的fd
    if (dir_fd as isize) > 0 {
        // let file = process_inner.fd_manager.fd_table[dir_fd].clone();
        // if !file.unwrap().lock().set_time(new_atime, new_mtime) {
        //     error!("Set time failed: unknown reason.");
        //     return ErrorNo::EPERM as isize;
        // }
        let fd_table = process.fd_manager.fd_table.lock();
        if dir_fd > fd_table.len() || fd_table[dir_fd].is_none() {
            return Err(SyscallError::EBADF);
        }
        if let Some(file) = fd_table[dir_fd].as_ref() {
            if let Some(fat_file) = file.as_any().downcast_ref::<FileDesc>() {
                // if !fat_file.set_time(new_atime, new_mtime) {
                //     error!("Set time failed: unknown reason.");
                //     return ErrorNo::EPERM as isize;
                // }
                fat_file.stat.lock().atime.set_as_utime(&new_atime);
                fat_file.stat.lock().mtime.set_as_utime(&new_mtime);
            } else {
                return Err(SyscallError::EPERM);
            }
        }
        Ok(0)
    } else {
        let file_path = deal_with_path(dir_fd, Some(path), false).unwrap();
        if !axfs::api::path_exists(file_path.path()) {
            error!("Set time failed: file {} doesn't exist!", file_path.path());
            if !axfs::api::path_exists(file_path.dir().unwrap()) {
                return Err(SyscallError::ENOTDIR);
            } else {
                return Err(SyscallError::ENOENT);
            }
        }
        let file = new_fd(file_path.path().to_string(), 0.into()).unwrap();
        file.stat.lock().atime.set_as_utime(&new_atime);
        file.stat.lock().mtime.set_as_utime(&new_mtime);
        Ok(0)
    }
}
