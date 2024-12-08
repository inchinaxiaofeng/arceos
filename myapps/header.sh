#!/bin/bash

# 解析命令行参数
options=$(getopt -o h:e:b:a:r: --long header_file:,readelf:,app_bin:,app:,release_dir: -- "$@")
eval set -- "$options"

# 提取选项和参数
while true; do
	case $1 in
	-h | --header_file)
		shift
		header_file=$1
		shift
		;;
	-e | --readelf)
		shift
		readelf=$1
		shift
		;;
	-b | --app_bin)
		shift
		app_bin=$1
		shift
		;;
	-a | --app)
		shift
		app=$1
		shift
		;;
	-r | --release_dir)
		shift
		release_dir=$1
		shift
		;;
	--)
		shift
		break
		;;
	*) echo "Invalid option: $1" exit 1 ;;
	esac
done

echo -n "7856341278563412" | xxd -r -p >"$header_file"
stat -c %s "$app_bin" | xargs printf '%016lx\n' | sed 's/../& /g' | awk '{for(i=8;i>0;i--) printf $i; printf "\n"}' | xxd -r -p >>"$header_file"
"$readelf" -h "$release_dir"/"$app" | grep 'Entry point address:' | awk '{printf $4}' | sed 's/^0x//' | sed 's/../& /g' | awk '{for(i=8;i>0;i--) printf $i; printf "\n"}' | xxd -r -p >>"$header_file"
