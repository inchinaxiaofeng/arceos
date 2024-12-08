APP ?=

all: subdir link

subdir:
	$(MAKE) -C $(APP)

link:
	dd if=/dev/zero of=./apps.bin bs=1M count=32
	dd if=./$(APP)/apps.bin of=./apps.bin conv=notrunc

.PHONY: all link
