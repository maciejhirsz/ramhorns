// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

//! # [WIP] Ramhorns
//!
//! Experimental [**`{{ mustache }}`**](https://mustache.github.io/)-ish implementation.
//!
//! **Ramhorns** loads and processes templates **at runtime**. It comes with a derive
//! macro for structs which allows templates to be rendered from native Rust data
//! structures without doing temporary allocations, intermediate hashmaps and what
//! have you.
//!
//! ```rust
//! use ramhorns::{Template, Context};
//!
//! #[derive(Context)]
//! struct Post<'a> {
//!     title: &'a str,
//!     body: &'a str,
//! }
//!
//! let tpl = Template::new("<h1>{{title}}</h1><div>{{body}}</div>");
//!
//! let rendered = tpl.render(&Post {
//!     title: "Hello Ramhorns",
//!     body: "Well, that was easy!",
//! });
//!
//! assert_eq!(rendered, "<h1>Hello Ramhorns</h1><div>Well, that was easy!</div>")
//! ```

mod template;
mod context;
mod encoder;

pub use template::{Template, Section};
pub use context::Context;
pub use encoder::Encoder;

#[cfg(feature = "export_derive")]
pub use ramhorns_derive::Context;

/// Utility for writing slices into a buffer, escaping HTML characters
pub fn escape(buf: &mut String, part: &str) {
    let mut start = 0;

    for (idx, byte) in part.bytes().enumerate() {
        let replace = match byte {
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'&' => "&amp;",
            b'"' => "&quot;",
            _ => continue,
        };

        buf.push_str(&part[start..idx]);
        buf.push_str(replace);

        start = idx + 1;
    }

    buf.push_str(&part[start..]);
}
