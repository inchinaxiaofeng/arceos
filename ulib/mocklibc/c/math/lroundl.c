// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>

long lroundl(long double x)
{
    return roundl(x);
}
