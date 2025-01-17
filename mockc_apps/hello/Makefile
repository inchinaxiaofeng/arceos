MOCK_LIBC=$(ARCEOS_HOME)/ulib/mocklibc
PLATFORM=riscv64-linux-musl
CC=$(PLATFORM)-gcc
LD=$(PLATFORM)-ld
STRIP=$(PLATFORM)-strip
OBJCOPY=$(PLATFORM)-objcopy

READELF := riscv64-unknown-elf-readelf
HEADER_FILE=header.bin
OUTPUT_BIN?=apps.bin

SRC?=hello # TODO: 处理
SRC_C=$(SRC).c # TODO: 改成Makefile函数
SRC_O=$(SRC).o # TODO: 改成Makefile函数
SRC_BIN=$(OUTPUT_BIN)#$(SRC).bin
SRC_DIR=$(SRC)_apps

CFLAGS := -nostdlib -nodefaultlibs -ffreestanding -O0 -mcmodel=medany -nostartfiles
STATIC_CFLAGS := -static -no-pie
LINK_FLAGS := # -T$(ARCEOS_HOME)/mockc_apps/link.ld

all: default_build

default_build: dynamic

static:
	$(CC) $(CFLAGS) $(STATIC_CFLAGS) -I$(MOCK_LIBC)/include -c $(SRC_C) -o $(SRC_O)
	#$(LD) $(SRC_O) -L$(MOCK_LIBC) -lmock -T$(ARCEOS_HOME)/mockc_apps/link.ld -o $(SRC)
	$(LD) $(SRC_O) $(MOCK_LIBC)/obj/crt1.o $(MOCK_LIBC)/lib/libmock.a $(LINK_FLAGS) -T$(ARCEOS_HOME)/mockc_apps/link.ld -o $(SRC)
	$(STRIP) -s $(SRC)
	mv $(SRC) $(SRC_BIN)
	#rust-objcopy --binary-architecture=riscv64 --strip-all -O binary $(SRC) ./$(SRC_BIN)
	#$(OBJCOPY) --strip-all -O binary $(SRC) $(SRC_BIN)

dynamic:
	$(CC) $(CFLAGS) -I$(MOCK_LIBC)/include -c $(SRC_C) -o $(SRC_O)
	$(LD) $(SRC_O)  -L$(MOCK_LIBC)/lib -lmock $(LINK_FLAGS) -o $(SRC)
	$(STRIP) -s $(SRC)
	mv $(SRC) $(SRC_BIN)


.PHONY: all build
