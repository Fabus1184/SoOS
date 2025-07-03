const std = @import("std");

const ioport = @import("ioport.zig");

const PIC_COMMAND_PORT = ioport.IoPort.new(0x20);
const PIC_DATA_PORT = ioport.IoPort.new(0x21);

const PIC2_COMMAND_PORT = ioport.IoPort.new(0xA0);
const PIC2_DATA_PORT = ioport.IoPort.new(0xA1);

fn remapPic() void {
    // Save current state of the PIC data ports
    const a1 = PIC_DATA_PORT.read();
    const a2 = PIC2_DATA_PORT.read();

    // Start initialization sequence
    PIC_COMMAND_PORT.write(0x11); // Start initialization of Master PIC
    PIC2_COMMAND_PORT.write(0x11); // Start initialization of Slave PIC
    // Set vector offsets
    PIC_DATA_PORT.write(0x20); // Master PIC vector offset
    PIC2_DATA_PORT.write(0x28); // Slave PIC vector offset
    // Configure the PICs to cascade
    PIC_DATA_PORT.write(0x04); // Master PIC cascade to Slave PIC
    PIC2_DATA_PORT.write(0x02); // Slave PIC cascade to Master PIC
    // Set the mode to 8086/88
    PIC_DATA_PORT.write(0x01); // Master PIC 8086/88 mode
    PIC2_DATA_PORT.write(0x01); // Slave PIC 8086/88 mode
    // Restore saved state
    PIC_DATA_PORT.write(a1);
    PIC2_DATA_PORT.write(a2);
}

pub fn init() void {
    // Remap the PICs to avoid conflicts with other interrupts
    remapPic();

    // Unmask all interrupts on the Master PIC
    PIC_DATA_PORT.write(0x00);
    // Unmask all interrupts on the Slave PIC
    PIC2_DATA_PORT.write(0x00);

    std.log.debug("pic initialized, remapped, and interrupts unmasked", .{});
}

pub fn endOfInterrupt(irq: u8) void {
    if (irq >= 8) {
        // Send end-of-interrupt signal to Slave PIC
        PIC2_COMMAND_PORT.write(0x20);
    }

    // Send end-of-interrupt signal to Master PIC
    PIC_COMMAND_PORT.write(0x20);
}
