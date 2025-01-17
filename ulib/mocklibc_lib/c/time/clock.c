#include <libc.h>
#include <limits.h>
#include <stdio.h>
#include <time.h>

clock_t clock()
{
    struct timespec ts;
    struct timespec *ts_ptr = &ts;

    typedef int (*FnABI)(long);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_TIMESPEC);
    FnABI func = (FnABI)(*abi_ptr);
    func((long)ts_ptr);

    if (ts.tv_sec > LONG_MAX / 1000000 || ts.tv_nsec / 1000 > LONG_MAX - 1000000 * ts.tv_sec)
        return -1;

    return ts.tv_sec * 1000000 + ts.tv_nsec / 1000;
}
