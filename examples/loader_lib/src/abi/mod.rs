pub mod mem;
mod noimpl;
mod syscall;
pub mod thread;

use axhal::time::monotonic_time;
use axlog::{debug, info};
use axstd::process::exit;
use compiler_builtins::{
    float::{
        add::*,
        cmp::{
            __eqdf2, __eqsf2, __eqtf2, __gedf2, __gesf2, __getf2, __gtdf2, __gtsf2, __gttf2,
            __ledf2, __lesf2, __letf2, __ltdf2, __ltsf2, __lttf2, __nedf2, __nesf2, __netf2,
            __unorddf2, __unordsf2, __unordtf2,
        },
        conv::{
            __fixdfdi, __fixdfsi, __fixdfti, __fixsfdi, __fixsfsi, __fixsfti, __fixtfdi, __fixtfsi,
            __fixtfti, __fixunsdfdi, __fixunsdfsi, __fixunsdfti, __fixunssfdi, __fixunssfsi,
            __fixunssfti, __fixunstfdi, __fixunstfsi, __fixunstfti, __floatdidf, __floatdisf,
            __floatditf, __floatsidf, __floatsisf, __floatsitf, __floattidf, __floattisf,
            __floattitf, __floatundidf, __floatundisf, __floatunditf, __floatunsidf, __floatunsisf,
            __floatunsitf, __floatuntidf, __floatuntisf, __floatuntitf,
        },
        div::*,
        extend::{__extenddftf2, __extendsfdf2, __extendsftf2},
        mul::*,
        pow::{__powidf2, __powisf2, __powitf2},
        sub::*,
        trunc::{__truncdfsf2, __trunctfdf2, __trunctfsf2},
    },
    int::{
        bswap::{__bswapdi2, __bswapsi2},
        leading_zeros::{__clzdi2, __clzsi2, __clzti2},
    },
};

use axtask::init_scheduler;
use mem::*;
use noimpl::abi_noimpl;
use syscall::*;
use thread::*;

// `0-10提供ArceOS相关ABI调用`
const ABI_NOIMPL: usize = 0;
const ABI_INIT_SCHEDULER: usize = 1;
pub const ABI_TERMINATE: usize = 2;
const ABI_CHECKPOINT: usize = 3;
// `stdio`
const ABI_TIMESPEC: usize = 11;
// `pthread`
const ABI_PTHREAD_CREATE: usize = 20;
const ABI_PTHREAD_JOIN: usize = 21;
const ABI_PTHREAD_EXIT: usize = 22;
const ABI_PTHREAD_SELF: usize = 23;
const ABI_PTHREAD_MUTEX_INIT: usize = 24;
const ABI_PTHREAD_MUTEX_LOCK: usize = 25;
const ABI_PTHREAD_MUTEX_UNLOCK: usize = 26;
const ABI_PTHREAD_MUTEX_DESTORY: usize = 27;
// `malloc`
const ABI_MALLOC: usize = 40;
const ABI_CALLOC: usize = 41;
const ABI_REALLOC: usize = 42;
const ABI_FREE: usize = 43;
// `unistd`
const ABI_SLEEP: usize = 50;
// `syscall`
const ABI_SYSCALL0: usize = 60;
const ABI_SYSCALL1: usize = 61;
const ABI_SYSCALL2: usize = 62;
const ABI_SYSCALL3: usize = 63;
const ABI_SYSCALL4: usize = 64;
const ABI_SYSCALL5: usize = 65;
const ABI_SYSCALL6: usize = 66;

// `rt_float` 的实现
// Arithmetic functions[100, 119]
const ABI_RT_ADDSF3: usize = 100;
const ABI_RT_ADDDF3: usize = 101;
const ABI_RT_ADDTF3: usize = 102;
const ABI_RT_ADDXF3: usize = 103;

const ABI_RT_SUBSF3: usize = 104;
const ABI_RT_SUBDF3: usize = 105;
const ABI_RT_SUBTF3: usize = 106;
const ABI_RT_SUBXF3: usize = 107;

const ABI_RT_MULSF3: usize = 108;
const ABI_RT_MULDF3: usize = 109;
const ABI_RT_MULTF3: usize = 110;
const ABI_RT_MULXF3: usize = 111;

