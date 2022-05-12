#!/bin/bash
qemu-system-i386 -drive format=raw,file=os-image.bin -d guest_errors -soundhw pcspk -m 1G
