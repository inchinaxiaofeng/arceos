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
DIRS = obj

LIBOBJS = $(DIRS)/puts.o \
					$(DIRS)/putchar.o \
					$(DIRS)/abi.o
CRTOBJS = $(DIRS)/crt1.o

all: libmock.a $(CRTOBJS)

$(DIRS):
	mkdir -p $@

$(LIBOBJS): | $(dir $(LIBOBJS))
$(CRTOBJS): | $(dir $(CRTOBJS))

$(CRTOBJS): c/crt1.c include/crt_arch.h
	$(CC) $(CFLAGS) -c -o $@ $<

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

%.o: %.s
	$(AS) $(ASFLAGS) -o $@ $<

libmock.a: $(LIBOBJS) $(CRTOBJS)
	$(AR) rcs $@ $(LIBOBJS)

.PHONYD: all
