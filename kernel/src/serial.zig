const ioport = @import("ioport.zig");

pub const SerialPort = struct {
    ports: [6]ioport.IoPort,

    /// serial0 aka COM1
    pub fn serial0() SerialPort {
        return SerialPort{
            .ports = [_]ioport.IoPort{
                ioport.IoPort.new(0x3F8),
                ioport.IoPort.new(0x3F9),
                ioport.IoPort.new(0x3FA),
                ioport.IoPort.new(0x3FB),
                ioport.IoPort.new(0x3FC),
                ioport.IoPort.new(0x3FD),
            },
        };
    }

    pub fn init(self: SerialPort) !void {
        self.ports[1].write(0x00); // Disable all interrupts
        self.ports[3].write(0x80); // Enable DLAB (set baud rate divisor)
        self.ports[0].write(0x03); // Set divisor to 3 (baud rate 38400)
        self.ports[3].write(0x03); // 8 bits, no parity, one stop bit
        self.ports[2].write(0xC7); // Enable FIFO, clear them, with 14-byte threshold
        self.ports[4].write(0x0B); // IRQs enabled, RTS/DSR set

        self.ports[4].write(0x1E); // Enable loopback mode for testing
        self.ports[0].write(0xAE); // Test command to check if serial port is working
        if (self.ports[0].read() != 0xAE) {
            return error.SerialLoopbackTestFailed;
        }
        self.ports[4].write(0x0F); // Disable loopback mode

        self.ports[1].write(0x01); // Enable interrupts for received data
    }

    pub fn write(self: SerialPort, data: u8) void {
        while ((self.ports[5].read() & 0x20) == 0) {} // Wait for the transmit buffer to be empty
        switch (data) {
            '\n' => {
                self.ports[0].write('\r');
                self.ports[0].write('\n');
            },
            else => self.ports[0].write(data),
        }
    }

    pub fn readNonBlocking(self: SerialPort, buffer: []u8) usize {
        var count: usize = 0;
        while ((self.ports[5].read() & 0x01) == 1) {
            buffer[count] = switch (self.ports[0].read()) {
                '\r' => '\n', // Convert carriage return to newline
                else => |byte| byte,
            };
            count += 1;

            if (count == buffer.len) {
                break; // Avoid buffer overflow
            }
        }

        return count;
    }

    // writer implementation

    pub const Error = error{};
    pub fn writeAll(self: @This(), data: []const u8) Error!void {
        for (data) |byte| {
            self.write(byte);
        }
    }
    pub fn writeBytesNTimes(self: @This(), bytes: []const u8, n: usize) Error!void {
        for (0..n) |_| {
            try self.writeAll(bytes);
        }
    }
};
