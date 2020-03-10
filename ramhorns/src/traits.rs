use crate::template::Section;
use crate::encoding::Encoder;
use crate::Content;

/// Another helper trait that wraps lists of Contents
pub trait Renderable: Sized + Copy {
    type I: Content + Copy;
    type J: Content + Copy;
    type K: Content + Copy;

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
        _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Renderable,
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
        _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Renderable,
        E: Encoder,
    {
        Ok(false)
    }

    fn combine<X: Content + Copy>(self, other: X) -> (Self::I, Self::J, Self::K, X);
}

impl Renderable for () {
    type I = ();
    type J = ();
    type K = ();

    fn combine<X: Content + Copy>(self, other: X) -> ((), (), (), X) {
        ((), (), (), other)
    }
}

impl<A, B, C, D> Renderable for (A, B, C, D)
where
    A: Content + Copy,
    B: Content + Copy,
    C: Content + Copy,
    D: Content + Copy,
{
    type I = B;
    type J = C;
    type K = D;

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

    fn render_field_section<'section, P, E>(
        &self,
        hash: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Renderable,
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

    fn render_field_inverse<'section, P, E>(
        &self,
        hash: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Renderable,
        E: Encoder,
    {
        match self.3.render_field_inverse(hash, name, section, encoder) {
            Ok(false) => match self.2.render_field_inverse(hash, name, section, encoder) {
                Ok(false) => match self.1.render_field_inverse(hash, name, section, encoder) {
                    Ok(false) => self.0.render_field_inverse(hash, name, section, encoder),
                    res => res,
                },
                res => res,
            },
            res => res,
        }
    }

    fn combine<X: Content + Copy>(self, other: X) -> (B, C, D, X) {
        (self.1, self.2, self.3, other)
    }
}