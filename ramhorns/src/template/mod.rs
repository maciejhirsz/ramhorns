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
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io;
use std::path::{Path, PathBuf};

use crate::encoding::EscapingIOEncoder;
use crate::{Content, Error};

use fnv::FnvHasher;

pub use section::Section;

/// A preprocessed form of the plain text template, ready to be rendered
/// with data contained in types implementing the `Content` trait.
pub struct Template<'tpl> {
    /// Parsed blocks!
    blocks: Vec<Block<'tpl>>,

    /// Tailing html that isn't part of any `Block`
    capacity_hint: usize,

    /// Source from which this template was parsed.
    source: Cow<'tpl, str>,

    /// Partials used in this template
    partials: Vec<Cow<'tpl, str>>,
}

/// A safe wrapper around a `HashMap` containing preprocessed templates
/// of the type `Template`, accesible by their name
pub struct Templates {
    partials: HashMap<Cow<'static, str>, Template<'static>>,
    dir: PathBuf,
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
        Template::load(source, &mut NoPartials)
    }

    fn load<Source, T>(source: Source, partials: &mut T) -> Result<Self, Error>
    where
        Source: Into<Cow<'tpl, str>>,
        T: Partials<'tpl>,
    {
        let mut tpl = Template {
            blocks: Vec::new(),
            capacity_hint: 0,
            source: source.into(),
            partials: Vec::with_capacity(0),
        };

        // This is allows `Block`s inside this `Template` to be references of the `source` field.
        // This is safe as long as the `source` field is never mutated or moved.
        let source: &'tpl str = unsafe {
            use std::{slice, str};

            let ptr = tpl.source.as_ptr();
            let len = tpl.source.len();

            str::from_utf8_unchecked(slice::from_raw_parts(ptr, len))
        };

        let mut iter = source
            .as_bytes()
            .windows(2)
            .map(|b| unsafe { &*(b.as_ptr() as *const [u8; 2]) }) // windows iterator makes this safe
            .enumerate();

        let mut last = 0;

        tpl.parse(source, &mut iter, &mut last, None, partials)?;
        let tail = &source[last..].trim_end();
        tpl.blocks.push(Block::new(tail, "", Tag::Tail));
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
        let _ = Section::new(&self.blocks).render_once(content, &mut buf);

        buf
    }

    /// Render this `Template` with a given `Content` to a writer.
    pub fn render_to_writer<W, C>(&self, writer: &mut W, content: &C) -> io::Result<()>
    where
        W: io::Write,
        C: Content,
    {
        let mut encoder = EscapingIOEncoder::new(writer);
        Section::new(&self.blocks).render_once(content, &mut encoder)
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

        Section::new(&self.blocks).render_once(content, &mut encoder)
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
        let mut partials = Templates::new(
            path.as_ref()
                .parent()
                .unwrap_or_else(|| ".".as_ref())
                .canonicalize()?,
        );
        Template::load(std::fs::read_to_string(&path)?, &mut partials).map(move |mut tpl| {
            tpl.partials = partials.into_sources();
            tpl
        })
    }
}

impl Templates {
    fn new(dir: PathBuf) -> Self {
        Templates {
            partials: HashMap::new(),
            dir,
        }
    }

    /// Loads all the `.html` files as templates from the given folder into a hashmap, making them
    /// accessible via their path, joining partials as required
    /// ```no_run
    /// # use ramhorns::Templates;
    /// let tpls = Templates::from_folder("./templates").unwrap();
    /// let content = "I am the content";
    /// let rendered = tpls.get("hello.html").unwrap().render(&content);
    /// ```
    pub fn from_folder<P: AsRef<Path>>(dir: P) -> Result<Self, Error> {
        let mut templates = Templates::new(dir.as_ref().canonicalize()?);

        fn load_folder(path: &Path, templates: &mut Templates) -> Result<(), Error> {
            for entry in std::fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_dir() {
                    load_folder(&path, templates)?;
                } else if path.extension().unwrap_or("".as_ref()) == "html" {
                    let name = path
                        .strip_prefix(&templates.dir)
                        .unwrap_or(&path)
                        .to_string_lossy();
                    if !templates.partials.contains_key(&name) {
                        let template = Template::load(std::fs::read_to_string(&path)?, templates)?;
                        templates
                            .partials
                            .insert(name.into_owned().into(), template);
                    }
                }
            }
            Ok(())
        }
        load_folder(&templates.dir.clone(), &mut templates)?;

        Ok(templates)
    }

    /// Get the template with the given name, if it exists
    pub fn get<S>(&self, name: &S) -> Option<&Template<'static>>
    where
        for<'a> Cow<'a, str>: std::borrow::Borrow<S>,
        S: std::hash::Hash + Eq + ?Sized,
    {
        self.partials.get(name)
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

    /// `{{!comment}}` tag
    Comment,

    /// `{{>partial}}` tag
    Partial,

    /// Tailing html
    Tail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub(crate) trait Partials<'tpl> {
    fn get_partial(&mut self, name: &'tpl str) -> Result<&Template<'tpl>, Error>;
    fn into_sources(self) -> Vec<Cow<'tpl, str>>;
}

struct NoPartials;

impl<'tpl> Partials<'tpl> for NoPartials {
    fn get_partial(&mut self, _name: &'tpl str) -> Result<&Template<'tpl>, Error> {
        Err(Error::PartialsDisabled)
    }

    fn into_sources(self) -> Vec<Cow<'tpl, str>> {
        Vec::with_capacity(0)
    }
}

impl Partials<'static> for Templates {
    fn get_partial(&mut self, name: &'static str) -> Result<&Template<'static>, Error> {
        let path = self.dir.join(name).canonicalize()?;
        if !path.starts_with(&self.dir) {
            return Err(Error::IllegalPartial(name.into()));
        }
        if !self.partials.contains_key(name) {
            let template = Template::load(std::fs::read_to_string(&path)?, self)?;
            self.partials.insert(name.into(), template);
        };
        Ok(&self.partials[name])
    }

    fn into_sources(self) -> Vec<Cow<'static, str>> {
        self.partials
            .into_iter()
            .map(|(_, tpl)| tpl.source)
            .collect()
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
        assert_eq!(
            Block::new("", "test", Tag::Escaped),
            Block {
                html: "",
                name: "test",
                hash: 0xf9e6e6ef197c2b25,
                tag: Tag::Escaped,
            }
        );
    }

    #[test]
    fn constructs_blocks_correctly() {
        let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(
            &tpl.blocks,
            &[
                Block::new("<title>", "title", Tag::Escaped),
                Block::new("</title><h1>", "title", Tag::Escaped),
                Block::new("</h1><div>", "body", Tag::Unescaped),
                Block::new("</div>", "", Tag::Tail),
            ]
        );
    }

    #[test]
    fn constructs_nested_sections_correctly() {
        let source = "<body><h1>{{ title }}</h1>{{#posts}}<article>{{name}}</article>{{/posts}}{{^posts}}<p>Nothing here :(</p>{{/posts}}</body>";
        let tpl = Template::new(source).unwrap();

        assert_eq!(
            &tpl.blocks,
            &[
                Block::new("<body><h1>", "title", Tag::Escaped),
                Block::new("</h1>", "posts", Tag::Section(2)),
                Block::new("<article>", "name", Tag::Escaped),
                Block::new("</article>", "posts", Tag::Closing),
                Block::new("", "posts", Tag::Inverse(1)),
                Block::new("<p>Nothing here :(</p>", "posts", Tag::Closing),
                Block::new("</body>", "", Tag::Tail),
            ]
        );
    }
}
