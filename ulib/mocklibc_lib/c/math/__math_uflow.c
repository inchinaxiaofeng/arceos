// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"

double __math_uflow(uint32_t sign)
{
    return __math_xflow(sign, 0x1p-767);
}
