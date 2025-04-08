#ifndef _LIBC_H
#define _LIBC_H

// `0-10提供ArceOS相关ABI调用`
#define ABI_NOIMPL         0
#define ABI_INIT_SCHEDULER 1
#define ABI_TERMINATE      2
#define ABI_CHECKPOINT     3
// `stdio`
#define ABI_TIMESPEC 11
// `pthread`
#define ABI_PTHREAD_CREATE        20
#define ABI_PTHREAD_JOIN          21
#define ABI_PTHREAD_EXIT          22
#define ABI_PTHREAD_SELF          23
#define ABI_PTHREAD_MUTEX_INIT    24
#define ABI_PTHREAD_MUTEX_LOCK    25
#define ABI_PTHREAD_MUTEX_UNLOCK  26
#define ABI_PTHREAD_MUTEX_DESTORY 27
// `malloc`
#define ABI_MALLOC  40
#define ABI_CALLOC  41
#define ABI_REALLOC 42
#define ABI_FREE    43
// `unistd`
#define ABI_SLEEP 50
// `abi_syscall`
#define ABI_SYSCALL0 60
#define ABI_SYSCALL1 61
#define ABI_SYSCALL2 62
#define ABI_SYSCALL3 63
#define ABI_SYSCALL4 64
#define ABI_SYSCALL5 65
#define ABI_SYSCALL6 66
// `rt_float`的实现
// Arithmetic functions[100, 119]
#define ABI_RT_ADDSF3 100
#define ABI_RT_ADDDF3 101
#define ABI_RT_ADDTF3 102
#define ABI_RT_ADDXF3 103

#define ABI_RT_SUBSF3 104
#define ABI_RT_SUBDF3 105
#define ABI_RT_SUBTF3 106
#define ABI_RT_SUBXF3 107

#define ABI_RT_MULSF3 108
#define ABI_RT_MULDF3 109
#define ABI_RT_MULTF3 110
#define ABI_RT_MULXF3 111

#define ABI_RT_DIVSF3 112
#define ABI_RT_DIVDF3 113
#define ABI_RT_DIVTF3 114
#define ABI_RT_DIVXF3 115

#define ABI_RT_NEGSF2 116
#define ABI_RT_NEGDF2 117
#define ABI_RT_NEGTF2 118
#define ABI_RT_NEGXF2 119
// Conversion functions[120, 187]
#define ABI_RT_EXTENDSFDF2 120
#define ABI_RT_EXTENDSFTF2 121
#define ABI_RT_EXTENDSFXF2 122
#define ABI_RT_EXTENDDFTF2 123
#define ABI_RT_EXTENDDFXF2 124

#define ABI_RT_TRUNCXFDF2 125
#define ABI_RT_TRUNCTFDF2 126
#define ABI_RT_TRUNCXFSF2 127
#define ABI_RT_TRUNCTFSF2 128
#define ABI_RT_TRUNCDFSF2 129

#define ABI_RT_FIXSFSI 130
#define ABI_RT_FIXDFSI 131
#define ABI_RT_FIXTFSI 132
#define ABI_RT_FIXXFSI 133

#define ABI_RT_FIXSFDI 134
#define ABI_RT_FIXDFDI 135
#define ABI_RT_FIXTFDI 136
#define ABI_RT_FIXXFDI 137

#define ABI_RT_FIXSFTI 138
#define ABI_RT_FIXDFTI 139
#define ABI_RT_FIXTFTI 140
#define ABI_RT_FIXXFTI 141

#define ABI_RT_FIXUNSSFSI 142
#define ABI_RT_FIXUNSDFSI 143
#define ABI_RT_FIXUNSTFSI 144
#define ABI_RT_FIXUNSXFSI 145

#define ABI_RT_FIXUNSSFDI 146
#define ABI_RT_FIXUNSDFDI 147
#define ABI_RT_FIXUNSTFDI 148
#define ABI_RT_FIXUNSXFDI 149

#define ABI_RT_FIXUNSSFTI 150
#define ABI_RT_FIXUNSDFTI 151
#define ABI_RT_FIXUNSTFTI 152
#define ABI_RT_FIXUNSXFTI 153

#define ABI_RT_FLOATSISF 154
#define ABI_RT_FLOATSIDF 155
#define ABI_RT_FLOATSITF 156
#define ABI_RT_FLOATSIXF 157

#define ABI_RT_FLOATDISF 158
#define ABI_RT_FLOATDIDF 159
#define ABI_RT_FLOATDITF 160
#define ABI_RT_FLOATDIXF 161

