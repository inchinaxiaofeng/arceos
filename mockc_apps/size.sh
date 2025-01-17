#!/bin/bash

# 解析命令行参数
options=$(getopt -o o:i: --long outfile:,infile: -- "$@")
eval set -- "$options"

# 提取选项和参数
while true; do
	case $1 in
	-o | --outfile)
		shift
		outfile=$1
		shift
		;;
	-i | --infile)
		shift
		infile=$1
		shift
		;;
	--)
		shift
		break
		;;
	*) echo "Invalid option: $1" exit 1 ;;
	esac
done

stat -c %s "$infile" | xargs printf '%016lx\n' | sed 's/../& /g' | awk '{for(i=8;i>0;i--) printf $i; printf "\n"}' | xxd -r -p >"$outfile"
