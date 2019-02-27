// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use std::io::{Write, Result};

use crate::{Template, Encoder};

pub trait Context {
    const FIELDS: &'static [&'static str];

    /// How much capacity is required for all the data in this `Context`.
    /// for a given `Template`.
    fn capacity_hint(&self, tpl: &Template) -> usize;

    /// Render a field, by the hash of it's name, into the buffer.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`
    fn render_escaped<W: Write>(&self, hash: u64, encoder: &mut Encoder<W>) -> Result<()>;

    /// Render a field, by the hash of it's name, into the buffer.
    ///
    /// This doesn't perform any escaping at all.
    fn render_unescaped<W: Write>(&self, hash: u64, encoder: &mut Encoder<W>) -> Result<()>;

    // /// Render a field, by the hash of it's name, as a section into the buffer.
    // fn render_section(&self, hash: u64, section: &Section, buf: &mut String) {}

    // /// Render a field, by the hash of it's name, as an inverse section into the buffer.
    // fn render_inverse(&self, hash: u64, section: &Section, buf: &mut String) {}
}
