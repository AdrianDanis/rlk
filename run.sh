#!/bin/sh

objcopy --output-target elf32-i386 $1 $1.elf32

qemu-system-x86_64 -M pc -m 64 -kernel $1.elf32 -cpu Haswell,+pdpe1gb -serial mon:stdio -nographic -append "--earlycon=serial,port=0x3f8" -no-reboot
