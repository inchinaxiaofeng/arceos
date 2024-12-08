SHELL := /bin/bash
# 定义变量
# 传递给cargo的--target的参数，同时也是target目录中的生成结果的目录名字
TARGET := riscv64gc-unknown-none-elf
# 使用的readelf工具
READELF := riscv64-unknown-elf-readelf
# release的目录
RELEASE_DIR := $(ARCEOS_HOME)/target/$(TARGET)/release
# myapps目录
MYAPPS_DIR := $(ARCEOS_HOME)/myapps
# Cargo配置文件的名字（默认都是Cargo）
FILE := Cargo.toml
# 使用cargo中的"name"来找到这个rust项目生成的名字
SEARCH_STRING ?= name

# 使用 shell 命令查找并提取内容，获得这个rust项目生成的名字
APP := $(shell grep -m 1 "$(SEARCH_STRING)" $(FILE) | sed -n 's/.*"\([^"]*\)".*/\1/p')
APP_BIN = $(APP).bin
# 头结构的名字
HEADER_FILE = header.bin
# 输出的bin的名字
OUTPUT_BIN ?= apps.bin

# 目标
all: build header link

build:
	@echo "APP variable: $(APP)"
	cargo rustc --target $(TARGET) --release -- -C link-arg=-T$(MYAPPS_DIR)/linker.ld
	rust-objcopy --binary-architecture=riscv64 --strip-all -O binary $(RELEASE_DIR)/$(APP) $(APP_BIN)
	
header: $(APP_BIN)
	bash $(MYAPPS_DIR)/header.sh --header_file $(HEADER_FILE) --readelf $(READELF) --app_bin $(APP_BIN) --app $(APP) --release_dir $(RELEASE_DIR)

link: $(APP_BIN) $(HEADER_FILE)
	dd if=./$(HEADER_FILE) of=./$(OUTPUT_BIN) bs=1 count=$$(stat -c %s $(HEADER_FILE))
	dd if=./$(APP_BIN) of=./$(OUTPUT_BIN) bs=1 oflag=append conv=notrunc

clean:
	@echo "Cleaning up..."
	cargo clean
	rm -f $(OUTPUT_BIN) $(HEADER_FILE) $(APP_BIN)

.PHONY: all build clean header link

