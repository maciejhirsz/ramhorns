use std::io::{Write, Result};

pub struct Encoder<W: Write> {
    inner: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner
        }
    }

    pub fn write(&mut self, part: &str) -> Result<()> {
        self.inner.write_all(part.as_bytes())
    }

    pub fn write_escaped(&mut self, part: &str) -> Result<()> {
        let mut start = 0;

        for (idx, byte) in part.bytes().enumerate() {
            let replace: &[u8] = match byte {
                b'<' => b"&lt;",
                b'>' => b"&gt;",
                b'&' => b"&amp;",
                b'"' => b"&quot;",
                _ => continue,
            };

            self.inner.write_all(&part.as_bytes()[start..idx])?;
            self.inner.write_all(replace)?;

            start = idx + 1;
        }

        self.inner.write_all(&part.as_bytes()[start..])
    }
}
