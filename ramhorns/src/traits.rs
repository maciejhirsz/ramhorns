//! This module contains helper traits that are used internally to manage
//! sequences of types implementing the `Content` trait, allowing us to
//! statically manage parent section lookups without doing extra work on
//! runtime.

use crate::template::Section;
use crate::encoding::Encoder;
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

    /// Combines current tuple with a new element.
    fn combine<X: Content + ?Sized>(self, other: &X) -> (Self::I, Self::J, Self::K, &X);
}

/// Helper trait that re-exposes `render_field_x` methods of a `Content` trait,
/// calling those methods internally on all `Content`s contained within `Self`.
pub trait ContentSequence: Combine + Sized + Copy {
    /// Render a field by the hash **or** string of its name.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    /// If successful, returns `true` if the field exists in this content, otherwise `false`.
    #[inline]
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
    #[inline]
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
    #[inline]
    fn render_field_section<'section, P, E>(
        &self,
        _hash: u64,
        _name: &str,
        _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        Ok(false)
    }

    /// Render a field, by the hash of **or** string its name, as an inverse section.
    /// If successful, returns `true` if the field exists in this content, otherwise `false`.
    #[inline]
    fn render_field_inverse<'section, P, E>(
        &self,
        _hash: u64,
        _name: &str,
        _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        Ok(false)
    }
}

impl Combine for () {
    type I = ();
    type J = ();
    type K = ();

    #[inline]
    fn combine<X: Content + ?Sized>(self, other: &X) -> ((), (), (), &X) {
        ((), (), (), other)
    }
}

impl ContentSequence for () {}

impl<'tup, A, B, C, D> Combine for (A, B, C, &'tup D)
where
    A: Content + Copy + Sized,
    B: Content + Copy + Sized,
    C: Content + Copy + Sized,
    D: Content + ?Sized,
{
    type I = B;
    type J = C;
    type K = &'tup D;

    #[inline]
    fn combine<X: Content + ?Sized>(self, other: &X) -> (B, C, &'tup D, &X) {
        (self.1, self.2, self.3, other)
    }
}

impl<A, B, C, D> ContentSequence for (A, B, C, &D)
where
    A: Content + Copy + Sized,
    B: Content + Copy + Sized,
    C: Content + Copy + Sized,
    D: Content + ?Sized,
{
    #[inline]
    fn render_field_escaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        match self.3.render_field_escaped(hash, name, encoder) {
            Ok(false) => match self.2.render_field_escaped(hash, name, encoder) {
                Ok(false) => match self.1.render_field_escaped(hash, name, encoder) {
                    Ok(false) => self.0.render_field_escaped(hash, name, encoder),
                    res => res,
                },
                res => res,
            },
            res => res,
        }
    }

    #[inline]
    fn render_field_unescaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        match self.3.render_field_unescaped(hash, name, encoder) {
            Ok(false) => match self.2.render_field_unescaped(hash, name, encoder) {
                Ok(false) => match self.1.render_field_unescaped(hash, name, encoder) {
                    Ok(false) => self.0.render_field_unescaped(hash, name, encoder),
                    res => res,
                },
                res => res,
            }
            res => res,
        }
    }

    #[inline]
    fn render_field_section<P, E>(
        &self,
        hash: u64,
        name: &str,
        section: Section<P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        match self.3.render_field_section(hash, name, section, encoder) {
            Ok(false) => match self.2.render_field_section(hash, name, section, encoder) {
                Ok(false) => match self.1.render_field_section(hash, name, section, encoder) {
                    Ok(false) => self.0.render_field_section(hash, name, section, encoder),
                    res => res,
                },
                res => res,
            },
            res => res,
        }
    }

    #[inline]
    fn render_field_inverse<P, E>(
        &self,
        hash: u64,
        name: &str,
        section: Section<P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: ContentSequence,
        E: Encoder,
    {
        match self.3.render_field_inverse(hash, name, section, encoder) {
            Ok(false) => match self.2.render_field_inverse(hash, name, section, encoder) {
                Ok(false) => match self.1.render_field_inverse(hash, name, section, encoder) {
                    Ok(false) => match self.0.render_field_inverse(hash, name, section, encoder) {
                        Ok(false) => section.render(encoder).map(|()| true),
                        res => res,
                    },
                    res => res,
                },
                res => res,
            },
            res => res,
        }
    }
}