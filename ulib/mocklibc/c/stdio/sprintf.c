#include <libc.h>
#include <stdarg.h>
#include <stdio.h>

extern unsigned long volatile abi_entry;
// NOTE: C Std done.
int sprintf(char *restrict s, const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);

    typedef int (*FnABI)(char *, const char *, va_list);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_VSPRINTF);
    FnABI func = (FnABI)(*abi_ptr);
    va_list *ap_ptr = &ap;
    ret = func(s, fmt, ap);

    va_end(ap);
    return ret;
}
