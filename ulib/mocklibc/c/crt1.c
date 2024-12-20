#include "crt_arch.h"
#include <libc.h>

unsigned long volatile abi_entry = 0;

void terminate();
hidden void _start_c(long *p)
{
    __asm__ volatile("mv %0, a7"       // 将 a7 的值移动到 abi_entry
                     : "=r"(abi_entry) // 输出操作数
    );
    int argc = p[0];
    char **argv = (void *)(p + 1);

    main(argc, argv);

    terminate();
}
#define SYS_TERMINATE 3
void terminate()
{
    typedef void (*FnABI)();
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_TERMINATE);
    FnABI func = (FnABI)(*abi_ptr);
    func();
}
