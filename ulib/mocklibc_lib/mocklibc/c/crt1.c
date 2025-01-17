#include "crt_arch.h"
#include <libc.h>

void terminate();

unsigned long volatile abi_entry = 0;
// extern unsigned long volatile abi_entry;
__attribute__((visibility("hidden"))) void _start_c(long *p)
{
    __asm__ volatile("mv %0, a7" : "=r"(abi_entry));
    int argc = p[0];
    char **argv = (void *)(p + 1);

    main(argc, argv);

    terminate();
}

void terminate()
{
    typedef void (*Fn)();
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_TERMINATE);
    Fn func = (Fn)(*abi_ptr);
    func();
}
