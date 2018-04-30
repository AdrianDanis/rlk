#!/bin/sh

objcopy --output-target elf32-i386 $1 $1.elf32

qemu-system-x86_64 -M pc -m 64 -kernel $1.elf32 -cpu Haswell,+pdpe1gb -serial mon:stdio -append "--earlycon=vga_80_25" -no-reboot
