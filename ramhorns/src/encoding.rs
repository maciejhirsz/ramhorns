use std::io::{self, Write};

#[derive(Clone, Copy, Debug)]
pub struct EncodingError;

impl From<io::Error> for EncodingError {
    fn from(_: io::Error) -> Self {
        Self
    }
}

pub type Result = std::result::Result<(), EncodingError>;

pub trait Encoder {
    fn write(&mut self, part: &str) -> Result;

    fn write_escaped(&mut self, part: &str) -> Result {
        let mut start = 0;

        for (idx, byte) in part.bytes().enumerate() {
            let replace = match byte {
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'&' => "&amp;",
                b'"' => "&quot;",
                _ => continue,
            };

            self.write(&part[start..idx])?;
            self.write(replace)?;

            start = idx + 1;
        }

        self.write(&part[start..])
    }
}

pub(crate) struct IOEncoder<W: Write> {
    inner: W
}

impl<W: Write> IOEncoder<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner
        }
    }
}

impl<W: Write> Encoder for IOEncoder<W> {
    fn write(&mut self, part: &str) -> Result {
        self.inner.write_all(part.as_bytes()).map_err(Into::into)
    }
}

impl Encoder for String {
    fn write(&mut self, part: &str) -> Result {
        self.push_str(part);

        Ok(())
    }
}
