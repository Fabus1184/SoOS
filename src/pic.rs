use x86_64::structures::port::{PortRead, PortWrite};

pub fn remap_pic(offset1: u8, offset2: u8) {
    let a1: u8 = unsafe { PortRead::read_from_port(0x21) };
    let a2: u8 = unsafe { PortRead::read_from_port(0xA1) };

    unsafe {
        PortWrite::write_to_port(0x20, 0x11_u8);
        PortWrite::write_to_port(0xA0, 0x11_u8);
        PortWrite::write_to_port(0x21, offset1);
        PortWrite::write_to_port(0xA1, offset2);
        PortWrite::write_to_port(0x21, 0x04_u8);
        PortWrite::write_to_port(0xA1, 0x02_u8);
        PortWrite::write_to_port(0x21, 0x01_u8);
        PortWrite::write_to_port(0xA1, 0x01_u8);
        PortWrite::write_to_port(0x21, a1);
        PortWrite::write_to_port(0xA1, a2);
    }
}

pub fn eoi(irq: u8) {
    if irq >= 8 {
        unsafe {
            PortWrite::write_to_port(0xA0, 0x20_u8);
        }
    }
    unsafe {
        PortWrite::write_to_port(0x20, 0x20_u8);
    }
}

pub fn init() {
    remap_pic(0x20, 0x28);

    // unmask all PIC interrupts
    unsafe {
        PortWrite::write_to_port(0x21, 0x00_u8);
        PortWrite::write_to_port(0xA1, 0x00_u8);
    }
}
