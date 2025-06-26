#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "
            and rsp, rsp, ~0xF
            call {}
            ",
            sym main
        );
    }

    unreachable!();
}

fn main() {
    for i in 0..100_000_000 {
        unsafe {
            core::arch::asm!("nop");
        }
    }
    loop {
        // deference null pointer to trigger a panic
        unsafe {
            let ptr: *const u8 = core::ptr::null();
            core::ptr::read_volatile(ptr);
        }
    }
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
