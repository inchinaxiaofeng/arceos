#include "crt_arch.h"
#include <mocklibc.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

unsigned long volatile abi_entry = 0;

extern void __entry(long *p);
extern char **environ;

__attribute__((visibility("hidden"))) void _start(long *p)
{
    asm volatile("mv %0, a7" : "=r"(abi_entry));

    pthread_t musl_start;
    pthread_create(&musl_start, NULL, __entry, p);
    pthread_join(musl_start, NULL);
    // TODO :
    // 在static的条件下，将entry实装（就是另一个版本的start,或者在static的条件下条件编译这个函数，提供不同的实现）
    // __entry(p);
}

int __libc_start_main(int (*main)(), int argc, char **argv, void (*_init)(), void (*_fini)(),
                      void (*rtld_fini)())
{
    printf("main entry %p, argc @%p=%d, argv @%p\n", main, &argc, argc, argv);
    main(argc, argv, environ);
}

void terminate()
{
    typedef void (*Fn)();
    long *abi_ptr = (long *)(abi_entry + 8 * ABI_TERMINATE);
    Fn func = (Fn)(*abi_ptr);
    return func();
}
