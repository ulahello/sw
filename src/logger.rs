use log::{self, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Simple logging implementation for `sw` non-fatal events.
pub struct Logger;

impl Logger {
    /// One-time initialize the logger.
    ///
    /// # Errors
    ///
    /// Returns [`SetLoggerError`] if the logger has already been initialized.
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
            let stderr = BufferWriter::stderr(ColorChoice::Auto);
            let mut buffer = stderr.buffer();

            // set log color based on level
            buffer
                .set_color(ColorSpec::new().set_fg(Some(match record.level() {
                    Level::Error => Color::Ansi256(9), // bright red
                    Level::Warn => Color::Yellow,      // unused
                    Level::Info => Color::Ansi256(13), // bright magenta
                    Level::Debug => Color::Blue,       // unused
                    Level::Trace => Color::Green,      // unused
                })))
                .unwrap();

            // print log contents
            writeln!(buffer, "{}", record.args()).unwrap();

            // reset color
            buffer.reset().unwrap();

            // flush buffer
            stderr.print(&buffer).unwrap();
        }
    }

    fn flush(&self) {}
}
