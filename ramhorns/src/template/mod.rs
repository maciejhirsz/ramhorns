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
use std::io::{self, Write};

use fnv::FnvHasher;

use crate::Encoder;

pub use section::Section;

pub struct Template<'tpl> {
    /// Parsed blocks!
    blocks: Vec<Block<'tpl>>,

    /// Tailing html that isn't part of any `Block`
    capacity_hint: usize,

    /// Tailing html that isn't part of any `Block`
    tail: &'tpl str,
}

impl<'tpl> Template<'tpl> {
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

    pub fn capacity_hint(&self) -> usize {
        self.capacity_hint + self.tail.len()
    }

    pub fn render_to<Context, Writer>(&self, ctx: &Context, writer: Writer) -> io::Result<()>
    where
        Context: crate::Context,
        Writer: Write,
    {
        let mut encoder = Encoder::new(writer);

        Section::new(&self.blocks).render_once(ctx, &mut encoder)?;

        encoder.write(self.tail)
    }

    pub fn render<Context: crate::Context>(&self, ctx: &Context) -> String {
        let mut capacity = ctx.capacity_hint(self);

        // Add extra 25% extra capacity for HTML escapes and an odd double variable use.
        capacity += capacity / 4;

        let mut buf = Vec::with_capacity(capacity);

        // Ignore the result, cannot fail
        let _ = self.render_to(ctx, &mut buf);

        unsafe {
            // Encoder guarantees that all writes are UTF8
            String::from_utf8_unchecked(buf)
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Escaped,
    Unescaped,
    Section(usize),
    Inverse(usize),
    Closing,
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
