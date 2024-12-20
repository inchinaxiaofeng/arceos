#include <libc.h>
#include <stdio.h>

extern unsigned long volatile abi_entry;
void putchar(char c)
{
    abi_call(abi_entry, SYS_PUTCHAR, c - 0);
}
