// NOTE: `Std C impl based on musl 1.2.5`
#include <stdlib.h>

int abs(int a)
{
    return a > 0 ? a : -a;
}
