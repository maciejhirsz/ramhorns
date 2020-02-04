// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use super::{Block, Tag};
use crate::content::EncContent;
use crate::encoding::Encoder;
use crate::Content;

/// A section of a `Template` that can be rendered individually, usually delimited by
/// `{{#section}} ... {{/section}}` tags.
pub struct Section<'section, 'content: 'section, E: Encoder> {
    blocks: &'section [Block<'section>],
    stack: &'section mut Vec<&'content dyn EncContent<E>>,
}

impl<'section, 'content: 'section, E: Encoder> Section<'section, 'content, E> {
    pub(crate) fn new(
        blocks: &'section [Block<'section>],
        stack: &'section mut Vec<&'content dyn EncContent<E>>,
    ) -> Self {
        Self { blocks, stack }
    }

    /// Render this section once to the provided `Encoder`. Some `Content`s will call
    /// this method multiple times (to render a list of elements).
    pub fn render_once<C: Content>(
        &mut self,
        content: &'content C,
        encoder: &mut E,
    ) -> Result<(), E::Error> {
        let mut index = 0;

        while let Some(block) = self.blocks.get(index) {
            index += 1;

            encoder.write_unescaped(block.html)?;

            match block.tag {
                Tag::Escaped => {
                    if !content.render_field_escaped(block.hash, block.name, encoder)? {
                        for parent in self.stack.iter().rev() {
                            if parent.render_field_escaped(block.hash, block.name, encoder)? {
                                break;
                            }
                        }
                    }
                }
                Tag::Unescaped => {
                    if !content.render_field_unescaped(block.hash, block.name, encoder)? {
                        for parent in self.stack.iter().rev() {
                            if parent.render_field_unescaped(block.hash, block.name, encoder)? {
                                break;
                            }
                        }
                    }
                }
                Tag::Section(count) => {
                    self.stack.push(content);
                    if !content.render_field_section(
                        block.hash,
                        block.name,
                        Section::new(&self.blocks[index..index + count], self.stack),
                        encoder,
                    )? && self.stack.len() > 1
                    {
                        // Can't use an iterator since it references stack, which is mutable
                        for i in (0..self.stack.len() - 1).rev() {
                            if self.stack[i].render_field_section(
                                block.hash,
                                block.name,
                                Section::new(&self.blocks[index..index + count], self.stack),
                                encoder,
                            )? {
                                break;
                            };
                        }
                    };
                    self.stack.pop();

                    index += count;
                }
                Tag::Inverse(count) => {
                    self.stack.push(content);
                    if !content.render_field_inverse(
                        block.hash,
                        block.name,
                        Section::new(&self.blocks[index..index + count], self.stack),
                        encoder,
                    )? && self.stack.len() > 1
                    {
                        // Can't use an iterator since it references stack, which is mutable
                        for i in (0..self.stack.len() - 1).rev() {
                            if self.stack[i].render_field_inverse(
                                block.hash,
                                block.name,
                                Section::new(&self.blocks[index..index + count], self.stack),
                                encoder,
                            )? {
                                break;
                            };
                        }
                    };
                    self.stack.pop();

                    index += count;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