const ABI_RT_DIVSF3: usize = 112;
const ABI_RT_DIVDF3: usize = 113;
const ABI_RT_DIVTF3: usize = 114;
const ABI_RT_DIVXF3: usize = 115;

const ABI_RT_NEGSF2: usize = 116;
const ABI_RT_NEGDF2: usize = 117;
const ABI_RT_NEGTF2: usize = 118;
const ABI_RT_NEGXF2: usize = 119;

// Conversion functions[120, 187]
const ABI_RT_EXTENDSFDF2: usize = 120;
const ABI_RT_EXTENDSFTF2: usize = 121;
const ABI_RT_EXTENDSFXF2: usize = 122;
const ABI_RT_EXTENDDFTF2: usize = 123;
const ABI_RT_EXTENDDFXF2: usize = 124;

const ABI_RT_TRUNCXFDF2: usize = 125;
const ABI_RT_TRUNCTFDF2: usize = 126;
const ABI_RT_TRUNCXFSF2: usize = 127;
const ABI_RT_TRUNCTFSF2: usize = 128;
const ABI_RT_TRUNCDFSF2: usize = 129;

const ABI_RT_FIXSFSI: usize = 130;
const ABI_RT_FIXDFSI: usize = 131;
const ABI_RT_FIXTFSI: usize = 132;
const ABI_RT_FIXXFSI: usize = 133;

const ABI_RT_FIXSFDI: usize = 134;
const ABI_RT_FIXDFDI: usize = 135;
const ABI_RT_FIXTFDI: usize = 136;
const ABI_RT_FIXXFDI: usize = 137;

const ABI_RT_FIXSFTI: usize = 138;
const ABI_RT_FIXDFTI: usize = 139;
const ABI_RT_FIXTFTI: usize = 140;
const ABI_RT_FIXXFTI: usize = 141;

const ABI_RT_FIXUNSSFSI: usize = 142;
const ABI_RT_FIXUNSDFSI: usize = 143;
const ABI_RT_FIXUNSTFSI: usize = 144;
const ABI_RT_FIXUNSXFSI: usize = 145;

const ABI_RT_FIXUNSSFDI: usize = 146;
const ABI_RT_FIXUNSDFDI: usize = 147;
const ABI_RT_FIXUNSTFDI: usize = 148;
const ABI_RT_FIXUNSXFDI: usize = 149;

const ABI_RT_FIXUNSSFTI: usize = 150;
const ABI_RT_FIXUNSDFTI: usize = 151;
const ABI_RT_FIXUNSTFTI: usize = 152;
const ABI_RT_FIXUNSXFTI: usize = 153;

const ABI_RT_FLOATSISF: usize = 154;
const ABI_RT_FLOATSIDF: usize = 155;
const ABI_RT_FLOATSITF: usize = 156;
const ABI_RT_FLOATSIXF: usize = 157;

const ABI_RT_FLOATDISF: usize = 158;
const ABI_RT_FLOATDIDF: usize = 159;
const ABI_RT_FLOATDITF: usize = 160;
const ABI_RT_FLOATDIXF: usize = 161;

const ABI_RT_FLOATTISF: usize = 162;
const ABI_RT_FLOATTIDF: usize = 163;
const ABI_RT_FLOATTITF: usize = 164;
const ABI_RT_FLOATTIXF: usize = 165;

const ABI_RT_FLOATUNSISF: usize = 166;
const ABI_RT_FLOATUNSIDF: usize = 167;
const ABI_RT_FLOATUNSITF: usize = 168;
const ABI_RT_FLOATUNSIXF: usize = 169;

const ABI_RT_FLOATUNDISF: usize = 170;
const ABI_RT_FLOATUNDIDF: usize = 171;
const ABI_RT_FLOATUNDITF: usize = 172;
const ABI_RT_FLOATUNDIXF: usize = 173;

const ABI_RT_FLOATUNTISF: usize = 174;
const ABI_RT_FLOATUNTIDF: usize = 175;
const ABI_RT_FLOATUNTITF: usize = 176;
const ABI_RT_FLOATUNTIXF: usize = 177;

