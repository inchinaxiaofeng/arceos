// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

long lround(double x)
{
    return round(x);
}
