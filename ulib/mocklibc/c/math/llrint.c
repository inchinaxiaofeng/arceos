// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

/* uses LLONG_MAX > 2^53, see comments in lrint.c */

long long llrint(double x)
{
    return rint(x);
}
