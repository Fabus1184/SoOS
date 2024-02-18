use alloc::{boxed::Box, format};
use log::LevelFilter;

use crate::term;

pub struct KernelLogger {
    level_filter: LevelFilter,
}

impl KernelLogger {
    pub fn new(level_filter: LevelFilter) -> &'static Self {
        Box::leak(Box::new(Self { level_filter }))
    }

    pub fn init(&'static self) {
        log::set_logger(self).unwrap();
        log::set_max_level(self.level_filter);
    }
}

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &log::Record) {
        unsafe {
            term::TERM.println(&format!(
                "[{}] ({}:{}) {}",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            ))
        };
    }

    fn flush(&self) {}
}
