PLATFORM=riscv64-linux-musl
CC=$(PLATFORM)-gcc
AS=$(PLATFORM)-as
LD=$(PLATFORM)-ld
AR=$(PLATFORM)-ar
STRIP=$(PLATFORM)-strip
OBJCOPY=$(PLATFORM)-objcopy

CFLAGS = -ffreestanding -nostdlib -O2 -mcmodel=medany -I./include -I./c/internal/ -I./c/math/ -I./arch/riscv64/ # -I./c/include/
ASFLAGS = 
LDFLAGS = -T link.ld
LIB_DIRS = lib
OBJ_DIRS = obj

LIB = $(LIB_DIRS)/libmock.a
LIBSRC = $(wildcard c/stdio/*.c) \
				 $(wildcard c/ctype/*.c) \
				 $(wildcard c/stdlib/*.c) \
				 $(wildcard c/time/*.c) \
				 $(wildcard c/network/*.c) \
				 $(wildcard c/math/*.c) \
				 c/abi.c
LIBO = $(patsubst %.c, %.o, $(LIBSRC))
LIBOBJS = $(patsubst c/%, $(OBJ_DIRS)/%, $(LIBO))
CRTOBJS = $(OBJ_DIRS)/crt1.o

all: $(LIB) $(CRTOBJS) $(LIBOBJS)

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

clean:
	rm $(CRTOBJS)
	rm $(LIBOBJS)
	rm $(LIB_DIRS)/*

.PHONYD: all clean