const ABI_RT_FIXSFBITINT: usize = 178;
const ABI_RT_FIXDFBITINT: usize = 179;
const ABI_RT_FIXXFBITINT: usize = 180;
const ABI_RT_FIXTFBITINT: usize = 181;

const ABI_RT_FLOATBITINTSF: usize = 182;
const ABI_RT_FLOATBITINTDF: usize = 183;
const ABI_RT_FLOATBITINTXF: usize = 184;
const ABI_RT_FLOATBITINTTF: usize = 185;
const ABI_RT_FLOATBITINTHF: usize = 186;
const ABI_RT_FLOATBITINTBF: usize = 187;

// Comparison functions [180, 211]
const ABI_RT_CMPSF2: usize = 188;
const ABI_RT_CMPDF2: usize = 189;
const ABI_RT_CMPTF2: usize = 190;

const ABI_RT_UNORDSF2: usize = 191;
const ABI_RT_UNORDDF2: usize = 192;
const ABI_RT_UNORDTF2: usize = 193;

const ABI_RT_EQSF2: usize = 194;
const ABI_RT_EQDF2: usize = 195;
const ABI_RT_EQTF2: usize = 196;

const ABI_RT_NESF2: usize = 197;
const ABI_RT_NEDF2: usize = 198;
const ABI_RT_NETF2: usize = 199;

const ABI_RT_GESF2: usize = 200;
const ABI_RT_GEDF2: usize = 201;
const ABI_RT_GETF2: usize = 202;

const ABI_RT_LTSF2: usize = 203;
const ABI_RT_LTDF2: usize = 204;
const ABI_RT_LTTF2: usize = 205;

const ABI_RT_LESF2: usize = 206;
const ABI_RT_LEDF2: usize = 207;
const ABI_RT_LETF2: usize = 208;

const ABI_RT_GTSF2: usize = 209;
const ABI_RT_GTDF2: usize = 210;
const ABI_RT_GTTF2: usize = 211;

// Other floating-point functions [212, 223]
const ABI_RT_POWISF2: usize = 212;
const ABI_RT_POWIDF2: usize = 213;
const ABI_RT_POWITF2: usize = 214;
const ABI_RT_POWIXF2: usize = 215;

const ABI_RT_MULSC3: usize = 216;
const ABI_RT_MULDC3: usize = 217;
const ABI_RT_MULTC3: usize = 218;
const ABI_RT_MULXC3: usize = 219;

const ABI_RT_DIVSC3: usize = 220;
const ABI_RT_DIVDC3: usize = 221;
const ABI_RT_DIVTC3: usize = 222;
const ABI_RT_DIVXC3: usize = 223;

// `rt_integer` 的实现
// Arithmetic functions[230, 256]
const _ABI_RT_ASHLSI3: usize = 230;
const _ABI_RT_ASHLDI3: usize = 231;
const _ABI_RT_ASHLTI3: usize = 231;

const _ABI_RT_ASHRSI3: usize = 232;
const _ABI_RT_ASHRDI3: usize = 233;
const _ABI_RT_ASHRTI3: usize = 234;

const _ABI_RT_DIVSI3: usize = 235;
const _ABI_RT_DIVDI3: usize = 236;
const _ABI_RT_DIVTI3: usize = 237;

const _ABI_RT_LSHRSI3: usize = 238;
const _ABI_RT_LSHRDI3: usize = 239;
const _ABI_RT_LSHRTI3: usize = 240;

const _ABI_RT_MODSI3: usize = 241;
const _ABI_RT_MODDI3: usize = 242;
const _ABI_RT_MODTI3: usize = 243;

const _ABI_RT_MULSI3: usize = 244;
const _ABI_RT_MULDI3: usize = 245;
const _ABI_RT_MULTI3: usize = 246;

const _ABI_RT_NEGDI2: usize = 247;
const _ABI_RT_NEGTI2: usize = 248;

const _ABI_RT_UDIVSI3: usize = 249;
const _ABI_RT_UDIVDI3: usize = 250;
const _ABI_RT_UDIVTI3: usize = 251;

const _ABI_RT_UDIVMODDI4: usize = 252;
const _ABI_RT_UDIVMODTI4: usize = 253;

