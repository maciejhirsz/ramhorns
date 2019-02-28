//! Utilities dealing with writing the bits of a template or data to the output and
//! escaping special HTML characters.

use std::io;
use std::fmt;

/// A trait that wraps around either a `String` or `std::io::Write`, providing UTF-8 safe
/// writing boundaries and special HTML character escaping.
pub trait Encoder {
    /// Error type for this encoder
    type Error;

    /// Write a `&str` to this `Encoder` in plain mode.
    fn write_unescaped(&mut self, part: &str) -> Result<(), Self::Error>;

    /// Write a `&str` to this `Encoder`, escaping special HTML characters.
    fn write_escaped(&mut self, part: &str) -> Result<(), Self::Error>;

    /// Write a `Display` implementor to this `Encoder` in plain mode.
    fn format_unescaped<D: fmt::Display>(&mut self, display: D) -> Result<(), Self::Error>;

    /// Write a `Display` implementor to this `Encoder`, escaping special HTML characters.
    fn format_escaped<D: fmt::Display>(&mut self, display: D) -> Result<(), Self::Error>;
}

/// Local helper for escaping stuff into strings.
struct EscapingStringEncoder<'a>(&'a mut String);

impl<'a> EscapingStringEncoder<'a> {
    /// Write with escaping special HTML characters. Since we are dealing
    /// with a String, we don't need to return a `Result`.
    fn write_escaped(&mut self, part: &str) {
        let mut start = 0;

        for (idx, byte) in part.bytes().enumerate() {
            let replace = match byte {
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'&' => "&amp;",
                b'"' => "&quot;",
                _ => continue,
            };

            self.0.push_str(&part[start..idx]);
            self.0.push_str(replace);

            start = idx + 1;
        }

        self.0.push_str(&part[start..]);
    }
}

/// Provide a `fmt::Write` interface, so we can use `write!` macro.
impl<'a> fmt::Write for EscapingStringEncoder<'a> {
    fn write_str(&mut self, part: &str) -> fmt::Result {
        self.write_escaped(part);

        Ok(())
    }
}

/// Encoder wrapper around io::Write. We can't implement `Encoder` on a generic here,
/// because we're implementing it directly for `String`.
pub(crate) struct EscapingIOEncoder<W: io::Write> {
    inner: W,
}

impl<W: io::Write> EscapingIOEncoder<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner
        }
    }

    /// Same as `EscapingStringEncoder`, but dealing with byte arrays and writing to
    /// the inner `io::Write`.
    fn write_escaped_bytes(&mut self, part: &[u8]) -> io::Result<()> {
        let mut start = 0;

        for (idx, byte) in part.iter().enumerate() {
            let replace: &[u8] = match *byte {
                b'<' => b"&lt;",
                b'>' => b"&gt;",
                b'&' => b"&amp;",
                b'"' => b"&quot;",
                _ => continue,
            };

            self.inner.write_all(&part[start..idx])?;
            self.inner.write_all(replace)?;

            start = idx + 1;
        }

        self.inner.write_all(&part[start..])
    }
}

// Additionally we implement `io::Write` for it directly. This allows us to use
// the `write!` macro for formatting without allocations.
impl<W: io::Write> io::Write for EscapingIOEncoder<W> {
    fn write(&mut self, part: &[u8]) -> io::Result<usize> {
        self.write_escaped_bytes(part).map(|()| part.len())
    }

    fn write_all(&mut self, part: &[u8]) -> io::Result<()> {
        self.write_escaped_bytes(part)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<W: io::Write> Encoder for EscapingIOEncoder<W> {
    type Error = io::Error;

    fn write_unescaped(&mut self, part: &str) -> io::Result<()> {
        self.inner.write_all(part.as_bytes())
    }

    fn write_escaped(&mut self, part: &str) -> io::Result<()> {
        self.write_escaped_bytes(part.as_bytes())
    }

    fn format_unescaped<D: fmt::Display>(&mut self, display: D) -> Result<(), Self::Error> {
        write!(self.inner, "{}", display)
    }

    fn format_escaped<D: fmt::Display>(&mut self, display: D) -> Result<(), Self::Error> {
        use io::Write;

        write!(self, "{}", display)
    }
}

impl Encoder for String {
    // Change this to `!` once stabilized.
    type Error = ();

    fn write_unescaped(&mut self, part: &str) -> Result<(), Self::Error> {
        self.push_str(part);

        Ok(())
    }

    fn write_escaped(&mut self, part: &str) -> Result<(), Self::Error> {
        EscapingStringEncoder(self).write_escaped(part);

        Ok(())
    }

    fn format_unescaped<D: fmt::Display>(&mut self, display: D) -> Result<(), Self::Error> {
        use std::fmt::Write;

        // Never fails for a string
        let _ = write!(self, "{}", display);

        Ok(())
    }

    fn format_escaped<D: fmt::Display>(&mut self, display: D) -> Result<(), Self::Error> {
        use std::fmt::Write;

        // Never fails for a string
        let _ = write!(EscapingStringEncoder(self), "{}", display);

        Ok(())
    }
}
