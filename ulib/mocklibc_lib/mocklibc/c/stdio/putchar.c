#include <libc.h>
#include <stdio.h>

void putchar(char c)
{
    typedef int (*FnABI)(char c);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_PUTCHAR);
    FnABI func = (FnABI)(*abi_ptr);
    func(c);
}
