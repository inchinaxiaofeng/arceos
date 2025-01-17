// NOTE: `Std C impl based on musl 1.2.5`
#include <stdlib.h>

long labs(long a)
{
    return a > 0 ? a : -a;
}
