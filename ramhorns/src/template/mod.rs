// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

mod parse;
mod section;

use std::borrow::Cow;
use std::hash::Hasher;
use std::fs::File;
use std::io;
use std::path::Path;

use crate::{Content, Error};
use crate::encoding::{Encoder, EscapingIOEncoder};

use fnv::FnvHasher;

pub use section::Section;

/// A preprocessed form of the plain text template, ready to be rendered
/// with data contained in types implementing the `Content` trait.
pub struct Template<'tpl> {
    /// Parsed blocks!
    blocks: Vec<Block<'tpl>>,

    /// Tailing html that isn't part of any `Block`
    capacity_hint: usize,

    /// Tailing html that isn't part of any `Block`
    tail: &'tpl str,

    /// Source from which this template was parsed.
    source: Cow<'tpl, str>,
}

impl<'tpl> Template<'tpl> {
    /// Create a new `Template` out of the source.
    ///
    /// + If `Source` is a `&str`, this `Template` will borrow it with appropriate lifetime.
    /// + If `Source` is a `String`, this `Template` will take it's ownership (The `'tpl` lifetime will be `'static`).
    pub fn new<Source>(source: Source) -> Result<Self, Error>
    where
        Source: Into<Cow<'tpl, str>>,
    {
        let mut tpl = Template {
            blocks: Vec::new(),
            capacity_hint: 0,
            tail: "",
            source: source.into(),
        };

        // This is allows `Block`s inside this `Template` to be references of the `source` field.
        // This is safe as long as the `source` field is never mutated or moved.
        let source: &'tpl str = unsafe {
            use std::{slice, str};

            let ptr = tpl.source.as_ptr();
            let len = tpl.source.len();

            str::from_utf8_unchecked(slice::from_raw_parts(ptr, len))
        };

        let mut iter = source.as_bytes()
            .get(..tpl.source.len().saturating_sub(1))
            .unwrap_or(&[])
            .iter()
            .map(|b| unsafe { &*(b as *const u8 as *const [u8; 2]) }) // Because we iterate up till last byte,
            .enumerate();                                             // this is safe.

        let mut last = 0;

        tpl.parse(source, &mut iter, &mut last, None)?;
        tpl.tail = &source[last..];

        Ok(tpl)
    }

    /// Estimate how big of a buffer should be allocated to render this `Template`.
    pub fn capacity_hint(&self) -> usize {
        self.capacity_hint + self.tail.len()
    }

    /// Render this `Template` with a given `Content` to a `String`.
    pub fn render<C: crate::Content>(&self, content: &C) -> String {
        let mut capacity = content.capacity_hint(self);

        // Add extra 25% extra capacity for HTML escapes and an odd double variable use.
        capacity += capacity / 4;

        let mut buf = String::with_capacity(capacity);

        // Ignore the result, cannot fail
        let _ = Section::new(&self.blocks).render_once(content, &mut buf);

        buf.push_str(self.tail);
        buf
    }

    /// Render this `Template` with a given `Content` to a writer.
    pub fn render_to_writer<W, C>(&self, writer: &mut W, content: &C) -> io::Result<()>
    where
        W: io::Write,
        C: Content,
    {
        let mut encoder = EscapingIOEncoder::new(writer);

        Section::new(&self.blocks).render_once(content, &mut encoder)?;

        encoder.write_unescaped(self.tail)
    }

    /// Render this `Template` with a given `Content` to a file.
    pub fn render_to_file<P, C>(&self, path: P, content: &C) -> io::Result<()>
    where
        P: AsRef<Path>,
        C: Content,
    {
        use io::BufWriter;

        let writer = BufWriter::new(File::create(path)?);
        let mut encoder = EscapingIOEncoder::new(writer);

        Section::new(&self.blocks).render_once(content, &mut encoder)?;

        encoder.write_unescaped(self.tail)
    }

    /// Get a reference to a source this `Template` was created from.
    pub fn source(&self) -> &str {
        &self.source
    }
}

impl Template<'static> {
    /// Create a template from a file.
    ///
    /// ```no_run
    /// # use ramhorns::Template;
    /// let tpl = Template::from_file("./templates/my_template.html").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        use io::Read;

        let mut file = File::open(path)?;
        let len = file.metadata()?.len();

        let mut buf = String::with_capacity(len as usize);

        file.read_to_string(&mut buf)?;

        Template::new(buf)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// `{{escaped}}` tag
    Escaped,

    /// `{{{unescaped}}}` tag
    Unescaped,

    /// `{{#section}}` opening tag (with number of subsequent blocks it contains)
    Section(usize),

    /// `{{^inverse}}` section opening tag (with number of subsequent blocks it contains)
    Inverse(usize),

    /// `{{/closing}}` section tag
    Closing,

    /// {{!comment}}` tag
    Comment,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Block<'tpl> {
    html: &'tpl str,
    name: &'tpl str,
    hash: u64,
    tag: Tag,
}

impl<'tpl> Block<'tpl> {
    fn new(html: &'tpl str, name: &'tpl str, tag: Tag) -> Self {
        let mut hasher = FnvHasher::default();

        hasher.write(name.as_bytes());

        let hash = hasher.finish();

        Block {
            html,
            name,
            hash,
            tag,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn template_from_string_is_static() {
        let tpl: Template<'static> = Template::new(String::from("Ramhorns")).unwrap();

        assert_eq!(tpl.source(), "Ramhorns");
    }

    #[test]
    fn block_hashes_correctly() {
        assert_eq!(Block::new("", "test", Tag::Escaped), Block {
            html: "",
            name: "test",
            hash: 0xf9e6e6ef197c2b25,
            tag: Tag::Escaped,
        });
    }

    #[test]
    fn constructs_blocks_correctly() {
        let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(&tpl.blocks, &[
            Block::new("<title>", "title", Tag::Escaped),
            Block::new("</title><h1>", "title", Tag::Escaped),
            Block::new("</h1><div>", "body", Tag::Unescaped),
        ]);

        assert_eq!(tpl.tail, "</div>");
    }

    #[test]
    fn constructs_nested_sections_correctly() {
        let source = "<body><h1>{{ title }}</h1>{{#posts}}<article>{{name}}</article>{{/posts}}{{^posts}}<p>Nothing here :(</p>{{/posts}}</body>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(&tpl.blocks, &[
            Block::new("<body><h1>", "title", Tag::Escaped),
            Block::new("</h1>", "posts", Tag::Section(2)),
            Block::new("<article>", "name", Tag::Escaped),
            Block::new("</article>", "posts", Tag::Closing),
            Block::new("", "posts", Tag::Inverse(1)),
            Block::new("<p>Nothing here :(</p>", "posts", Tag::Closing),
        ]);

        assert_eq!(tpl.tail, "</body>");
    }
}
