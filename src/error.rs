use std::fmt;
use std::io;

/// Errors associated with the command-line interface.
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Io(err) => write!(f, "io: {}", err),
        }
    }
}
