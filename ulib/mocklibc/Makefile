PLATFORM=riscv64-linux-musl
CC=$(PLATFORM)-gcc
AS=$(PLATFORM)-as
LD=$(PLATFORM)-ld
AR=$(PLATFORM)-ar
STRIP=$(PLATFORM)-strip
OBJCOPY=$(PLATFORM)-objcopy

CFLAGS = -ffreestanding -nostdlib -O2 -mcmodel=medany -I./include
ASFLAGS = 
LDFLAGS = -T link.ld
LIB_DIRS = lib
OBJ_DIRS = obj

LIB = $(LIB_DIRS)/libmock.a
LIBOBJS = $(OBJ_DIRS)/puts.o \
					$(OBJ_DIRS)/putchar.o \
					$(OBJ_DIRS)/stdio/printf.o \
					$(OBJ_DIRS)/abi.o

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
	rm $(OBJ_DIRS)/*.o
	rm $(LIB_DIRS)/*.o

.PHONYD: all clean
