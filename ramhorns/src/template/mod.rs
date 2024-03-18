// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::Path;

use beef::Cow;
use fnv::FnvHasher;

use crate::encoding::EscapingIOEncoder;
use crate::Partials;
use crate::{Content, Error};

mod parse;
mod section;

#[cfg(not(feature = "indexes"))]
pub use parse::Tag;
#[cfg(feature = "indexes")]
pub use parse::{Index::Last, Indexed, Tag};
pub use section::Section;

/// A preprocessed form of the plain text template, ready to be rendered
/// with data contained in types implementing the `Content` trait.
pub struct Template<'tpl> {
    /// Parsed blocks!
    blocks: Vec<Block<'tpl>>,

    /// Total byte length of all the blocks, used to estimate preallocations.
    capacity_hint: usize,

    /// Source from which this template was parsed.
    source: Cow<'tpl, str>,
}

impl<'tpl> fmt::Debug for Template<'tpl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Template")
            .field("source", &self.source)
            .finish()
    }
}

impl<'tpl> Template<'tpl> {
    /// Create a new `Template` out of the source.
    ///
    /// + If `source` is a `&str`, this `Template` will borrow it with appropriate lifetime.
    /// + If `source` is a `String`, this `Template` will take it's ownership (The `'tpl` lifetime will be `'static`).
    pub fn new<S>(source: S) -> Result<Self, Error>
    where
        S: Into<Cow<'tpl, str>>,
    {
        Template::load(source, &mut NoPartials)
    }

    pub(crate) fn load<S>(source: S, partials: &mut impl Partials<'tpl>) -> Result<Self, Error>
    where
        S: Into<Cow<'tpl, str>>,
    {
        let source = source.into();

        // This is allows `Block`s inside this `Template` to be references of the `source` field.
        // This is safe as long as the `source` field is never mutated or dropped.
        let unsafe_source: &'tpl str = unsafe { &*(&*source as *const str) };

        let mut tpl = Template {
            blocks: Vec::with_capacity(16),
            capacity_hint: 0,
            source,
        };

        let last = tpl.parse(unsafe_source, partials)?;
        let tail = &unsafe_source[last..].trim_end();
        tpl.blocks.push(Block::nameless(tail, Tag::Tail));
        tpl.capacity_hint += tail.len();

        Ok(tpl)
    }

    /// Estimate how big of a buffer should be allocated to render this `Template`.
    pub fn capacity_hint(&self) -> usize {
        self.capacity_hint
    }

    /// Render this `Template` with a given `Content` to a `String`.
    pub fn render<C: crate::Content>(&self, content: &C) -> String {
        let mut capacity = content.capacity_hint(self);

        // Add extra 25% extra capacity for HTML escapes and an odd double variable use.
        capacity += capacity / 4;

        let mut buf = String::with_capacity(capacity);

        // Ignore the result, cannot fail
        let _ = Section::new(&self.blocks).with(content).render(&mut buf);

        buf
    }

    /// Render this `Template` with a given `Content` to a writer.
    pub fn render_to_writer<W, C>(&self, writer: &mut W, content: &C) -> io::Result<()>
    where
        W: io::Write,
        C: Content,
    {
        let mut encoder = EscapingIOEncoder::new(writer);
        Section::new(&self.blocks)
            .with(content)
            .render(&mut encoder)
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

        Section::new(&self.blocks)
            .with(content)
            .render(&mut encoder)
    }

    /// Get a reference to a source this `Template` was created from.
    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Block<'tpl> {
    html: &'tpl str,
    name: &'tpl str,
    hash: u64,
    tag: Tag,
    children: u32,
}

#[inline]
pub(crate) fn hash_name(name: &str) -> u64 {
    let mut hasher = FnvHasher::default();
    name.hash(&mut hasher);
    hasher.finish()
}

impl<'tpl> Block<'tpl> {
    #[inline]
    fn new(html: &'tpl str, name: &'tpl str, tag: Tag) -> Self {
        Block {
            html,
            name,
            hash: hash_name(name),
            tag,
            children: 0,
        }
    }

