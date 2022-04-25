use log::{self, LevelFilter, Log, Metadata, Record, SetLoggerError};

/// Simple logging implementation for `sw` non-fatal events.
pub struct Logger;

impl Logger {
    /// One-time initialize the logger.
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&Self).map(|()| log::set_max_level(LevelFilter::Info))
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{}", record.args());
        }
    }

    fn flush(&self) {}
}
