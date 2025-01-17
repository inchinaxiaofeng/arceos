#include <libc.h>
#include <stdarg.h>
#include <stdio.h>

// NOTE: C Std done.
int printf(const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);

    typedef int (*FnABI)(const char *, va_list);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_VFPRINTF);
    FnABI func = (FnABI)(*abi_ptr);
    va_list *ap_ptr = &ap;
    ret = func(fmt, ap);

    va_end(ap);
    return ret;
}
