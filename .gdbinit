target remote 127.0.0.1:1234

add-symbol-file build/iso-root/kernel.elf

set disassembly-flavor intel
set disassemble-next-line on