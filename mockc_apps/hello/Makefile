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

all: build

build:
	$(CC) --static -nostartfiles -nostdlib -I$(MOCK_LIBC)/include -ffreestanding -O2 -mcmodel=medany -c $(SRC_C)
	#$(LD) $(SRC_O) -L$(MOCK_LIBC) -lmock -T$(ARCEOS_HOME)/mockc_apps/link.ld -o $(SRC)
	$(LD) $(SRC_O) $(MOCK_LIBC)/obj/crt1.o $(MOCK_LIBC)/libmock.a -T$(ARCEOS_HOME)/mockc_apps/link.ld -o $(SRC)
	$(STRIP) -s $(SRC)
	mv $(SRC) $(SRC_BIN)
	#rust-objcopy --binary-architecture=riscv64 --strip-all -O binary $(SRC) ./$(SRC_BIN)
	#$(OBJCOPY) --strip-all -O binary $(SRC) $(SRC_BIN)

.PHONY: all build
