// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{Block, Tag};
use crate::encoding::Encoder;
#[cfg(feature = "indexes")]
use crate::template::Indexed;
use crate::traits::{Combine, ContentSequence};
use crate::Content;
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

    /// The section without the last `Content` in the stack
    #[inline]
    pub fn without_last(self) -> Section<'section, C::Previous> {
        Section {
            blocks: self.blocks,
            contents: self.contents.crawl_back(),
        }
    }

    /// The section without the first `Block` in the stack.
    #[inline]
    pub fn without_first(self) -> Self {
        Section {
            blocks: &self.blocks[1..],
            contents: self.contents,
        }
    }

    /// The section index of the next block, if it's a section index tag.
    #[cfg(feature = "indexes")]
    #[inline]
    pub fn section_index(&self) -> Option<&Indexed> {
        self.blocks.first().and_then(|b| b.index())
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

            match &block.tag {
                Tag::Escaped => {
                    self.contents
                        .render_field_escaped(block.hash, block.name, encoder)?;
                }
                Tag::Unescaped => {
                    self.contents
                        .render_field_unescaped(block.hash, block.name, encoder)?;
                }
                Tag::Section => {
                    self.contents.render_field_section(
                        block.hash,
                        block.name,
                        self.slice(index..index + block.children as usize),
                        encoder,
                    )?;
                    index += block.children as usize;
                }
                Tag::Inverse => {
                    self.contents.render_field_inverse(
                        block.hash,
                        block.name,
                        self.slice(index..index + block.children as usize),
                        encoder,
                    )?;
                    index += block.children as usize;
                }
                #[cfg(feature = "indexes")]
                Tag::Indexed(indexed) => {
                    self.contents.render_index_section(
                        indexed,
                        self.slice(index..index + block.children as usize),
                        encoder,
                    )?;
                    index += block.children as usize;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
