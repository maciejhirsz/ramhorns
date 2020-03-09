// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use crate::encoding::Encoder;
use crate::{Section, Template};

use std::borrow::{Borrow, Cow, ToOwned};
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};
use std::ops::Deref;

/// Trait allowing the rendering to quickly access data stored in the type that
/// implements it. You needn't worry about implementing it, in virtually all
/// cases the `#[derive(Content)]` attribute above your types should be sufficient.
pub trait Content {
    /// Marks whether this content is truthy. Used when attempting to render a section.
    fn is_truthy(&self) -> bool {
        true
    }

    /// How much capacity is _likely_ required for all the data in this `Content`
    /// for a given `Template`.
    fn capacity_hint(&self, _tpl: &Template) -> usize {
        0
    }

    /// Renders self as a variable to the encoder.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    fn render_escaped<E: Encoder>(&self, _encoder: &mut E) -> Result<(), E::Error> {
        Ok(())
    }

    /// Renders self as a variable to the encoder.
    ///
    /// This doesn't perform any escaping at all.
    fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.render_escaped(encoder)
    }

    /// Renders self as a variable to the encoder with CommonMark processing.
    ///
    /// The generated HTML is never escaped.
    fn render_cmark<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.render_escaped(encoder)
    }

    /// Render a section with self.
    fn render_section<'section, P, E>(
        &self,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
        Self: Sized,
    {
        if self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a section with self.
    fn render_inverse<'section, P, E>(
        &self,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
        Self: Sized,
    {
        if !self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a field by the hash **or** string of its name.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    /// If successful, returns `true` if the field exists in this content, otherwise `false`.
    fn render_field_escaped<E: Encoder>(
        &self,
        _hash: u64,
        _name: &str,
        _encoder: &mut E,
    ) -> Result<bool, E::Error> {
        Ok(false)
    }

    /// Render a field by the hash **or** string of its name.
    ///
    /// This doesn't perform any escaping at all.
    /// If successful, returns `true` if the field exists in this content, otherwise `false`.
    fn render_field_unescaped<E: Encoder>(
        &self,
        _hash: u64,
        _name: &str,
        _encoder: &mut E,
    ) -> Result<bool, E::Error> {
        Ok(false)
    }

    /// Render a field by the hash **or** string of its name, as a section.
    /// If successful, returns `true` if the field exists in this content, otherwise `false`.
    fn render_field_section<'section, P, E>(
        &self,
        _hash: u64,
        _name: &str,
        mut _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        Ok(false)
    }

    /// Render a field, by the hash of **or** string its name, as an inverse section.
    /// If successful, returns `true` if the field exists in this content, otherwise `false`.
    fn render_field_inverse<'section, P, E>(
        &self,
        _hash: u64,
        _name: &str,
        mut _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        Ok(false)
    }
}

impl Content for () {}

impl Content for str {
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        self.len()
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.write_escaped(self)
    }

    fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.write_unescaped(self)
    }

    fn render_cmark<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        crate::cmark::encode(self, encoder)
    }
}

impl Content for String {
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        self.len()
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.write_escaped(self)
    }

    fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.write_unescaped(self)
    }

    fn render_cmark<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        crate::cmark::encode(self, encoder)
    }
}

impl Content for bool {
    fn is_truthy(&self) -> bool {
        *self
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        5
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        // Nothing to escape here
        encoder.write_unescaped(if *self { "true" } else { "false" })
    }
}

macro_rules! impl_number_types {
    ($( $ty:ty ),*) => {
        $(
            impl Content for $ty {
                fn is_truthy(&self) -> bool {
                    *self != 0 as $ty
                }

                fn capacity_hint(&self, _tpl: &Template) -> usize {
                    5
                }

                fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error>
                {
                    // Nothing to escape here
                    encoder.format_unescaped(self)
                }
            }
        )*
    }
}

impl_number_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl Content for f32 {
    fn is_truthy(&self) -> bool {
        // Floats shoudn't be directly compared to 0
        self.abs() > std::f32::EPSILON
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        5
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        // Nothing to escape here
        encoder.format_unescaped(self)
    }
}

impl Content for f64 {
    fn is_truthy(&self) -> bool {
        // Floats shoudn't be directly compared to 0
        self.abs() > std::f64::EPSILON
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        5
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        // Nothing to escape here
        encoder.format_unescaped(self)
    }
}

impl<T: Content> Content for Option<T> {
    fn is_truthy(&self) -> bool {
        self.is_some()
    }

    fn capacity_hint(&self, tpl: &Template) -> usize {
        match self {
            Some(inner) => inner.capacity_hint(tpl),
            _ => 0,
        }
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        if let Some(inner) = self {
            inner.render_escaped(encoder)?;
        }

        Ok(())
    }

    fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        if let Some(ref inner) = self {
            inner.render_unescaped(encoder)?;
        }

