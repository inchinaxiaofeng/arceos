// NOTE: `Std C impl based on musl 1.2.5`
#include <ctype.h>

/* nonsense function that should NEVER be used! */
int toascii(int c)
{
    return c & 0x7f;
}
