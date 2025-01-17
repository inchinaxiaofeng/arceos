// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

/* uses LONG_MAX > 2^24, see comments in lrint.c */

long lrintf(float x)
{
    return rintf(x);
}
