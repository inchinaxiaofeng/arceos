#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

/* 程序构建方法：
```
# 构建ImageHeader的方式：
echo -n -e "\x78\x56\x34\x12" > header.bin  # 魔数
echo -n -e "\x02\x00\x00\x00" >> header.bin  # 应用程序数量
echo -n -e "\x18\x00\x00\x22\xc0\xff\xff\xff" >> header.bin # 应用程序1头入口地址(hello_app)
echo -n -e "\x2e\x00\x00\x22\xc0\xff\xff\xff" >> header.bin # 应用程序2头入口地址(ebreak_app)

# 给每一个程序去构建的方式：
# 创建头部
echo -n -e "\x78\x56\x34\x12" > header.bin  # 魔数
echo -n -e "\x00\x00\x00\x00" >> header.bin  # 应用程序大小
echo -n -e "\x00\x00\x00\x22\xc0\xff\xff\xff" >> header.bin  # 入口地址
(第一个大小：12(x0c)地址入口:x28)
(第二个大小：8(x08)地址入口:x3e)

# 将头部和应用程序合并
dd if=header.bin of=apps.bin bs=1 count=16
dd if=hello_app.bin of=apps.bin bs=1 oflag=append conv=notrunc

# 构建应用程序（这里假定每个的大小都是48字节)
dd if=/dev/zero of=./empty_apps.bin bs=1 count=48
dd if=./apps.bin of=./empty_apps.bin conv=notrunc
mv empty_apps.bin apps.bin

# 受限于PFlash的大小为32M，所以这里与要求保持一致
dd if=/dev/zero of=./empty_apps.bin bs=1M count=32
dd if=./apps.bin of=./empty_apps.bin conv=notrunc
mv empty_apps.bin apps.bin
```
*/

/// `bin`的开始位置
const PLASH_START: usize = 0xffff_ffc0_2200_0000;

#[repr(C)]
struct AppHeader {
    magic_number: u32,  // 用于识别 4bytes
    app_size: u32,      // 应用程序大小 4bytes
    entry_point: usize, // 应用程序入口地址 8bytes
}

#[repr(C)]
struct ImageHeader {
    magic_number: u32, // 用于识别 4bytes
    app_count: u32,    // 4bytes
    apps: [usize; 2],  // 最多支持两个应用 8*2bytes
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let image_ptr = PLASH_START as *const ImageHeader;

    // 读取头部信息
    let image = unsafe { &*image_ptr };

    println!("Load payload ...");
    println!("Loading applications...");

    println!("image start {:?}", image_ptr);

    for i in 0..image.app_count {
        let app_header_ptr = image.apps[i as usize] as *const AppHeader;
        let app_header = unsafe { &*app_header_ptr };

        // 根据头部信息获取每个应用程序的起始地址和大小
        let apps_start = app_header.entry_point as *const u8;
        let apps_size = app_header.app_size as usize;

        println!(
            "header {:?} start {:?} size {}",
            app_header_ptr, apps_start, apps_size
        );

        // 读取应用程序代码
        let app_code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
        println!("App {} content: {:?}", i + 1, app_code);
    }

    println!("Loading complete!");
    println!("Load payload ok!");
}
