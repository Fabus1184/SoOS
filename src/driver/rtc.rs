use crate::asm::{inb, outb};

pub fn get_time() -> chrono::NaiveDateTime {
    let mut second = rtc_read(0x00);
    let mut minute = rtc_read(0x02);
    let mut hour = rtc_read(0x04);
    let mut day = rtc_read(0x07);
    let mut month = rtc_read(0x08);
    let mut year = rtc_read(0x09);

    let register_b = rtc_read(0x0B);

    if (register_b & 0x04) == 0 {
        second = (second & 0x0F) + ((second / 16) * 10);
        minute = (minute & 0x0F) + ((minute / 16) * 10);
        hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
        day = (day & 0x0F) + ((day / 16) * 10);
        month = (month & 0x0F) + ((month / 16) * 10);
        year = (year & 0x0F) + ((year / 16) * 10);
    }

    if (register_b & 0x02) == 0 && (hour & 0x80) != 0 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    let year = year as u64 + 2000;

    let unix: u64 = (year - 1970) * 31536000 + ((year - 1969) / 4) * 86400
        - ((year - 1901) / 100) * 86400
        + ((year - 1601) / 400) * 86400
        + match month {
            1 => 0,
            2 => 2678400,
            3 => 5097600,
            4 => 7776000,
            5 => 10368000,
            6 => 13046400,
            7 => 15638400,
            8 => 18316800,
            9 => 20995200,
            10 => 23587200,
            11 => 26265600,
            12 => 28857600,
            _ => 0,
        }
        + (day as u64 - 1) * 86400
        + hour as u64 * 3600
        + minute as u64 * 60
        + second as u64;

    chrono::NaiveDateTime::from_timestamp_opt(unix as i64, 0).expect("Invalid time!")
}

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

fn rtc_read(reg: u8) -> u8 {
    core::iter::repeat_with(|| {
        (
            unsafe {
                outb(CMOS_ADDRESS, reg);
                inb(CMOS_DATA)
            },
            unsafe {
                outb(CMOS_ADDRESS, reg);
                inb(CMOS_DATA)
            },
        )
    })
    .find(|(a, b)| a == b)
    .unwrap()
    .0
}
