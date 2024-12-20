#include <libc.h>
void abi_call(unsigned long entry, int abi_id, long arg)
{
    typedef void (*Fn)(long);
    long *abi_ptr = (long *)(entry + 8 * abi_id);
    Fn func = (Fn)(*abi_ptr);
    func(arg);
}
