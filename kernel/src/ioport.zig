pub const IoPort = struct {
    port: u16,

    pub fn new(port: u16) IoPort {
        return IoPort{ .port = port };
    }

    pub inline fn read(self: IoPort) u8 {
        return asm volatile (
            \\ inb %dx, %al
            : [value] "={al}" (-> u8),
            : [port] "{dx}" (self.port),
        );
    }

    pub inline fn write(self: IoPort, value: u8) void {
        asm volatile (
            \\ outb %al, %dx
            :
            : [port] "{dx}" (self.port),
              [value] "{al}" (value),
        );
    }
};
