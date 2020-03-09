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
use crate::traits::{Renderable};

/// A section of a `Template` that can be rendered individually, usually delimited by
/// `{{#section}} ... {{/section}}` tags.
#[derive(Clone, Copy)]
pub struct Section<'section, P: Renderable> {
    blocks: &'section [Block<'section>],
    parents: P,
}

// struct Parents<'content, C: Content + ?Sized, P: Content> {
//     current: &'content C,
//     parent: &'content P,
// }

impl<'section> Section<'section, ()> {
    pub(crate) fn new(blocks: &'section [Block<'section>]) -> Self {
        Self {
            blocks,
            parents: (),
        }
    }
}

// impl<'section, P: Renderable> Section<'section, P> {
//     pub fn content<C>(self, content: C) -> Section<'section, (P::I, P::J, P::K, C)>
//     where
//         C: Content + Copy,
//     {
//         Section {
//             blocks: self.blocks,
//             parents: self.parents.combine(content),
//         }
//     }
// }

impl<'section, P> Section<'section, P>
where
    P: Renderable,
{
    fn with_parents(blocks: &'section [Block<'section>], parents: P) -> Self {
        Self { blocks, parents }
    }

    /// Render this section once to the provided `Encoder`. Some `Content`s will call
    /// this method multiple times (to render a list of elements).
    pub fn render_once<'c, C, E>(&self, content: &'c C, encoder: &mut E) -> Result<(), E::Error>
    where
        C: Content + 'section,
        E: Encoder,
        // P: Combine<&'c C>,
    {
        let contents = self.parents.combine(content);

        let mut index = 0;
        // let contents = Parents {
        //     current: content,
        //     parent: &self.parents,
        // };

        // let contents = self.parents.child(content);

        // let test = ().combine(content);

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
                        Section::with_parents(&self.blocks[index..index + count], contents),
                        encoder,
                    )?;
                    index += count;
                }
                Tag::Inverse(count) => {
                    contents.render_field_inverse(
                        block.hash,
                        block.name,
                        Section::with_parents(&self.blocks[index..index + count], contents),
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

// impl<'content, G: Content + ?Sized, H: Content> Content for Parents<'content, G, H> {
//     fn render_field_escaped<E: Encoder>(
//         &self,
//         hash: u64,
//         name: &str,
//         encoder: &mut E,
//     ) -> Result<bool, E::Error> {
//         Ok(self.current.render_field_escaped(hash, name, encoder)?
//             || self.parent.render_field_escaped(hash, name, encoder)?)
//     }

//     fn render_field_unescaped<E: Encoder>(
//         &self,
//         hash: u64,
//         name: &str,
//         encoder: &mut E,
//     ) -> Result<bool, E::Error> {
//         Ok(self.current.render_field_unescaped(hash, name, encoder)?
//             || self.parent.render_field_unescaped(hash, name, encoder)?)
//     }

//     fn render_field_section<'section, P: Content + Copy + 'section, E: Encoder>(
//         &self,
//         hash: u64,
//         name: &str,
//         section: Section<'section, P>,
//         encoder: &mut E,
//     ) -> Result<bool, E::Error> {
//         Ok(self.current.render_field_section(
//             hash,
//             name,
//             Section::with_parents(section.blocks, self),
//             encoder,
//         )? || self
//             .parent
//             .render_field_section(hash, name, section, encoder)?)
//     }

//     fn render_field_inverse<'section, P: Content + Copy + 'section, E: Encoder>(
//         &self,
//         hash: u64,
//         name: &str,
//         section: Section<'section, P>,
//         encoder: &mut E,
//     ) -> Result<bool, E::Error> {
//         Ok(self.current.render_field_inverse(
//             hash,
//             name,
//             Section::with_parents(section.blocks, self),
//             encoder,
//         )? || self
//             .parent
//             .render_field_inverse(hash, name, section, encoder)?)
//     }
// }

// // These traits need to be implemented manually since C or D may not be Clone
// impl<'content, C: Content + ?Sized, D: Content> Clone for Parents<'content, C, D> {
//     fn clone(&self) -> Self {
//         Parents {
//             current: self.current,
//             parent: self.parent,
//         }
//     }
// }

// impl<'content, C: Content + ?Sized, D: Content> Copy for Parents<'content, C, D> {}
