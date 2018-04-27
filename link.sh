#!/bin/sh

# Remove "-flavor gnu"
shift
shift

# No way to ask what the actual linker that rust was going to use was so just try an
# x86_64 linker. We are only using this linker wrapper as this executes in the current
# directory and so our -T flag will find our linker script (which is in this directory)
x86_64-linux-gnu-ld $@
