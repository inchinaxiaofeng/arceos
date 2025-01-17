// NOTE: `Std C impl based on musl 1.2.5`
#include <math.h>
#include <stdint.h>

float fabsf(float x)
{
    union {
        float f;
        uint32_t i;
    } u = {x};
    u.i &= 0x7fffffff;
    return u.f;
}
