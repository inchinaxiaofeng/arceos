// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"

float __math_uflowf(uint32_t sign)
{
    return __math_xflowf(sign, 0x1p-95f);
}
