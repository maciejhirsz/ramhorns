// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use super::{Block, Tag};
use crate::Content;
use crate::encoding::Encoder;

/// A section of a `Template` that can be rendered individually, usually delimited by
/// `{{#section}} ... {{/section}}` tags.
#[derive(Debug, PartialEq, Eq)]
pub struct Section<'section> {
    blocks: &'section [Block<'section>],
}

impl<'section> Section<'section> {
    pub(crate) const fn new(blocks: &'section [Block<'section>]) -> Self {
        Self {
            blocks,
        }
    }

    /// Render this section once to the provided `Encoder`. Some `Content`s will call
    /// this method multiple times (to render a list of elements).
    pub fn render_once<C, E>(&self, content: &C, encoder: &mut E) -> Result<(), E::Error>
    where
        C: Content,
        E: Encoder,
    {
        let mut index = 0;

        while let Some(block) = self.blocks.get(index) {
            index += 1;

            encoder.write_unescaped(block.html)?;

            match block.tag {
                Tag::Escaped => content.render_field_escaped(block.hash, block.name, encoder)?,
                Tag::Unescaped => content.render_field_unescaped(block.hash, block.name, encoder)?,
                Tag::Section(count) => {
                    content.render_field_section(block.hash, block.name, Section::new(&self.blocks[index..index + count]), encoder)?;

                    index += count;
                },
                Tag::Inverse(count) => {
                    content.render_field_inverse(block.hash, block.name, Section::new(&self.blocks[index..index + count]), encoder)?;

                    index += count;
                },
                Tag::Closing |
                Tag::Comment => {},
            }
        }

        Ok(())
    }
}
