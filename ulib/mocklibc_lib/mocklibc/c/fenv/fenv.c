#include <fenv.h>

/* Dummy functions for archs lacking fenv implementation */

int fetestexcept(int mask)
{
    return 0;
}
