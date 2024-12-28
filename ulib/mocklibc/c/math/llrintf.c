// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

/* uses LLONG_MAX > 2^24, see comments in lrint.c */

long long llrintf(float x)
{
    return rintf(x);
}
