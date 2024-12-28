// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

float ldexpf(float x, int n)
{
    return scalbnf(x, n);
}