    // Skips hashing; can be used when tag is Partial, Comment or Tail
    #[inline]
    fn nameless(html: &'tpl str, tag: Tag) -> Self {
        Block {
            html,
            name: "",
            hash: 0,
            tag,
            children: 0,
        }
    }
    /// Get the index if this block refers to a section index.
    #[cfg(feature = "indexes")]
    #[inline]
    pub(crate) fn index(&self) -> Option<&Indexed> {
        match &self.tag {
            Tag::Indexed(indexed) => Some(indexed),
            _ => None,
        }
    }
}

struct NoPartials;

impl<'tpl> Partials<'tpl> for NoPartials {
    fn get_partial(&mut self, _name: &'tpl str) -> Result<&Template<'tpl>, Error> {
        Err(Error::PartialsDisabled)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    impl Block<'_> {
        fn children(self, children: u32) -> Self {
            Block { children, ..self }
        }
    }

    #[test]
    fn template_from_string_is_static() {
        let tpl: Template<'static> = Template::new(String::from("Ramhorns")).unwrap();

        assert_eq!(tpl.source(), "Ramhorns");
    }

    #[test]
    fn block_hashes_correctly() {
        assert_eq!(
            Block::new("", "test", Tag::Escaped),
            Block {
                html: "",
                name: "test",
                hash: 2271575940368597870,
                tag: Tag::Escaped,
                children: 0,
            }
        );
    }

    #[test]
    fn constructs_blocks_correctly() {
        let source = "<title>{{title}}</title><h1>{{title}}</h1><div>{{{body}}}</div>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(
            &tpl.blocks,
            &[
                Block::new("<title>", "title", Tag::Escaped),
                Block::new("</title><h1>", "title", Tag::Escaped),
                Block::new("</h1><div>", "body", Tag::Unescaped),
                Block::nameless("</div>", Tag::Tail),
            ]
        );
    }

    #[cfg(feature = "indexes")]
    #[test]
    fn blocks() {
        let source =
            "{{#person}}{{^-last}}{{{name}}}{{/-last}}{{#-last}}{{{name}}}{{/-last}}{{/person}}";
        let tpl = Template::new(source).unwrap();

        assert_eq!(
            &tpl.blocks,
            &[
                Block::new("", "person", Tag::Section).children(7),
                Block::new("", "-last", Tag::Indexed(Indexed::Exclude(Last))).children(2),
                Block::new("", "name", Tag::Unescaped),
                Block::nameless("", Tag::Closing),
                Block::new("", "-last", Tag::Indexed(Indexed::Include(Last))).children(2),
                Block::new("", "name", Tag::Unescaped),
                Block::nameless("", Tag::Closing),
                Block::nameless("", Tag::Closing),
                Block::nameless("", Tag::Tail),
            ]
        );
    }

    #[test]
    fn constructs_nested_sections_correctly() {
        let source = "<body><h1>{{title}}</h1>{{#posts}}<article>{{name}}</article>{{/posts}}{{^posts}}<p>Nothing here :(</p>{{/posts}}</body>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(
            &tpl.blocks,
            &[
                Block::new("<body><h1>", "title", Tag::Escaped),
                Block::new("</h1>", "posts", Tag::Section).children(2),
                Block::new("<article>", "name", Tag::Escaped),
                Block::nameless("</article>", Tag::Closing),
                Block::new("", "posts", Tag::Inverse).children(1),
                Block::nameless("<p>Nothing here :(</p>", Tag::Closing),
                Block::nameless("</body>", Tag::Tail),
            ]
        );
    }

    #[test]
    fn constructs_nested_sections_with_dot_correctly() {
        let source = "<body><h1>{{site title}}</h1>{{^archive posts}}<article>{{name}}</article>{{/posts archive}}</body>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(
            &tpl.blocks,
            &[
                Block::new("<body><h1>", "site", Tag::Section).children(1),
                Block::new("", "title", Tag::Escaped),
                Block::new("</h1>", "archive", Tag::Section).children(3),
                Block::new("", "posts", Tag::Inverse).children(2),
                Block::new("<article>", "name", Tag::Escaped),
                Block::nameless("</article>", Tag::Closing),
                Block::nameless("</body>", Tag::Tail),
            ]
        );
    }
}
