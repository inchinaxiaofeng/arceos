// NOTE: `Std C impl based on musl 1.2.5`
#include <ctype.h>
#undef isascii

int isascii(int c)
{
    return !(c & ~0x7f);
}
