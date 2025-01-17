// NOTE: `Std C impl based on musl 1.2.5`
#define _GNU_SOURCE
#include <math.h>

double significand(double x)
{
    return scalbn(x, -ilogb(x));
}
