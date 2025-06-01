#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    log::error!("Kernel panic: {info:#}");

    loop {
        x86_64::instructions::hlt();
    }
}
