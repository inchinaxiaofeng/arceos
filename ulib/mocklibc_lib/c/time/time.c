#include "syscall.h"
#include <libc.h>
#include <time.h>

time_t time(time_t *t)
{
    struct timespec ts;
    struct timespec *ts_ptr = &ts;
    //    __clock_gettime(CLOCK_REALTIME, &ts);
    typedef int (*FnABI)(long);
    long *abi_ptr = (long *)(abi_entry + 8 * SYS_TIMESPEC);
    FnABI func = (FnABI)(*abi_ptr);
    func((long)ts_ptr);

    if (t)
        *t = ts.tv_sec;
    return ts.tv_sec;
}
