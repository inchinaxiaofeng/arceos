// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"
#include <math.h>

float lgammaf(float x)
{
    return __lgammaf_r(x, &__signgam);
}
