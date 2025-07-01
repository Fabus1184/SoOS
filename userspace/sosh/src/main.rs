#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!("and rsp, ~0xF");
    }

    main();

    panic!("main function returned");
}

fn main() {
    loop {
        for i in 0..10_000_000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }

        unsafe {
            core::arch::asm!("int $0x80", in("rax") 0xdeadbeef_u64);
        }
    }
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
