echo "Loader my lib version"
#!/bin/bash

# 设置默认参数
log="warn"
qemu_log="y"

# 解析命令行参数
options=$(getopt -o l:q: --long log:,qemu_log:, -- "$@")
eval set -- "$options"

# 提取选项和参数
while true; do
	case $1 in
	-l | --log)
		shift
		log=$1
		shift
		;;
	-q | --qemu_log)
		shift
		qemu_log=$1
		shift
		;;
	--)
		shift
		break
		;;
	*)
		echo "Invalid option: $1" exit 1
		;;
	esac
done
make run ARCH=riscv64 A=examples/loader_lib LOG="$log" QEMU_LOG="$qemu_log"
