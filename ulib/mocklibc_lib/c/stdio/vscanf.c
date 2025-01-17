#include <libc.h>
#include <stdarg.h>
#include <stdio.h>

int vscanf(const char *restrict fmt, va_list ap)
{
    int ret;

    typedef int (*FnABI)(const char *, va_list);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_VSCANF);
    FnABI func = (FnABI)(*abi_ptr);
    ret = func(fmt, ap);

    return ret;
}

// TODO: weak_alias();
