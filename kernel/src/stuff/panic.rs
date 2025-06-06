use core::fmt::Write;

use ringbuffer::RingBuffer as _;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    if let Some(ringbuffer) = crate::kernel::logger::KERNEL_LOGGER.try_lock_ringbuffer() {
        let mut writer = crate::term::TERM.writer();
        ringbuffer.iter().copied().for_each(|byte| {
            writer
                .write_char(byte as char)
                .expect("Failed to write to terminal");
        });
    }

    log::error!("\nKernel panic: {info:#}");

    loop {
        x86_64::instructions::hlt();
    }
}
