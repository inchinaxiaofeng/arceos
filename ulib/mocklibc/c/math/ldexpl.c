// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

long double ldexpl(long double x, int n)
{
    return scalbnl(x, n);
}
