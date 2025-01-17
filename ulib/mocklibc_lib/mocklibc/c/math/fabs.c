// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>
#include <stdint.h>

double fabs(double x)
{
    union {
        double f;
        uint64_t i;
    } u = {x};
    u.i &= -1ULL / 2;
    return u.f;
}
