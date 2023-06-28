#![no_std]
#![no_main]

use core::arch::asm;

static FRAMEBUFFER_REQUEST: limine::LimineFramebufferRequest =
    limine::LimineFramebufferRequest::new(0);

#[no_mangle]
extern "C" fn _start() -> ! {
    let response = FRAMEBUFFER_REQUEST.get_response().get().unwrap();
    if response.framebuffer_count == 0 {
        panic!("No framebuffer found!");
    }
    let fb = &response.framebuffers()[0];

    for x in 0..fb.width {
        for y in 0..fb.height {
            if x % 100 == 0 || y % 100 == 0 {
                unsafe {
                    let ptr = fb.address.as_ptr().unwrap() as *mut u32;
                    ptr.wrapping_offset((y * fb.width + x) as isize)
                        .write(0xFF00FF00);
                }
            }
        }
    }

    panic!("Kernel reached end of _start()!");
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}
