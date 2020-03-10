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
//! Fastest [**Mustache**](https://mustache.github.io/) template engine implementation
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

mod cmark;
mod content;
mod error;
mod template;
pub mod traits;

pub mod encoding;

pub use content::Content;
pub use error::Error;
pub use template::{Section, Template, Templates};

#[cfg(feature = "export_derive")]
pub use ramhorns_derive::Content;
