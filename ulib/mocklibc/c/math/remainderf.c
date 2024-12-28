// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

float remainderf(float x, float y)
{
    int q;
    return remquof(x, y, &q);
}

weak_alias(remainderf, dremf);