        Ok(())
    }

    fn render_section<'section, P, E>(
        &self,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        if let Some(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Content, U> Content for Result<T, U> {
    fn is_truthy(&self) -> bool {
        self.is_ok()
    }

    fn capacity_hint(&self, tpl: &Template) -> usize {
        match self {
            Ok(inner) => inner.capacity_hint(tpl),
            _ => 0,
        }
    }

    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        if let Ok(inner) = self {
            inner.render_escaped(encoder)?;
        }

        Ok(())
    }

    fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        if let Ok(ref inner) = self {
            inner.render_unescaped(encoder)?;
        }

        Ok(())
    }

    fn render_section<'section, P, E>(
        &self,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        if let Ok(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Content> Content for Vec<T> {
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn render_section<'section, P, E>(
        &self,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Content> Content for [T] {
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn render_section<'section, P, E>(
        &self,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<K, V, S> Content for HashMap<K, V, S>
where
    K: Borrow<str> + Hash + Eq,
    V: Content,
    S: BuildHasher,
{
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn render_field_escaped<E>(&self, _: u64, name: &str, encoder: &mut E) -> Result<bool, E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_escaped(encoder).map(|_| true),
            None => Ok(false),
        }
    }

    fn render_field_unescaped<E>(
        &self,
        _: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_unescaped(encoder).map(|_| true),
            None => Ok(false),
        }
    }

    fn render_field_section<'section, P, E>(
        &self,
        _: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_section(section, encoder).map(|_| true),
            None => Ok(false),
        }
    }

    fn render_field_inverse<'section, P, E>(
        &self,
        _: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_inverse(section, encoder).map(|_| true),
            None => Ok(false),
        }
    }
}

impl<K, V> Content for BTreeMap<K, V>
where
    K: Borrow<str> + Ord,
    V: Content,
{
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn render_field_escaped<E>(&self, _: u64, name: &str, encoder: &mut E) -> Result<bool, E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_escaped(encoder).map(|_| true),
            None => Ok(false),
        }
    }

    fn render_field_unescaped<E>(
        &self,
        _: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_unescaped(encoder).map(|_| true),
            None => Ok(false),
        }
    }

    fn render_field_section<'section, P, E>(
        &self,
        _: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_section(section, encoder).map(|_| true),
            None => Ok(false),
        }
    }

    fn render_field_inverse<'section, P, E>(
        &self,
        _: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_inverse(section, encoder).map(|_| true),
            None => Ok(false),
        }
    }
}

macro_rules! impl_pointer_types {
    ($( $ty:ty $(: $bounds:tt)? ),*) => {
        $(
            impl<T: Content $(+ $bounds)?> Content for $ty {
                fn is_truthy(&self) -> bool {
                    self.deref().is_truthy()
                }

                fn capacity_hint(&self, tpl: &Template) -> usize {
                    self.deref().capacity_hint(tpl)
                }

                fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
                    self.deref().render_escaped(encoder)
                }

                fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
                    self.deref().render_unescaped(encoder)
                }

                fn render_section<'section, P, E>(
                    &self,
                    section: Section<'section, P>,
                    encoder: &mut E,
                ) -> Result<(), E::Error>
                where
                    P: Content + Copy + 'section,
                    E: Encoder,
                    Self: Sized,
                {
                    self.deref().render_section(section, encoder)
                }

                fn render_inverse<'section, P, E>(
                    &self,
                    section: Section<'section, P>,
                    encoder: &mut E,
                ) -> Result<(), E::Error>
                where
                    P: Content + Copy + 'section,
                    E: Encoder,
                {
                    self.deref().render_inverse(section, encoder)
                }

                fn render_field_escaped<E: Encoder>(
                    &self,
                    hash: u64,
                    name: &str,
                    encoder: &mut E,
                ) -> Result<bool, E::Error> {
                    self.deref().render_field_escaped(hash, name, encoder)
                }

                fn render_field_unescaped<E: Encoder>(
                    &self,
                    hash: u64,
                    name: &str,
                    encoder: &mut E,
                ) -> Result<bool, E::Error> {
                    self.deref().render_field_unescaped(hash, name, encoder)
                }

                fn render_field_section<'section, P, E>(
                    &self,
                    hash: u64,
                    name: &str,
                    section: Section<'section, P>,
                    encoder: &mut E,
                ) -> Result<bool, E::Error>
                where
                    P: Content + Copy + 'section,
                    E: Encoder,
                {
                    self.deref().render_field_section(hash, name, section, encoder)
                }

                fn render_field_inverse<'section, P, E>(
                    &self,
                    hash: u64,
                    name: &str,
                    section: Section<'section, P>,
                    encoder: &mut E,
                ) -> Result<bool, E::Error>
                where
                    P: Content + Copy + 'section,
                    E: Encoder,
                {
                    self.deref().render_field_inverse(hash, name, section, encoder)
                }
            }
        )*
    }
}

impl_pointer_types!(&T, Box<T>, std::rc::Rc<T>, std::sync::Arc<T>, Cow<'_, T>: ToOwned);
