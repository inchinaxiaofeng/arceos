#include <libc.h>
#include <stdarg.h>
#include <stdio.h>

// NOTE: C Std done.

int vsnprintf(char *restrict s, size_t n, const char *restrict fmt, va_list ap)
{
    int ret;

    typedef int (*FnABI)(char *, size_t, const char *, va_list);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_VSNPRINTF);
    FnABI func = (FnABI)(*abi_ptr);
    ret = func(s, n, fmt, ap);

    return ret;
}
