// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use std::io::{self, Write};

use crate::{Template, Section, Encoder};

pub trait Context: Sized {
    /// How much capacity is _likely_ required for all the data in this `Context`
    /// for a given `Template`.
    fn capacity_hint(&self, _tpl: &Template) -> usize {
        0
    }

    /// Marks whether this context is truthy when attempting to render a section.
    fn is_truthy(&self) -> bool { true }

    /// Render a section with self.
    fn render_section<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
        if self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a section with self.
    fn render_inverse<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
        if !self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a field, by the hash of it's name.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    fn render_field_escaped<W: Write>(&self, _hash: u64, _encoder: &mut Encoder<W>) -> io::Result<()> {
        Ok(())
    }

    /// Render a field, by the hash of it's name.
    ///
    /// This doesn't perform any escaping at all.
    fn render_field_unescaped<W: Write>(&self, _hash: u64, _encoder: &mut Encoder<W>) -> io::Result<()> {
        Ok(())
    }

    /// Render a field, by the hash of it's name, as a section.
    fn render_field_section<'section, W: Write>(&self, _hash: u64, _section: Section<'section>, _encoder: &mut Encoder<W>) -> io::Result<()> {
        Ok(())
    }

    /// Render a field, by the hash of it's name, as an inverse section.
    fn render_field_inverse<'section, W: Write>(&self, _hash: u64, _section: Section<'section>, _encoder: &mut Encoder<W>) -> io::Result<()> {
        Ok(())
    }
}

impl Context for bool {
    fn is_truthy(&self) -> bool { *self }
}

impl Context for &str {
    fn is_truthy(&self) -> bool { self.len() != 0 }
}

impl Context for String {
    fn is_truthy(&self) -> bool { self.len() != 0 }
}

impl<T: Context> Context for Option<T> {
    fn is_truthy(&self) -> bool { self.is_some() }

    fn render_section<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
        if let Some(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Context, E> Context for Result<T, E> {
    fn is_truthy(&self) -> bool { self.is_ok() }

    fn render_section<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
        if let Ok(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Context> Context for Vec<T> {
    fn is_truthy(&self) -> bool { self.len() != 0 }

    fn render_section<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Context> Context for &[T] {
    fn is_truthy(&self) -> bool { self.len() != 0 }

    fn render_section<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}
