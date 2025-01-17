// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

double ldexp(double x, int n)
{
    return scalbn(x, n);
}
