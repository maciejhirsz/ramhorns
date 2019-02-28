// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use crate::{Template, Section};
use crate::encoding::{self as en, Encoder};

pub trait Context: Sized {
    /// How much capacity is _likely_ required for all the data in this `Context`
    /// for a given `Template`.
    fn capacity_hint(&self, _tpl: &Template) -> usize {
        0
    }

    /// Marks whether this context is truthy when attempting to render a section.
    fn is_truthy(&self) -> bool { true }

    /// Render a section with self.
    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        if self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a section with self.
    fn render_inverse<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        if !self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a field, by the hash of it's name.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    fn render_field_escaped<E>(&self, _hash: u64, _encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field, by the hash of it's name.
    ///
    /// This doesn't perform any escaping at all.
    fn render_field_unescaped<E>(&self, _hash: u64, _encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field, by the hash of it's name, as a section.
    fn render_field_section<'section, E>(&self, _hash: u64, _section: Section<'section>, _encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field, by the hash of it's name, as an inverse section.
    fn render_field_inverse<'section, E>(&self, _hash: u64, _section: Section<'section>, _encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
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

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        if let Some(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Context, U> Context for Result<T, U> {
    fn is_truthy(&self) -> bool { self.is_ok() }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        if let Ok(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Context> Context for Vec<T> {
    fn is_truthy(&self) -> bool { self.len() != 0 }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Context> Context for &[T] {
    fn is_truthy(&self) -> bool { self.len() != 0 }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> en::Result
    where
        E: Encoder,
    {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}