const _ABI_RT_UMODSI3: usize = 254;
const _ABI_RT_UMODDI3: usize = 255;
const _ABI_RT_UMODTI3: usize = 256;

// Comparison functions[257, 260]
const _ABI_RT_CMPDI2: usize = 257;
const _ABI_RT_CMPTI2: usize = 258;

const _ABI_RT_UCMPDI2: usize = 259;
const _ABI_RT_UCMPTI2: usize = 260;

// Trapping Arithmetic functions[261, 270]
const _ABI_RT_ABSVSI2: usize = 261;
const _ABI_RT_ABSVDI2: usize = 262;

const _ABI_RT_ADDVSI3: usize = 263;
const _ABI_RT_ADDVDI3: usize = 264;

const _ABI_RT_MULVSI3: usize = 265;
const _ABI_RT_MULVDI3: usize = 266;

const _ABI_RT_NEGVSI2: usize = 267;
const _ABI_RT_NEGVDI2: usize = 268;

const _ABI_RT_SUBVSI3: usize = 269;
const _ABI_RT_SUBVDI3: usize = 270;

// Bit operations functions[271, 286]
const ABI_RT_CLZSI2: usize = 271;
const ABI_RT_CLZDI2: usize = 272;
const ABI_RT_CLZTI2: usize = 273;

const _ABI_RT_CTZSI2: usize = 274;
const _ABI_RT_CTZDI2: usize = 275;
const _ABI_RT_CTZTI2: usize = 276;

const _ABI_RT_FFSDI2: usize = 277;
const _ABI_RT_FFSTI2: usize = 278;

const _ABI_RT_PARITYSI2: usize = 279;
const _ABI_RT_PARITYDI2: usize = 280;
const _ABI_RT_PARITYTI2: usize = 281;

const _ABI_RT_POPCOUNTSI2: usize = 282;
const _ABI_RT_POPCOUNTDI2: usize = 283;
const _ABI_RT_POPCOUNTTI2: usize = 284;

const ABI_RT_BSWAPSI2: usize = 285;
const ABI_RT_BSWAPDI2: usize = 286;

// Bit-precise integer arithmetic functions[287, 288]
const _ABI_RT_MULBITINT3: usize = 287;

const _ABI_RT_DIVMODBITINT4: usize = 288;

pub static mut ABI_TABLE: [usize; 300] = [ABI_NOIMPL; 300];

