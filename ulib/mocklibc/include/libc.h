#ifndef _LIBC_H
#define _LIBC_H

#define SYS_HELLO     1
#define SYS_PUTCHAR   2
#define SYS_TERMINATE 3
#define SYS_TIMESPEC  4
#define SYS_PRINT     5

void abi_call(unsigned long entry, int abi_id, long arg);
extern int main(int, char **);

#endif
