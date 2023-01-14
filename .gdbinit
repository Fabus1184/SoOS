set pagination off
set logging file gdb.log
set logging on

add-symbol-file build/soos.bin.debug

target remote localhost:1234
