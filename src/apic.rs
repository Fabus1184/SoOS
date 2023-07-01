use core::arch::asm;

fn pic_mask_interrupts() {
    const PIC1_ADDR: u16 = 0x20;
    const PIC2_ADDR: u16 = 0xA0;
    const PIC1_DATA: u16 = PIC1_ADDR + 1;
    const PIC2_DATA: u16 = PIC2_ADDR + 1;

    unsafe {
        asm!("mov al, 0xFF", "out 0x21, al", "out 0xA1, al",);
    }
}

pub fn init() {
    pic_mask_interrupts();

    if !raw_cpuid::CpuId::new()
        .get_feature_info()
        .map(|f| f.has_apic())
        .unwrap_or(false)
    {
        panic!("APIC not supported!");
    }
}
