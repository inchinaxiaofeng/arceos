// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"
#include <math.h>

double lgamma(double x)
{
    return __lgamma_r(x, &__signgam);
}
