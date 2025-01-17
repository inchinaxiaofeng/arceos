// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"

float __math_invalidf(float x)
{
    return (x - x) / (x - x);
}