pub fn init_abis() {
    register_abi("noimpl", ABI_NOIMPL, abi_noimpl as usize);
    register_abi("init", ABI_INIT_SCHEDULER, abi_init_scheduler as usize);
    register_abi("exit", ABI_TERMINATE, abi_terminate as usize);
    register_abi("exit", ABI_CHECKPOINT, abi_checkpoint as usize);

    register_abi("timespec", ABI_TIMESPEC, abi_timespec as usize);

    register_abi(
        "pthread_create",
        ABI_PTHREAD_CREATE,
        abi_pthread_create as usize,
    );
    register_abi("pthread_join", ABI_PTHREAD_JOIN, abi_pthread_join as usize);
    register_abi("pthread_exit", ABI_PTHREAD_EXIT, abi_pthread_exit as usize);
    register_abi("pthread_self", ABI_PTHREAD_SELF, abi_pthread_self as usize);
    register_abi(
        "pthread_mutex_init",
        ABI_PTHREAD_MUTEX_INIT,
        abi_pthread_mutex_init as usize,
    );
    register_abi(
        "pthread_mutex_lock",
        ABI_PTHREAD_MUTEX_LOCK,
        abi_pthread_mutex_lock as usize,
    );
    register_abi(
        "pthread_mutex_unlock",
        ABI_PTHREAD_MUTEX_UNLOCK,
        abi_pthread_mutex_unlock as usize,
    );
    register_abi(
        "pthread_mutex_destroy",
        ABI_PTHREAD_MUTEX_DESTORY,
        abi_pthread_mutex_destroy as usize,
    );

    register_abi("malloc", ABI_MALLOC, abi_malloc as usize);
    register_abi("calloc", ABI_CALLOC, abi_calloc as usize);
    register_abi("realloc", ABI_REALLOC, abi_realloc as usize);
    register_abi("free", ABI_FREE, abi_free as usize);

    register_abi("sleep", ABI_SLEEP, abi_sleep as usize);

    register_abi("syscall0", ABI_SYSCALL0, abi_syscall0 as usize);
    register_abi("syscall1", ABI_SYSCALL1, abi_syscall1 as usize);
    register_abi("syscall2", ABI_SYSCALL2, abi_syscall2 as usize);
    register_abi("syscall3", ABI_SYSCALL3, abi_syscall3 as usize);
    register_abi("syscall4", ABI_SYSCALL4, abi_syscall4 as usize);
    register_abi("syscall5", ABI_SYSCALL5, abi_syscall5 as usize);
    register_abi("syscall6", ABI_SYSCALL6, abi_syscall6 as usize);

    // `rt_float`的实现
    // Arithmetic functions[100, 119]
    register_abi("rt_addsf3", ABI_RT_ADDSF3, __addsf3 as usize);
    register_abi("rt_adddf3", ABI_RT_ADDDF3, __adddf3 as usize);
    register_abi("rt_addtf3", ABI_RT_ADDTF3, __addtf3 as usize);
    register_abi("rt_addxf3", ABI_RT_ADDXF3, __addtf3 as usize);

    register_abi("rt_subsf3", ABI_RT_SUBSF3, __subsf3 as usize);
    register_abi("rt_subdf3", ABI_RT_SUBDF3, __subdf3 as usize);
    register_abi("rt_subtf3", ABI_RT_SUBTF3, __subtf3 as usize);
    register_abi("rt_subxf3", ABI_RT_SUBXF3, __subtf3 as usize);

    register_abi("rt_mulsf3", ABI_RT_MULSF3, __mulsf3 as usize);
    register_abi("rt_muldf3", ABI_RT_MULDF3, __muldf3 as usize);
    register_abi("rt_multf3", ABI_RT_MULTF3, __multf3 as usize);
    register_abi("rt_mulxf3", ABI_RT_MULXF3, __multf3 as usize);

    register_abi("rt_divsf3", ABI_RT_DIVSF3, __divsf3 as usize);
    register_abi("rt_divdf3", ABI_RT_DIVDF3, __divdf3 as usize);
    register_abi("rt_divtf3", ABI_RT_DIVTF3, __divtf3 as usize);
    register_abi("rt_divxf3", ABI_RT_DIVXF3, __divtf3 as usize);

    register_abi("rt_negsf2", ABI_RT_NEGSF2, abi_noimpl as usize);
    register_abi("rt_negdf2", ABI_RT_NEGDF2, abi_noimpl as usize);
    register_abi("rt_negtf2", ABI_RT_NEGTF2, abi_noimpl as usize);
    register_abi("rt_negxf2", ABI_RT_NEGXF2, abi_noimpl as usize);
    // Conversion functions[120, 187]
    register_abi("rt_extendsfdf2", ABI_RT_EXTENDSFDF2, __extendsfdf2 as usize);
    register_abi("rt_extendsftf2", ABI_RT_EXTENDSFTF2, __extendsftf2 as usize);
    register_abi("rt_extendsfxf2", ABI_RT_EXTENDSFXF2, __extendsftf2 as usize);
    register_abi("rt_extenddftf2", ABI_RT_EXTENDDFTF2, __extenddftf2 as usize);
    register_abi("rt_extenddfxf2", ABI_RT_EXTENDDFXF2, __extenddftf2 as usize);

    register_abi("rt_truncxfdf2", ABI_RT_TRUNCXFDF2, __trunctfdf2 as usize);
    register_abi("rt_trunctfdf2", ABI_RT_TRUNCTFDF2, __trunctfdf2 as usize);
    register_abi("rt_truncxfsf2", ABI_RT_TRUNCXFSF2, __trunctfsf2 as usize);
    register_abi("rt_trunctfsf2", ABI_RT_TRUNCTFSF2, __trunctfsf2 as usize);
    register_abi("rt_truncdfsf2", ABI_RT_TRUNCDFSF2, __truncdfsf2 as usize);

    register_abi("rt_fixsfsi", ABI_RT_FIXSFSI, __fixsfsi as usize);
    register_abi("rt_fixdfsi", ABI_RT_FIXDFSI, __fixdfsi as usize);
    register_abi("rt_fixtfsi", ABI_RT_FIXTFSI, __fixtfsi as usize);
    register_abi("rt_fixxfsi", ABI_RT_FIXXFSI, __fixtfsi as usize);

    register_abi("rt_fixsfdi", ABI_RT_FIXSFDI, __fixsfdi as usize);
    register_abi("rt_fixdfdi", ABI_RT_FIXDFDI, __fixdfdi as usize);
    register_abi("rt_fixtfdi", ABI_RT_FIXTFDI, __fixtfdi as usize);
    register_abi("rt_fixxfdi", ABI_RT_FIXXFDI, __fixtfdi as usize);

    register_abi("rt_fixsfti", ABI_RT_FIXSFTI, __fixsfti as usize);
    register_abi("rt_fixdfti", ABI_RT_FIXDFTI, __fixdfti as usize);
    register_abi("rt_fixtfti", ABI_RT_FIXTFTI, __fixtfti as usize);
    register_abi("rt_fixxfti", ABI_RT_FIXXFTI, __fixtfti as usize);

    register_abi("rt_fixunssfsi", ABI_RT_FIXUNSSFSI, __fixunssfsi as usize);
    register_abi("rt_fixunsdfsi", ABI_RT_FIXUNSDFSI, __fixunsdfsi as usize);
    register_abi("rt_fixunstfsi", ABI_RT_FIXUNSTFSI, __fixunstfsi as usize);
    register_abi("rt_fixunsxfsi", ABI_RT_FIXUNSXFSI, __fixunstfsi as usize);

    register_abi("rt_fixunssfdi", ABI_RT_FIXUNSSFDI, __fixunssfdi as usize);
    register_abi("rt_fixunsdfdi", ABI_RT_FIXUNSDFDI, __fixunsdfdi as usize);
    register_abi("rt_fixunstfdi", ABI_RT_FIXUNSTFDI, __fixunstfdi as usize);
    register_abi("rt_fixunsxfdi", ABI_RT_FIXUNSXFDI, __fixunstfdi as usize);

    register_abi("rt_fixunssfti", ABI_RT_FIXUNSSFTI, __fixunssfti as usize);
    register_abi("rt_fixunsdfti", ABI_RT_FIXUNSDFTI, __fixunsdfti as usize);
    register_abi("rt_fixunstfti", ABI_RT_FIXUNSTFTI, __fixunstfti as usize);
    register_abi("rt_fixunsxfti", ABI_RT_FIXUNSXFTI, __fixunstfti as usize);

    register_abi("rt_floatsisf", ABI_RT_FLOATSISF, __floatsisf as usize);
    register_abi("rt_floatsidf", ABI_RT_FLOATSIDF, __floatsidf as usize);
    register_abi("rt_floatsitf", ABI_RT_FLOATSITF, __floatsitf as usize);
    register_abi("rt_floatsixf", ABI_RT_FLOATSIXF, __floatsitf as usize);

    register_abi("rt_floatdisf", ABI_RT_FLOATDISF, __floatdisf as usize);
    register_abi("rt_floatdidf", ABI_RT_FLOATDIDF, __floatdidf as usize);
    register_abi("rt_floatditf", ABI_RT_FLOATDITF, __floatditf as usize);
    register_abi("rt_floatdixf", ABI_RT_FLOATDIXF, __floatditf as usize);

    register_abi("rt_floattisf", ABI_RT_FLOATTISF, __floattisf as usize);
    register_abi("rt_floattidf", ABI_RT_FLOATTIDF, __floattidf as usize);
    register_abi("rt_floattitf", ABI_RT_FLOATTITF, __floattitf as usize);
    register_abi("rt_floattixf", ABI_RT_FLOATTIXF, __floattitf as usize);

    register_abi("rt_floatunsisf", ABI_RT_FLOATUNSISF, __floatunsisf as usize);
    register_abi("rt_floatunsidf", ABI_RT_FLOATUNSIDF, __floatunsidf as usize);
    register_abi("rt_floatunsitf", ABI_RT_FLOATUNSITF, __floatunsitf as usize);
    register_abi("rt_floatunsixf", ABI_RT_FLOATUNSIXF, __floatunsitf as usize);

    register_abi("rt_floatundisf", ABI_RT_FLOATUNDISF, __floatundisf as usize);
    register_abi("rt_floatundidf", ABI_RT_FLOATUNDIDF, __floatundidf as usize);
    register_abi("rt_floatunditf", ABI_RT_FLOATUNDITF, __floatunditf as usize);
    register_abi("rt_floatundixf", ABI_RT_FLOATUNDIXF, __floatunditf as usize);

    register_abi("rt_floatuntisf", ABI_RT_FLOATUNTISF, __floatuntisf as usize);
    register_abi("rt_floatuntidf", ABI_RT_FLOATUNTIDF, __floatuntidf as usize);
    register_abi("rt_floatuntitf", ABI_RT_FLOATUNTITF, __floatuntitf as usize);
    register_abi("rt_floatuntixf", ABI_RT_FLOATUNTIXF, __floatuntitf as usize);

    register_abi("rt_fixsfbitint", ABI_RT_FIXSFBITINT, abi_noimpl as usize);
    register_abi("rt_fixdfbitint", ABI_RT_FIXDFBITINT, abi_noimpl as usize);
    register_abi("rt_fixxfbitint", ABI_RT_FIXXFBITINT, abi_noimpl as usize);
    register_abi("rt_fixtfbitint", ABI_RT_FIXTFBITINT, abi_noimpl as usize);

    register_abi(
        "rt_floatbitintsf",
        ABI_RT_FLOATBITINTSF,
        abi_noimpl as usize,
    );
    register_abi(
        "rt_floatbitintdf",
        ABI_RT_FLOATBITINTDF,
        abi_noimpl as usize,
    );
    register_abi(
        "rt_floatbitintxf",
        ABI_RT_FLOATBITINTXF,
        abi_noimpl as usize,
    );
    register_abi(
        "rt_floatbitinttf",
        ABI_RT_FLOATBITINTTF,
        abi_noimpl as usize,
    );
    register_abi(
        "rt_floatbitinthf",
        ABI_RT_FLOATBITINTHF,
        abi_noimpl as usize,
    );
    register_abi(
        "rt_floatbitintbf",
        ABI_RT_FLOATBITINTBF,
        abi_noimpl as usize,
    );

    // Comparison functions [180, 223]
    register_abi("rt_cmpsf2", ABI_RT_CMPSF2, abi_noimpl as usize);
    register_abi("rt_cmpdf2", ABI_RT_CMPDF2, abi_noimpl as usize);
    register_abi("rt_cmptf2", ABI_RT_CMPTF2, abi_noimpl as usize);

    register_abi("rt_unordsf2", ABI_RT_UNORDSF2, __unordsf2 as usize);
    register_abi("rt_unorddf2", ABI_RT_UNORDDF2, __unorddf2 as usize);
    register_abi("rt_unordtf2", ABI_RT_UNORDTF2, __unordtf2 as usize);

    register_abi("rt_eqsf2", ABI_RT_EQSF2, __eqsf2 as usize);
    register_abi("rt_eqdf2", ABI_RT_EQDF2, __eqdf2 as usize);
    register_abi("rt_eqtf2", ABI_RT_EQTF2, __eqtf2 as usize);

    register_abi("rt_nesf2", ABI_RT_NESF2, __nesf2 as usize);
    register_abi("rt_nedf2", ABI_RT_NEDF2, __nedf2 as usize);
    register_abi("rt_netf2", ABI_RT_NETF2, __netf2 as usize);

    register_abi("rt_gesf2", ABI_RT_GESF2, __gesf2 as usize);
    register_abi("rt_gedf2", ABI_RT_GEDF2, __gedf2 as usize);
    register_abi("rt_getf2", ABI_RT_GETF2, __getf2 as usize);

    register_abi("rt_ltsf2", ABI_RT_LTSF2, __ltsf2 as usize);
    register_abi("rt_ltdf2", ABI_RT_LTDF2, __ltdf2 as usize);
    register_abi("rt_lttf2", ABI_RT_LTTF2, __lttf2 as usize);

    register_abi("rt_lesf2", ABI_RT_LESF2, __lesf2 as usize);
    register_abi("rt_ledf2", ABI_RT_LEDF2, __ledf2 as usize);
    register_abi("rt_letf2", ABI_RT_LETF2, __letf2 as usize);

    register_abi("rt_gtsf2", ABI_RT_GTSF2, __gtsf2 as usize);
    register_abi("rt_gtdf2", ABI_RT_GTDF2, __gtdf2 as usize);
    register_abi("rt_gttf2", ABI_RT_GTTF2, __gttf2 as usize);

    // Other floating-point functions [212, 223]
    register_abi("rt_powisf2", ABI_RT_POWISF2, __powisf2 as usize);
    register_abi("rt_powidf2", ABI_RT_POWIDF2, __powidf2 as usize);
    register_abi("rt_powitf2", ABI_RT_POWITF2, __powitf2 as usize);
    register_abi("rt_powixf2", ABI_RT_POWIXF2, __powitf2 as usize);

    register_abi("rt_mulsc3", ABI_RT_MULSC3, abi_noimpl as usize);
    register_abi("rt_muldc3", ABI_RT_MULDC3, abi_noimpl as usize);
    register_abi("rt_multc3", ABI_RT_MULTC3, abi_noimpl as usize);
    register_abi("rt_mulxc3", ABI_RT_MULXC3, abi_noimpl as usize);

    register_abi("rt_divsc3", ABI_RT_DIVSC3, abi_noimpl as usize);
    register_abi("rt_divdc3", ABI_RT_DIVDC3, abi_noimpl as usize);
    register_abi("rt_divtc3", ABI_RT_DIVTC3, abi_noimpl as usize);
    register_abi("rt_divxc3", ABI_RT_DIVXC3, abi_noimpl as usize);

    // ----------------

    register_abi("rt_clzsi2", ABI_RT_CLZSI2, __clzsi2 as usize);
    register_abi("rt_clzdi2", ABI_RT_CLZDI2, __clzdi2 as usize);
    register_abi("rt_clzti2", ABI_RT_CLZTI2, __clzti2 as usize);

    register_abi("rt_bswapsi2", ABI_RT_BSWAPSI2, __bswapsi2 as usize);
    register_abi("rt_bswapdi2", ABI_RT_BSWAPDI2, __bswapdi2 as usize);
}

