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
use crate::traits::{Combine, ContentSequence};
use std::ops::Range;

/// A section of a `Template` that can be rendered individually, usually delimited by
/// `{{#section}} ... {{/section}}` tags.
#[derive(Clone, Copy)]
pub struct Section<'section, Contents: ContentSequence> {
    blocks: &'section [Block<'section>],
    contents: Contents,
}

/// Necessary so that the warning of very complex type created when compiling
/// with `cargo clippy` doesn't propagate to downstream crates
type Next<C, X> = (<C as Combine>::I, <C as Combine>::J, <C as Combine>::K, X);

impl<'section> Section<'section, ()> {
    #[inline]
    pub(crate) fn new(blocks: &'section [Block<'section>]) -> Self {
        Self {
            blocks,
            contents: (),
        }
    }
}

impl<'section, C> Section<'section, C>
where
    C: ContentSequence,
{
    #[inline]
    fn slice(self, range: Range<usize>) -> Self {
        Self {
            blocks: &self.blocks[range],
            contents: self.contents,
        }
    }

    /// Attach a `Content` to this section. This will keep track of a stack up to
    /// 4 `Content`s deep, cycling on overflow.
    #[inline]
    pub fn with<X>(self, content: &X) -> Section<'section, Next<C, &X>>
    where
        X: Content + ?Sized,
    {
        Section {
            blocks: self.blocks,
            contents: self.contents.combine(content),
        }
    }

    /// Render this section once to the provided `Encoder`.
    pub fn render<E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        let mut index = 0;

        while let Some(block) = self.blocks.get(index) {
            index += 1;

            encoder.write_unescaped(block.html)?;

            match block.tag {
                Tag::Escaped => {
                    self.contents.render_field_escaped(block.hash, block.name, encoder)?;
                }
                Tag::Unescaped => {
                    self.contents.render_field_unescaped(block.hash, block.name, encoder)?;
                }
                Tag::Section(count) => {
                    self.contents.render_field_section(
                        block.hash,
                        block.name,
                        self.slice(index..index + count as usize),
                        encoder,
                    )?;
                    index += count as usize;
                }
                Tag::Inverse(count) => {
                    self.contents.render_field_inverse(
                        block.hash,
                        block.name,
                        self.slice(index..index + count as usize),
                        encoder,
                    )?;
                    index += count as usize;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
