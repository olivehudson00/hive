#!/bin/sh
# bee - hive test builder
# Copyright (C) 2025 Olive Hudson
# see LICENCE file for licensing information

if [ "$#" -lt 2 ] ; then
	echo 'bee: missing command and directory' >&2
	exit 1
fi

case "$1" in
	init)
		mkdir -p "$2"
		echo '#!/bin/sh\nset -e' >"$2/compile.sh"
		echo '#!/bin/sh' >"$2/run.sh"
		chmod 755 "$2/compile.sh" "$2/run.sh"
		;;
	build)
		rm -f "$2/test.tar.gz"
		tar -czvf "$2/test.tar.gz" "$2/*"
		;;
	add-test)
		echo -n 'bee: enter test name: '
		read -r NAME
		echo -n 'bee: enter test case input: '
		read -r INPUT
		echo -n 'bee: enter test case arguments: '
		read -r ARGUMENTS
		echo -n 'bee: enter test case expected output: '
		read -r EXPECTED

		ARGUMENTS="${ARGUMENTS:+ $ARGUMENTS}"

		echo "echo '$NAME'" >>"$2/run.sh"
		echo "echo '$INPUT'" >>"$2/run.sh"
		echo "echo \"\$(echo '$INPUT' | shackle ./program $ARGUMENTS)\"" >>"$2/run.sh"
		echo "echo '$EXPECTED'" >>"$2/run.sh"
		echo >>"$2/run.sh"
		;;
	*)
		echo 'bee: unknown command' >&2
		exit 1
		;;
esac
