// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

//! <img src="https://raw.githubusercontent.com/maciejhirsz/ramhorns/master/ramhorns.svg?sanitize=true" alt="Ramhorns logo" width="250" align="right" style="background: #fff; margin: 0 0 1em 1em;">
//!
//! # Ramhorns
//!
//! Fast [**Mustache**](https://mustache.github.io/) template engine implementation
//! in pure Rust.
//!
//! **Ramhorns** loads and processes templates **at runtime**. It comes with a derive macro
//! which allows for templates to be rendered from native Rust data structures without doing
//! temporary allocations, intermediate `HashMap`s or what have you.
//!
//! With a touch of magic 🎩, the power of friendship 🥂, and a sparkle of
//! [FNV hashing](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function)
//! ✨, render times easily compete with static template engines like
//! [**Askama**](https://github.com/djc/askama).
//!
//! What else do you want, a sticker?
//!
//! ## Example
//!
//! ```rust
//! use ramhorns::{Template, Content};
//!
//! #[derive(Content)]
//! struct Post<'a> {
//!     title: &'a str,
//!     teaser: &'a str,
//! }
//!
//! #[derive(Content)]
//! struct Blog<'a> {
//!     title: String,        // Strings are cool
//!     posts: Vec<Post<'a>>, // &'a [Post<'a>] would work too
//! }
//!
//! // Standard Mustache action here
//! let source = "<h1>{{title}}</h1>\
//!               {{#posts}}<article><h2>{{title}}</h2><p>{{teaser}}</p></article>{{/posts}}\
//!               {{^posts}}<p>No posts yet :(</p>{{/posts}}";
//!
//! let tpl = Template::new(source).unwrap();
//!
//! let rendered = tpl.render(&Blog {
//!     title: "My Awesome Blog!".to_string(),
//!     posts: vec![
//!         Post {
//!             title: "How I tried Ramhorns and found love 💖",
//!             teaser: "This can happen to you too",
//!         },
//!         Post {
//!             title: "Rust is kinda awesome",
//!             teaser: "Yes, even the borrow checker! 🦀",
//!         },
//!     ]
//! });
//!
//! assert_eq!(rendered, "<h1>My Awesome Blog!</h1>\
//!                       <article>\
//!                           <h2>How I tried Ramhorns and found love 💖</h2>\
//!                           <p>This can happen to you too</p>\
//!                       </article>\
//!                       <article>\
//!                           <h2>Rust is kinda awesome</h2>\
//!                           <p>Yes, even the borrow checker! 🦀</p>\
//!                       </article>");
//! ```

#![warn(missing_docs)]

use std::path::{Path, PathBuf};

use beef::Cow;
use ahash::AHashMap as HashMap;

mod cmark;
mod content;
mod error;
mod template;
pub mod traits;

pub mod encoding;

pub use content::Content;
pub use error::Error;
pub use template::{Section, Template};

#[cfg(feature = "export_derive")]
pub use ramhorns_derive::Content;

/// Aggregator for [`Template`s](./struct.Template.html), that allows them to
/// be loaded from the file system and use partials: `{{>partial}}`
pub struct Ramhorns {
    partials: HashMap<Cow<'static, str>, Template<'static>>,
    dir: PathBuf,
}

impl Ramhorns {
    /// Loads all the `.html` files as templates from the given folder, making them
    /// accessible via their path, joining partials as required.
    /// ```no_run
    /// # use ramhorns::Ramhorns;
    /// let tpls = Ramhorns::from_folder("./templates").unwrap();
    /// let content = "I am the content";
    /// let rendered = tpls.get("hello.html").unwrap().render(&content);
    /// ```
    pub fn from_folder<P: AsRef<Path>>(dir: P) -> Result<Self, Error> {
        let mut templates = Ramhorns::lazy(dir)?;

        fn load_folder(path: &Path, templates: &mut Ramhorns) -> Result<(), Error> {
            for entry in std::fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_dir() {
                    load_folder(&path, templates)?;
                } else if path.extension().unwrap_or_else(|| "".as_ref()) == "html" {
                    let name = path
                        .strip_prefix(&templates.dir)
                        .unwrap_or(&path)
                        .to_string_lossy();

                    if !templates.partials.contains_key(&*name) {
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

    /// Create a new empty aggregator for a given folder. This won't do anything until
    /// a template has been added using [`from_file`](#method.from_file).
    /// ```no_run
    /// # use ramhorns::Ramhorns;
    /// let mut tpls = Ramhorns::lazy("./templates").unwrap();
    /// let content = "I am the content";
    /// let rendered = tpls.from_file("hello.html").unwrap().render(&content);
    /// ```
    pub fn lazy<P: AsRef<Path>>(dir: P) -> Result<Self, Error> {
        Ok(Ramhorns {
            partials: HashMap::default(),
            dir: dir.as_ref().canonicalize()?,
        })
    }

    /// Get the template with the given name, if it exists.
    pub fn get<S>(&self, name: &S) -> Option<&Template<'static>>
    where
        for<'a> Cow<'a, str>: std::borrow::Borrow<S>,
        S: std::hash::Hash + AsRef<Path> + Eq + ?Sized,
    {
        self.partials.get(name)
    }

    /// Get the template with the given name. If the template doesn't exist,
    /// it will be loaded from file and parsed first.
    ///
    /// Use this method in tandem with [`lazy`](#method.lazy).
    pub fn from_file(&mut self, name: &str) -> Result<&Template<'static>, Error> {
        self.load_internal(name.to_owned().into())
    }

    fn load_internal(&mut self, name: Cow<'static, str>) -> Result<&Template<'static>, Error> {
        let n: &str = &name;
        if !self.partials.contains_key(n) {
            let path = self.dir.join(n).canonicalize()?;

            if !path.starts_with(&self.dir) {
                return Err(Error::IllegalPartial(n.into()));
            }
            let template = Template::load(std::fs::read_to_string(&path)?, self)?;
            self.partials.insert(name.clone(), template);
        };
        Ok(&self.partials[n])
    }
}

pub(crate) trait Partials<'tpl> {
    fn get_partial(&mut self, name: &'tpl str) -> Result<&Template<'tpl>, Error>;
}

impl Partials<'static> for Ramhorns {
    fn get_partial(&mut self, name: &'static str) -> Result<&Template<'static>, Error> {
        self.load_internal(Cow::borrowed(name))
    }
}