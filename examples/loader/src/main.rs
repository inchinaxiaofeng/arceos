#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

/* 程序构建方法：
```
# 创建头部
echo -n -e "\x78\x56\x34\x12" > header.bin  # 魔数
echo -n -e "\x20\x00\x00\x00" >> header.bin  # 应用程序大小（32字节）
echo -n -e "\x10\x00\x00\x22\xc0\xff\xff\xff" >> header.bin  # 入口地址（可以设为0）

# 将头部和应用程序合并
dd if=header.bin of=apps.bin bs=1 count=16
dd if=hello_app.bin of=apps.bin bs=1 oflag=append conv=notrunc

# 原本的逻辑，程序的大小为32，头文件大小为16
# 受限于PFlash的大小为32M，所以这里与要求保持一致
dd if=/dev/zero of=./empty_apps.bin bs=1M count=32
dd if=./apps.bin of=./empty_apps.bin conv=notrunc
mv empty_apps.bin apps.bin
```
*/

/// `bin`的开始位置
const PLASH_START: usize = 0xffff_ffc0_2200_0000;

#[repr(C)]
struct ImageHeader {
    magic_number: u32,
    app_size: u32,
    entry_point: usize,
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let header_ptr = PLASH_START as *const ImageHeader;

    // 读取头部信息
    let header = unsafe { &*header_ptr };

    // 获取应用程序的大小
    let apps_size = header.app_size as usize;

    // 方式一，不推荐
    let apps_start = (PLASH_START + core::mem::size_of::<ImageHeader>()) as *const u8;

    // 方式二，直接获得头结构，获得起始地址
    let apps_start = header.entry_point as *const u8;

    println!("Load payload ...");

    let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
    println!("content: {:?}: ", code);

    println!("Load payload ok!");
}
