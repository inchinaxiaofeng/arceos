#include <libc.h>
#include <limits.h>
#include <stdio.h>
#include <time.h>
extern unsigned long volatile abi_entry;
clock_t clock()
{
    struct timespec ts;
    struct timespec *ts_ptr = &ts;
    abi_call(abi_entry, SYS_TIMESPEC, (long)ts_ptr);

    if (ts.tv_sec > LONG_MAX / 1000000 || ts.tv_nsec / 1000 > LONG_MAX - 1000000 * ts.tv_sec)
        return -1;

    return ts.tv_sec * 1000000 + ts.tv_nsec / 1000;
}
