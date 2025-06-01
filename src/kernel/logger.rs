use core::fmt::Write as _;
use log::LevelFilter;

use crate::term;
pub struct KernelLogger {
    level_filter: LevelFilter,
}
unsafe impl Sync for KernelLogger {}

static mut KERNEL_LOGGER: KernelLogger = KernelLogger {
    level_filter: LevelFilter::Off,
};

pub fn init(level_filter: LevelFilter) {
    unsafe {
        KERNEL_LOGGER.level_filter = level_filter;
    }

    log::set_logger(unsafe { &KERNEL_LOGGER }).expect("Failed to set logger");
    log::set_max_level(level_filter);
}

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &log::Record) {
        let color: &str = match record.level() {
            log::Level::Error => "\x1b[31m", // Red
            log::Level::Warn => "\x1b[33m",  // Yellow
            log::Level::Info => "\x1b[32m",  // Green
            log::Level::Debug => "\x1b[34m", // Blue
            log::Level::Trace => "\x1b[36m", // Cyan
        };

        writeln!(
            term::TERM.writer(),
            "{color}[{}] ({}:{}) {}",
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
        .expect("Failed to write log message");
    }

    fn flush(&self) {}
}
