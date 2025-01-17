PLATFORM=riscv64-linux-musl
CC=$(PLATFORM)-gcc
AS=$(PLATFORM)-as
LD=$(PLATFORM)-ld
AR=$(PLATFORM)-ar
STRIP=$(PLATFORM)-strip
OBJCOPY=$(PLATFORM)-objcopy

INTERNAL_INCLUDE = -I./c/math/ -I./c/include/ -I./include -I./c/internal/ -I./c/time/ -I./arch/riscv64/
CFLAGS = -nostartfiles -ffreestanding -nostdlib -O2 -mcmodel=medany $(INTERNAL_INCLUDE) -fPIC
ASFLAGS = 
LDFLAGS = # -T link.ld
LIB_DIRS = lib
OBJ_DIRS = obj

LIB = $(LIB_DIRS)/libmock.a
SHARED_LIB = $(LIB_DIRS)/libmock.so
LIBSRC = $(wildcard c/stdio/*.c) \
				 $(wildcard c/ctype/*.c) \
				 $(wildcard c/stdlib/*.c) \
				 $(wildcard c/time/*.c) \
				 $(wildcard c/string/*.c) \
				 $(wildcard c/fenv/*.c) \
#				 $(wildcard c/errno/*.c) \
#				 $(wildcard c/complex/*.c) \
#				 $(wildcard c/network/*.c) \
#				 $(wildcard c/math/*.c) \
#				 c/abi.c \

LIBO = $(patsubst %.c, %.o, $(LIBSRC))
LIBOBJS = $(patsubst c/%, $(OBJ_DIRS)/%, $(LIBO))
CRTOBJS = $(OBJ_DIRS)/crt1.o

all: $(LIB) $(SHARED_LIB) $(CRTOBJS) $(LIBOBJS)

static: $(LIB) $(CRTOBJS) $(LIBOBJS)

dynamic: $(SHARED_LIB) $(CRTOBJS) $(LIBOBJS)

$(OBJ_DIRS):
	mkdir -p $@

$(LIB_DIRS):
	mkdir -p $@

$(LIBOBJS): | $(dir $(LIBOBJS))
$(CRTOBJS): | $(dir $(CRTOBJS))

$(CRTOBJS): c/crt1.c include/crt_arch.h
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIRS)/%.o: c/%.c
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIRS)/%.o: c/%.s
	$(AS) $(ASFLAGS) -o $@ $<

$(LIB): $(LIBOBJS) $(CRTOBJS)
	$(AR) rcs $@ $(LIBOBJS)

$(SHARED_LIB): $(LIBOBJS) $(CRTOBJS)
	$(CC) $(CFLAGS) $(LDFLAGS) -shared -fPIC -o $@ $^

clean:
	rm $(SHARED_LIB) -rf
	rm $(CRTOBJS) -rf
	rm $(LIBOBJS) -rf
	rm $(LIB_DIRS)/* -rf

.PHONYD: all clean
# // FIXME: 将数学库拿走，目前好像出现了对一些浮点数的操作的函数找不到的问题
