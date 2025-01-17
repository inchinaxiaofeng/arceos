// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"

double __math_invalid(double x)
{
    return (x - x) / (x - x);
}
