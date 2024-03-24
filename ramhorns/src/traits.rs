// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This module contains helper traits that are used internally to manage
//! sequences of types implementing the `Content` trait, allowing us to
//! statically manage parent section lookups without doing extra work on
//! runtime.

use crate::encoding::Encoder;
#[cfg(not(feature = "indexes"))]
use crate::template::Section;
#[cfg(feature = "indexes")]
use crate::template::{Indexed, Section};
use crate::Content;

/// Helper trait used to rotate a queue of parent `Content`s. Think of this as of a
/// rotating buffer such that:
///
/// ```text
/// (A, B, C, D).combine(X) -> (B, C, D, X)
/// ```
///
/// This allows us to keep track of up to 3 parent contexts. The constraint is implemented
/// so that self-referencing `Content`s don't blow up the stack on compilation.
pub trait Combine {
    /// First type for the result tuple
    type I: Content + Copy + Sized;
    /// Second type for the result tuple
    type J: Content + Copy + Sized;
    /// Third type for the result tuple
    type K: Content + Copy + Sized;

    /// Type when we crawl back one item
    type Previous: ContentSequence;

    /// Combines current tuple with a new element.
    fn combine<X: Content + ?Sized>(self, other: &X) -> (Self::I, Self::J, Self::K, &X);

    /// Crawl back to the previous tuple
    fn crawl_back(self) -> Self::Previous;
}

/// Helper trait that re-exposes `render_field_x` methods of a `Content` trait,
/// calling those methods internally on all `Content`s contained within `Self`.
pub trait ContentSequence: Combine + Sized + Copy {
    /// Render a field by the hash **or** string of its name.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    #[inline]
    fn render_field_escaped<E: Encoder>(
        &self,
        _hash: u64,
        _name: &str,
        _encoder: &mut E,
    ) -> Result<(), E::Error> {
        Ok(())
    }

    /// Render a field by the hash **or** string of its name.
    ///
    /// This doesn't perform any escaping at all.
    #[inline]
    fn render_field_unescaped<E: Encoder>(
        &self,
        _hash: u64,
        _name: &str,
        _encoder: &mut E,
    ) -> Result<(), E::Error> {
        Ok(())
    }

    /// Render a field by the hash **or** string of its name, as a section.
    #[inline]
    fn render_field_section<P, E>(
        &self,
        _hash: u64,
        _name: &str,
        _section: Section<'_, P>,
        _encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field, by the hash of **or** string its name, as an inverse section.
    #[inline]
    fn render_field_inverse<P, E>(
        &self,
        _hash: u64,
        _name: &str,
        _section: Section<'_, P>,
        _encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        Ok(())
    }

    /// Render an index based section.
    #[cfg(feature = "indexes")]
    #[inline]
    fn render_index_section<'section, P, E>(
        &self,
        _indexed: &Indexed,
        _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        Ok(())
    }
}

impl Combine for () {
    type I = ();
    type J = ();
    type K = ();
    type Previous = ();

    #[inline]
    fn combine<X: Content + ?Sized>(self, other: &X) -> ((), (), (), &X) {
        ((), (), (), other)
    }

    #[inline]
    fn crawl_back(self) -> Self::Previous {}
}

impl ContentSequence for () {}

impl<A, B, C, D> Combine for (A, B, C, D)
where
    A: Content + Copy,
    B: Content + Copy,
    C: Content + Copy,
    D: Content + Copy,
{
    type I = B;
    type J = C;
    type K = D;
    type Previous = ((), A, B, C);

    #[inline]
    fn combine<X: Content + ?Sized>(self, other: &X) -> (B, C, D, &X) {
        (self.1, self.2, self.3, other)
    }

    #[inline]
    fn crawl_back(self) -> ((), A, B, C) {
        ((), self.0, self.1, self.2)
    }
}

impl<A, B, C, D> ContentSequence for (A, B, C, D)
where
    A: Content + Copy,
    B: Content + Copy,
    C: Content + Copy,
    D: Content + Copy,
{
    #[inline]
    fn render_field_escaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<(), E::Error> {
        if !self.3.render_field_escaped(hash, name, encoder)?
            && !self.2.render_field_escaped(hash, name, encoder)?
            && !self.1.render_field_escaped(hash, name, encoder)?
        {
            self.0.render_field_escaped(hash, name, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn render_field_unescaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<(), E::Error> {
        if !self.3.render_field_unescaped(hash, name, encoder)?
            && !self.2.render_field_unescaped(hash, name, encoder)?
            && !self.1.render_field_unescaped(hash, name, encoder)?
        {
            self.0.render_field_unescaped(hash, name, encoder)?;
        }
        Ok(())
    }

    #[inline]
    fn render_field_section<P, E>(
        &self,
        hash: u64,
        name: &str,
        section: Section<P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        if !self.3.render_field_section(hash, name, section, encoder)? {
            let section = section.without_last();
            if !self.2.render_field_section(hash, name, section, encoder)? {
                let section = section.without_last();
                if !self.1.render_field_section(hash, name, section, encoder)? {
                    let section = section.without_last();
                    self.0.render_field_section(hash, name, section, encoder)?;
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn render_field_inverse<P, E>(
        &self,
        hash: u64,
        name: &str,
        section: Section<P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        if !self.3.render_field_inverse(hash, name, section, encoder)?
            && !self.2.render_field_inverse(hash, name, section, encoder)?
            && !self.1.render_field_inverse(hash, name, section, encoder)?
            && !self.0.render_field_inverse(hash, name, section, encoder)?
        {
            section.render(encoder)?;
        }
        Ok(())
    }

    #[cfg(feature = "indexes")]
    #[inline]
    fn render_index_section<'section, P, E>(
        &self,
        indexed: &Indexed,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<(), E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        if !self.3.render_index_section(indexed, section, encoder)? {
            let section = section.without_last();
            if !self.2.render_index_section(indexed, section, encoder)? {
                let section = section.without_last();
                if !self.1.render_index_section(indexed, section, encoder)? {
                    let section = section.without_last();
                    self.0.render_index_section(indexed, section, encoder)?;
                }
            }
        }

        Ok(())
    }
}