fn register_abi(name: &str, num: usize, handle: usize) {
    unsafe {
        ABI_TABLE[num] = handle;
    }
    info!("[ABI]{}: 0x{:x}", name, handle);
}

pub fn abi_init_scheduler() {
    info!("[ABI: SYS CTL] init_scheduler");
    init_scheduler();
}

/// `SYS_TERMINATE: 3`
#[unsafe(no_mangle)]
fn abi_terminate() -> ! {
    info!("[ABI: SYS CTL] abi_terminate");
    exit(0);
}

/// `SYS_CHECKPOINT: 4`
#[unsafe(no_mangle)]
fn abi_checkpoint() {
    let ptr_loc = &axhal::cpu::CURRENT_TASK_PTR as *const _ as usize;
    info!(
        "CURRENT_TASK_PTR is at 0x{:x}, value = 0x{:x}",
        ptr_loc,
        &axhal::cpu::CURRENT_TASK_PTR.read_current()
    );

    info!("[ABI: SYS CTL] abi_checkpoint");
}

#[repr(C)]
#[derive(Debug)]
struct TimeSpec {
    tv_sec: usize,
    tv_nsec: usize,
}

/// `SYS_TIMESPEC: 4`
#[unsafe(no_mangle)]
fn abi_timespec(ts: *mut TimeSpec) {
    info!("[ABI: Remove this] abi_timespec");
    unsafe {
        let ts = &mut *ts;
        let now = monotonic_time();
        ts.tv_nsec = now.as_nanos() as usize;
        ts.tv_sec = now.as_secs() as usize;
        debug!("{:?}", ts);
    }
}
