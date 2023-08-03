use alloc::format;

use crate::term;

pub struct KernelLogger {}

impl log::Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
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
