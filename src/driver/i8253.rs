use x86_64::structures::port::{PortRead, PortWrite};

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum Channel {
    CH0 = 0b00000000,
    CH1 = 0b01000000,
    CH2 = 0b10000000,
}

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum AccessMode {
    Latch = 0b00000000,
    LoByte = 0b00010000,
    HiByte = 0b00100000,
    LoHiByte = 0b00110000,
}

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum OperatingMode {
    InterruptOnTerminalCount = 0b00000000,
    HardwareRetriggerableOneShot = 0b00000010,
    RateGenerator = 0b00000100,
    SquareWaveGenerator = 0b00000110,
    SoftwareTriggeredStrobe = 0b00001000,
    HardwareTriggeredStrobe = 0b00001010,
}

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum BCDMode {
    Binary = 0b00000000,
    BCD = 0b00000001,
}

const COMMAND_PORT: u16 = 0x43;
pub struct Timer {
    channel: Channel,
    access_mode: AccessMode,
    operating_mode: OperatingMode,
    bcd_mode: BCDMode,
    frequency: u32,
    ticks: u64,
}

pub static mut TIMER0: Timer = Timer {
    channel: Channel::CH0,
    access_mode: AccessMode::LoHiByte,
    operating_mode: OperatingMode::RateGenerator,
    bcd_mode: BCDMode::Binary,
    frequency: 0,
    ticks: 0,
};

pub static mut TIMER1: Timer = Timer {
    channel: Channel::CH1,
    access_mode: AccessMode::LoHiByte,
    operating_mode: OperatingMode::RateGenerator,
    bcd_mode: BCDMode::Binary,
    frequency: 0,
    ticks: 0,
};

pub static mut TIMER2: Timer = Timer {
    channel: Channel::CH2,
    access_mode: AccessMode::LoHiByte,
    operating_mode: OperatingMode::RateGenerator,
    bcd_mode: BCDMode::Binary,
    frequency: 0,
    ticks: 0,
};

impl Timer {
    pub fn init(
        &mut self,
        hz: u32,
        channel: Channel,
        access_mode: AccessMode,
        operating_mode: OperatingMode,
        bcd_mode: BCDMode,
    ) {
        let mut mode: u8 = 0;
        mode |= channel as u8;
        mode |= access_mode as u8;
        mode |= operating_mode as u8;
        mode |= bcd_mode as u8;

        let divisor = 1193180 / hz;

        let data_port = match channel {
            Channel::CH0 => 0x40,
            Channel::CH1 => 0x41,
            Channel::CH2 => 0x42,
        };

        unsafe {
            PortWrite::write_to_port(COMMAND_PORT, mode);
            PortWrite::write_to_port(data_port, (divisor & 0xFF) as u8);
            PortWrite::write_to_port(data_port, ((divisor >> 8) & 0xFF) as u8);
        }

        self.channel = channel;
        self.access_mode = access_mode;
        self.operating_mode = operating_mode;
        self.bcd_mode = bcd_mode;
        self.frequency = hz;
        self.ticks = 0;
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    pub fn sleep(&self, ms: u64) {
        let start = self.ticks;
        let ticks = ms * self.frequency as u64 / 1000;

        while unsafe { core::ptr::read_volatile(&self.ticks) } < start + ticks {}
    }

    pub fn counter(&self) -> u16 {
        let data_port = match self.channel {
            Channel::CH0 => 0x40,
            Channel::CH1 => 0x41,
            Channel::CH2 => 0x42,
        };

        match self.access_mode {
            AccessMode::LoByte => unsafe {
                let b: u8 = PortRead::read_from_port(data_port);
                b as u16
            },
            AccessMode::HiByte => unsafe {
                let b: u8 = PortRead::read_from_port(data_port);
                (b as u16) << 8
            },
            AccessMode::LoHiByte => {
                let lo = unsafe {
                    let b: u8 = PortRead::read_from_port(data_port);
                    b as u16
                };
                let hi = unsafe {
                    let b: u8 = PortRead::read_from_port(data_port);
                    b as u16
                } << 8;
                lo | hi
            }
            _ => 0,
        }
    }

    pub fn time(&self) -> core::time::Duration {
        core::time::Duration::from_millis(
            (self.ticks as f64 / self.frequency as f64 * 1000.0) as u64,
        )
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }
}
