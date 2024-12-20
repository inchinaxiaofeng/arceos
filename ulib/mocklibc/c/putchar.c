#include <libc.h>
#include <stdio.h>

extern unsigned long volatile abi_entry;
#define SYS_PUTCHAR 2
void putchar(char c)
{
    abi_call(abi_entry, SYS_PUTCHAR, c - 0);
}
