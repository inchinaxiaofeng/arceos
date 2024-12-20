#ifndef _LIBC_H
#define _LIBC_H

#define SYS_PUTCHAR 2

#define hidden __attribute__((visibility("hidden")))

void abi_call(unsigned long entry, int abi_id, long arg);
extern int main(int, char **);

#endif
