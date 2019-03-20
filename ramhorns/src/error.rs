// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use std::{io, fmt};

/// Error type used that can be emitted during template parsing.
#[derive(Debug)]
pub enum Error {
    /// There was an error with the IO (only happens when parsing a file)
    Io(io::Error),

    /// Parser was expecting a tag closing a section `{{/foo}}`,
    /// but never found it or found a different one.
    UnclosedSection(Box<str>),

    /// Parser was expecting to find the closing braces of a tag `}}`, but never found it.
    UnclosedTag,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => {
                err.fmt(f)
            },
            Error::UnclosedSection(name) => {
                write!(f, "Section not closed properly, was expecting {{{{/{}}}}}", name)
            },
            Error::UnclosedTag => {
                write!(f, "Couldn't find closing braces matching opening braces")
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn displays_properly() {
        assert_eq!(Error::UnclosedSection("foo".into()).to_string(), "Section not closed properly, was expecting {{/foo}}");
        assert_eq!(Error::UnclosedTag.to_string(), "Couldn't find closing braces matching opening braces");
    }
}
