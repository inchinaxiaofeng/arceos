// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

double remainder(double x, double y)
{
    int q;
    return remquo(x, y, &q);
}

weak_alias(remainder, drem);
