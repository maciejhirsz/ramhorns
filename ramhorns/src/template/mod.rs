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

use std::hash::Hasher;
use std::io;

use fnv::FnvHasher;

use crate::Context;
use crate::encoding::{Encoder, EscapingIOEncoder};

pub use section::Section;

/// A preprocessed form of the plain text template, ready to be rendered
/// with data contained in types implementing the `Context` trait.
pub struct Template<'tpl> {
    /// Parsed blocks!
    blocks: Vec<Block<'tpl>>,

    /// Tailing html that isn't part of any `Block`
    capacity_hint: usize,

    /// Tailing html that isn't part of any `Block`
    tail: &'tpl str,
}

impl<'tpl> Template<'tpl> {
    /// Create a new `Template` out of the source.
    pub fn new(source: &'tpl str) -> Self {
        let mut iter = source.as_bytes()
            .get(..source.len() - 1)
            .unwrap_or(&[])
            .iter()
            .map(|b| unsafe { &*(b as *const u8 as *const [u8; 2]) })
            .enumerate();

        let mut tpl = Template {
            blocks: Vec::new(),
            capacity_hint: 0,
            tail: "",
        };

        let mut last = 0;

        tpl.parse(source, &mut iter, &mut last, None);
        tpl.tail = &source[last..];

        tpl
    }

    /// Estimate how big of a buffer should be allocated to render this `Template`.
    pub fn capacity_hint(&self) -> usize {
        self.capacity_hint + self.tail.len()
    }

    /// Render this `Template` with a given `Context` to a writer. This is useful if you
    /// want to render templates directly to files or network.
    pub fn render_to_writer<C, W>(&self, ctx: &C, writer: &mut W) -> io::Result<()>
    where
        C: Context,
        W: io::Write,
    {
        let mut encoder = EscapingIOEncoder::new(writer);

        Section::new(&self.blocks).render_once(ctx, &mut encoder)?;

        encoder.write_unescaped(self.tail)
    }

    /// Render this `Template` with a given `Context` to a `String`.
    pub fn render<C: crate::Context>(&self, ctx: &C) -> String {
        let mut capacity = ctx.capacity_hint(self);

        // Add extra 25% extra capacity for HTML escapes and an odd double variable use.
        capacity += capacity / 4;

        let mut buf = String::with_capacity(capacity);

        // Ignore the result, cannot fail
        let _ = Section::new(&self.blocks).render_once(ctx, &mut buf);

        buf.push_str(self.tail);
        buf
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
        let tpl = Template::new(source);

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
        let tpl = Template::new(source);


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