#define ABI_RT_FLOATTISF 162
#define ABI_RT_FLOATTIDF 163
#define ABI_RT_FLOATTITF 164
#define ABI_RT_FLOATTIXF 165

#define ABI_RT_FLOATUNSISF 166
#define ABI_RT_FLOATUNSIDF 167
#define ABI_RT_FLOATUNSITF 168
#define ABI_RT_FLOATUNSIXF 169

#define ABI_RT_FLOATUNDISF 170
#define ABI_RT_FLOATUNDIDF 171
#define ABI_RT_FLOATUNDITF 172
#define ABI_RT_FLOATUNDIXF 173

#define ABI_RT_FLOATUNTISF 174
#define ABI_RT_FLOATUNTIDF 175
#define ABI_RT_FLOATUNTITF 176
#define ABI_RT_FLOATUNTIXF 177

#define ABI_RT_FIXSFBITINT 178
#define ABI_RT_FIXDFBITINT 179
#define ABI_RT_FIXXFBITINT 180
#define ABI_RT_FIXTFBITINT 181

#define ABI_RT_FLOATBITINTSF 182
#define ABI_RT_FLOATBITINTDF 183
#define ABI_RT_FLOATBITINTXF 184
#define ABI_RT_FLOATBITINTTF 185
#define ABI_RT_FLOATBITINTHF 186
#define ABI_RT_FLOATBITINTBF 187

// Comparison functions
#define ABI_RT_CMPSF2 188
#define ABI_RT_CMPDF2 189
#define ABI_RT_CMPTF2 190

#define ABI_RT_UNORDSF2 191
#define ABI_RT_UNORDDF2 192
#define ABI_RT_UNORDTF2 193

#define ABI_RT_EQSF2 194
#define ABI_RT_EQDF2 195
#define ABI_RT_EQTF2 196

#define ABI_RT_NESF2 197
#define ABI_RT_NEDF2 198
#define ABI_RT_NETF2 199

#define ABI_RT_GESF2 200
#define ABI_RT_GEDF2 201
#define ABI_RT_GETF2 202

#define ABI_RT_LTSF2 203
#define ABI_RT_LTDF2 204
#define ABI_RT_LTTF2 205

#define ABI_RT_LESF2 206
#define ABI_RT_LEDF2 207
#define ABI_RT_LETF2 208

#define ABI_RT_GTSF2 209
#define ABI_RT_GTDF2 210
#define ABI_RT_GTTF2 211

// Other floating-point functions
#define ABI_RT_POWISF2 212
#define ABI_RT_POWIDF2 213
#define ABI_RT_POWITF2 214
#define ABI_RT_POWIXF2 215

#define ABI_RT_MULSC3 216
#define ABI_RT_MULDC3 217
#define ABI_RT_MULTC3 218
#define ABI_RT_MULXC3 219

#define ABI_RT_DIVSC3 220
#define ABI_RT_DIVDC3 221
#define ABI_RT_DIVTC3 222
#define ABI_RT_DIVXC3 223

// -------------

#define ABI_RT_CLZSI2 271
#define ABI_RT_CLZDI2 272
#define ABI_RT_CLZTI2 273

#define ABI_RT_BSWAPSI2 285
#define ABI_RT_BSWAPDI2 286

extern unsigned long volatile abi_entry;

#define __DEBUG__

// 调试程序的可变参数宏定义
#ifdef __DEBUG__
#include <stdio.h>
#define DEBUG_PRINT(...) printf(__VA_ARGS__);
#else
#define DEBUG_PRINT(...)
#endif

#define NOIMPL_STR(str)                                   \
    DEBUG_PRINT(str)                                      \
    typedef int (*FnABI)();                               \
    long *abi_ptr = (long *)(abi_entry + 8 * ABI_NOIMPL); \
    FnABI func = (FnABI)(*abi_ptr);                       \
    func();

#define ASM_CHECKPOINT(val)                \
    __asm__ volatile("nop\n"               \
                     "nop\n"               \
                     "addi zero,zero,%0\n" \
                     "nop\n"               \
                     "nop\n"               \
                     :                     \
                     : "i"(val));

#define CHECKPOINT                                                                \
    typedef int (*FnABI_CheckPoint)();                                            \
    long *abi_ptr_check_point = (long *)(abi_entry + 8 * ABI_CHECKPOINT);         \
    FnABI_CheckPoint func_check_point = (FnABI_CheckPoint)(*abi_ptr_check_point); \
    func_check_point();

#include <stdarg.h>
extern int main(int, char **);

int __libc_start_main(int (*)(), int, char **, void (*)(), void (*)(), void (*)());
// void mock_start_main(long *p);
void terminate();

#endif
