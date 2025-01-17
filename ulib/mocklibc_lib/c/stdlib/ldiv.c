// NOTE: `Std C impl based on musl 1.2.5`
#include <stdlib.h>

ldiv_t ldiv(long num, long den)
{
    return (ldiv_t){num / den, num % den};
}
