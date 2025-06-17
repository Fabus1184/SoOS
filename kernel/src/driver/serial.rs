use x86_64::structures::port::{PortRead, PortWrite};

pub struct SerialPort {
    port: u16,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("Serial port initialization failed")]
    InitializationFailed,
}

static COM1: spin::Lazy<Result<SerialPort, Error>> = spin::Lazy::new(|| {
    let port = SerialPort::com1();
    port.init().map(|()| port)
});

pub fn com1() -> Result<&'static SerialPort, Error> {
    COM1.as_ref().map_err(|&e| e)
}

impl SerialPort {
    pub const fn com1() -> Self {
        SerialPort { port: 0x3F8 }
    }

    pub fn init(&self) -> Result<(), Error> {
        unsafe {
            <u8 as PortWrite>::write_to_port(self.port + 1, 0x00); // Disable all interrupts
            <u8 as PortWrite>::write_to_port(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            <u8 as PortWrite>::write_to_port(self.port + 0, 0x03); // Set divisor to 3 (baud rate 38400)
            <u8 as PortWrite>::write_to_port(self.port + 1, 0x00); // Set divisor to 3 (baud rate 38400)
            <u8 as PortWrite>::write_to_port(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
            <u8 as PortWrite>::write_to_port(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            <u8 as PortWrite>::write_to_port(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
            <u8 as PortWrite>::write_to_port(self.port + 4, 0x1E); // Set in loopback mode, test the serial chip

            <u8 as PortWrite>::write_to_port(self.port + 0, 0x0AE); // Test serial chip

            // check if the serial port is faulty
            if <u8 as PortRead>::read_from_port(self.port + 0) != 0x0AE {
                return Err(Error::InitializationFailed);
            }

            <u8 as PortWrite>::write_to_port(self.port + 4, 0x0F); // Set in normal operation mode

            <u8 as PortWrite>::write_to_port(self.port + 1, 0x01); // Enable data received interrupt

            Ok(())
        }
    }

    pub fn write_byte_blocking(&self, byte: u8) {
        unsafe {
            while <u8 as PortRead>::read_from_port(self.port + 5) & 0x20 == 0 {}
            <u8 as PortWrite>::write_to_port(self.port, byte);
        }
    }

    pub fn read_bytes(&self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for byte in buffer.iter_mut() {
            if unsafe { <u8 as PortRead>::read_from_port(self.port + 5) & 1 } != 0 {
                *byte = unsafe { <u8 as PortRead>::read_from_port(self.port) };
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// returns a writer for the serial port that implements `core::fmt::Write`
    /// the writer automatically handles newlines by converting `\n` to `\r\n`
    pub fn writer(&'_ self) -> SerialPortWriter<'_> {
        SerialPortWriter { port: self }
    }
}

pub struct SerialPortWriter<'a> {
    port: &'a SerialPort,
}

impl core::fmt::Write for SerialPortWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    self.port.write_byte_blocking(b'\r');
                    self.port.write_byte_blocking(b'\n');
                }
                _ => self.port.write_byte_blocking(byte),
            }
        }
        Ok(())
    }
}
