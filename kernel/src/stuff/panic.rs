use core::fmt::Write;

use ringbuffer::RingBuffer as _;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    log::error!("+++++++ KERNEL PANIC +++++++");

    if let Some(ringbuffer) = crate::kernel::logger::KERNEL_LOGGER.try_lock_ringbuffer() {
        let mut iter = ringbuffer.iter().copied();

        let mut writer = crate::term::TERM.writer();

        let mut buffer: [u8; 512] = [0; 512];
        let mut i = 0;
        loop {
            let mut end = false;
            if let Some(byte) = iter.next() {
                buffer[i] = byte;
                i += 1;
            } else {
                end = true;
            }

            if i == buffer.len() || end {
                let str = core::str::from_utf8(&buffer[..i]).unwrap_or("<invalid UTF-8!>");
                writer.write_str(str).expect("Failed to write to terminal");
                i = 0;
            }

            if end {
                break;
            }
        }
    }

    log::error!("kernel panic: {info:#}");

    loop {
        x86_64::instructions::hlt();
    }
}
