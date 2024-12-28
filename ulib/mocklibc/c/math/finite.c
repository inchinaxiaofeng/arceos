// NOTE: `Std C impl based on musl 1.2.5`
#define _GNU_SOURCE
#include <math.h>

int finite(double x)
{
    return isfinite(x);
}
