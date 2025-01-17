// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"
#include <float.h>

#if LDBL_MANT_DIG != DBL_MANT_DIG
long double __math_invalidl(long double x)
{
    return (x - x) / (x - x);
}
#endif
