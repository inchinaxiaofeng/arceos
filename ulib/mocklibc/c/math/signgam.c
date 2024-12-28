// NOTE: `Std C impl based on musl 1.2.5`
#include "libm.h"
#include <math.h>

int __signgam = 0;

weak_alias(__signgam, signgam);
