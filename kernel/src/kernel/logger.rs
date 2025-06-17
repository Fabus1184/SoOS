use core::fmt::Write as _;
use log::LevelFilter;

use crate::term;
pub struct KernelLogger {
    level_filter: core::cell::Cell<LevelFilter>,
    ringbuffer: spin::Once<spin::Mutex<ringbuffer::AllocRingBuffer<u8>>>,
}
unsafe impl Sync for KernelLogger {}

pub static KERNEL_LOGGER: spin::Lazy<KernelLogger> = spin::Lazy::new(|| KernelLogger {
    level_filter: core::cell::Cell::new(LevelFilter::Off),
    ringbuffer: spin::Once::new(),
});

impl KernelLogger {
    pub fn init(&'static self, level_filter: LevelFilter) {
        self.level_filter.set(level_filter);

        log::set_logger(self).expect("Failed to set logger");
        log::set_max_level(level_filter);
    }

    pub fn set_level_filter(&self, level_filter: LevelFilter) {
        self.level_filter.set(level_filter);
        log::set_max_level(level_filter);
    }

    pub fn init_ringbuffer(&self) {
        self.ringbuffer
            .call_once(|| spin::Mutex::new(ringbuffer::AllocRingBuffer::new(1024 * 1024)));
    }

    pub fn lock_ringbuffer(&self) -> spin::MutexGuard<'_, ringbuffer::AllocRingBuffer<u8>> {
        self.ringbuffer
            .get()
            .expect("logger ringbuffer not initialized")
            .lock()
    }

    pub fn try_lock_ringbuffer(
        &self,
    ) -> Option<spin::MutexGuard<'_, ringbuffer::AllocRingBuffer<u8>>> {
        self.ringbuffer.get().and_then(spin::mutex::Mutex::try_lock)
    }
}

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level_filter.get()
    }

    fn log(&self, record: &log::Record) {
        let color: &str = match record.level() {
            log::Level::Error => "\x1b[31m", // Red
            log::Level::Warn => "\x1b[33m",  // Yellow
            log::Level::Info => "\x1b[32m",  // Green
            log::Level::Debug => "\x1b[34m", // Blue
            log::Level::Trace => "\x1b[36m", // Cyan
        };

        if let Some(ringbuffer) = self.ringbuffer.get() {
            struct RingbufferWriter<'a> {
                ringbuffer: &'a mut ringbuffer::AllocRingBuffer<u8>,
            }

            impl core::fmt::Write for RingbufferWriter<'_> {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    self.ringbuffer.extend(s.bytes());
                    core::fmt::Result::Ok(())
                }
            }

            let mut ringbuffer = ringbuffer.try_lock().expect("logger ringbuffer deadlock");

            writeln!(
                RingbufferWriter {
                    ringbuffer: &mut ringbuffer
                },
                "{color}[{}] ({}:{}) {}",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
            .expect("Failed to write log message");
        }

        if !self.ringbuffer.is_completed() || record.level() <= log::Level::Warn {
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

        if let Ok(com1) = crate::driver::serial::com1() {
            writeln!(
                com1.writer(),
                "{color}[{}] ({}:{}) {}\x1b[0m",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
            .expect("Failed to write log message to COM1");
        }
    }

    fn flush(&self) {}
}
