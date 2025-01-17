#ifndef _LIBC_H
#define _LIBC_H

#define SYS_HELLO     1
#define SYS_PUTCHAR   2
#define SYS_TERMINATE 3
#define SYS_TIMESPEC  4
#define SYS_VFPRINTF  5
#define SYS_VSNPRINTF 6
#define SYS_VSCANF    7

// unsigned long volatile abi_entry = 0;
extern unsigned long volatile abi_entry;

extern int main(int, char **);

#endif
