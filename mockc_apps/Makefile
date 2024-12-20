SRC ?=

all: subdir toplash

subdir:
	$(MAKE) -C $(SRC)

toplash:
	dd if=/dev/zero of=./apps.bin bs=1M count=32
	dd if=./$(SRC)/apps.bin of=./apps.bin conv=notrunc
	cp ./apps.bin ../payload/apps.bin

.PHONY: all toplash
