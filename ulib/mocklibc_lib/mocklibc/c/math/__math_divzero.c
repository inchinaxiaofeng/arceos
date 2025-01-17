// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"

double __math_divzero(uint32_t sign)
{
    return fp_barrier(sign ? -1.0 : 1.0) / 0.0;
}
