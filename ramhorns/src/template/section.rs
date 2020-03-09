// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use super::{Block, Tag};
use crate::encoding::Encoder;
use crate::Content;
use crate::combine::Combine;

/// A section of a `Template` that can be rendered individually, usually delimited by
/// `{{#section}} ... {{/section}}` tags.
#[derive(Clone, Copy)]
pub struct Section<'section, P: Content + Copy + 'section> {
    blocks: &'section [Block<'section>],
    parents: P,
}

struct Parents<'content, C: Content + ?Sized, P: Content> {
    current: &'content C,
    parent: &'content P,
}

impl<'section> Section<'section, ()> {
    pub(crate) fn new(blocks: &'section [Block<'section>]) -> Self {
        Self {
            blocks,
            parents: (),
        }
    }
}

trait Renderable: Sized {
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
        _section: Section<'section, P>,
        _encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        P: Content + Copy + 'section,
        E: Encoder,
    {
        Ok(false)
    }
}

impl Renderable for () {}

impl<A: Content> Renderable for &A {
    fn render_field_escaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        (*self).render_field_escaped(hash, name, encoder)
    }

    fn render_field_unescaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        (*self).render_field_unescaped(hash, name, encoder)
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
        (*self).render_field_section(hash, name, section, encoder)
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
        (*self).render_field_inverse(hash, name, section, encoder)
    }
}

impl<A: Content, B: Content> Renderable for (&A, &B) {
    fn render_field_escaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        match self.1.render_field_escaped(hash, name, encoder) {
            Ok(false) => self.0.render_field_escaped(hash, name, encoder),
            res => res,
        }
    }

    fn render_field_unescaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        match self.1.render_field_unescaped(hash, name, encoder) {
            Ok(false) => self.0.render_field_unescaped(hash, name, encoder),
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
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.1.render_field_section(hash, name, section, encoder) {
            Ok(false) => self.0.render_field_section(hash, name, section, encoder),
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
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.1.render_field_inverse(hash, name, section, encoder) {
            Ok(false) => self.0.render_field_inverse(hash, name, section, encoder),
            res => res,
        }
    }
}

impl<A: Content, B: Content, C: Content> Renderable for (&A, &B, &C) {
    fn render_field_escaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        match self.2.render_field_escaped(hash, name, encoder) {
            Ok(false) => match self.1.render_field_escaped(hash, name, encoder) {
                Ok(false) => self.0.render_field_escaped(hash, name, encoder),
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
        match self.2.render_field_unescaped(hash, name, encoder) {
            Ok(false) => match self.1.render_field_unescaped(hash, name, encoder) {
                Ok(false) => self.0.render_field_unescaped(hash, name, encoder),
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
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.2.render_field_section(hash, name, section, encoder) {
            Ok(false) => match self.1.render_field_section(hash, name, section, encoder) {
                Ok(false) => self.0.render_field_section(hash, name, section, encoder),
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
        P: Content + Copy + 'section,
        E: Encoder,
    {
        match self.2.render_field_inverse(hash, name, section, encoder) {
            Ok(false) => match self.1.render_field_inverse(hash, name, section, encoder) {
                Ok(false) => self.0.render_field_inverse(hash, name, section, encoder),
                res => res,
            },
            res => res,
        }
    }
}

impl<A: Content, B: Content, C: Content, D: Content> Renderable for (&A, &B, &C, &D) {
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
        P: Content + Copy + 'section,
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
        P: Content + Copy + 'section,
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
}

impl<'section, P: Content + Copy + 'section> Section<'section, P> {
    fn with_parents(blocks: &'section [Block<'section>], parents: P) -> Self {
        Self { blocks, parents }
    }

    /// Render this section once to the provided `Encoder`. Some `Content`s will call
    /// this method multiple times (to render a list of elements).
    pub fn render_once<C: Content, E: Encoder>(
        &self,
        content: &C,
        encoder: &mut E,
    ) -> Result<(), E::Error> {
        let mut index = 0;
        let contents = Parents {
            current: content,
            parent: &self.parents,
        };

        let test = ().combine(content);

        while let Some(block) = self.blocks.get(index) {
            index += 1;

            encoder.write_unescaped(block.html)?;

            match block.tag {
                Tag::Escaped => {
                    contents.render_field_escaped(block.hash, block.name, encoder)?;
                }
                Tag::Unescaped => {
                    contents.render_field_unescaped(block.hash, block.name, encoder)?;
                }
                Tag::Section(count) => {
                    contents.render_field_section(
                        block.hash,
                        block.name,
                        Section::new(&self.blocks[index..index + count]),
                        encoder,
                    )?;
                    index += count;
                }
                Tag::Inverse(count) => {
                    contents.render_field_inverse(
                        block.hash,
                        block.name,
                        Section::new(&self.blocks[index..index + count]),
                        encoder,
                    )?;
                    index += count;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl<'content, G: Content + ?Sized, H: Content> Content for Parents<'content, G, H> {
    fn render_field_escaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        Ok(self.current.render_field_escaped(hash, name, encoder)?
            || self.parent.render_field_escaped(hash, name, encoder)?)
    }

    fn render_field_unescaped<E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        Ok(self.current.render_field_unescaped(hash, name, encoder)?
            || self.parent.render_field_unescaped(hash, name, encoder)?)
    }

    fn render_field_section<'section, P: Content + Copy + 'section, E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        Ok(self.current.render_field_section(
            hash,
            name,
            Section::with_parents(section.blocks, self),
            encoder,
        )? || self
            .parent
            .render_field_section(hash, name, section, encoder)?)
    }

    fn render_field_inverse<'section, P: Content + Copy + 'section, E: Encoder>(
        &self,
        hash: u64,
        name: &str,
        section: Section<'section, P>,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        Ok(self.current.render_field_inverse(
            hash,
            name,
            Section::with_parents(section.blocks, self),
            encoder,
        )? || self
            .parent
            .render_field_inverse(hash, name, section, encoder)?)
    }
}

// These traits need to be implemented manually since C or D may not be Clone
impl<'content, C: Content + ?Sized, D: Content> Clone for Parents<'content, C, D> {
    fn clone(&self) -> Self {
        Parents {
            current: self.current,
            parent: self.parent,
        }
    }
}

impl<'content, C: Content + ?Sized, D: Content> Copy for Parents<'content, C, D> {}
