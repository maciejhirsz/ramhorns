use std::{io, fmt};

/// Error type used that can be emitted during template parsing.
#[derive(Debug)]
pub enum Error {
    /// There was an error with the IO (only happens when parsing a file)
    Io(io::Error),

    /// There was a parsing error.
    ParsingError,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::ParsingError => write!(f, "There was an error parsing the template!"),
        }
    }
}
