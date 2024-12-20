#include <libc.h>
void abi_call(unsigned long entry, int abi_id, long arg)
{
    typedef void (*FnABI)(long);
    long *abi_ptr = (long *)(entry + 8 * abi_id);
    FnABI func = (FnABI)(*abi_ptr);
    func(arg);
}
