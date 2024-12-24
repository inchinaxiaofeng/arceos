#ifndef _SYS_TYPES_H
#define _SYS_TYPES_H

#ifdef __cplusplus
extern "C" {
#endif // `__cplusplus`

// TODO: add the #if ...
// #if defined(_GNU_SOURCE) || defined(_BSD_SOURCE)
typedef unsigned char u_int8_t;
typedef unsigned short u_int16_t;
typedef unsigned u_int32_t;
typedef char *caddr_t;
typedef unsigned char u_char;
typedef unsigned short u_short, ushort;
typedef unsigned u_int, uint;
typedef unsigned long u_long, ulong;
typedef long long quad_t;
typedef unsigned long long u_quad_t;
// #include <endian.h>
// #include <sys/select.h>
// #endif

#ifdef __cplusplus
}
#endif // `__cplusplus`

#endif // `!_SYS_TYPES_H`
