use super::{Block, Tag};
use crate::{Context, Encoder};

use std::io::{self, Write};

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

    pub fn render_once<C, W>(&self, ctx: &C, encoder: &mut Encoder<W>) -> io::Result<()>
    where
        C: Context,
        W: Write,
    {
        let mut index = 0;

        while let Some(block) = self.blocks.get(index) {
            index += 1;

            encoder.write(block.html)?;

            match block.tag {
                Tag::Escaped => ctx.render_field_escaped(block.hash, encoder)?,
                Tag::Unescaped => ctx.render_field_unescaped(block.hash, encoder)?,
                Tag::Section(count) => {
                    ctx.render_field_section(block.hash, Section::new(&self.blocks[index..index + count]), encoder)?;

                    index += count;
                },
                Tag::Inverse(count) => {
                    ctx.render_field_inverse(block.hash, Section::new(&self.blocks[index..index + count]), encoder)?;

                    index += count;
                },
                Tag::Closing |
                Tag::Comment => {},
            }
        }

        Ok(())
    }
}

// pub trait SectionContext {
//     fn render_section<'section, W: Write>(&self, _section: Section<'section>, _encoder: &mut Encoder<W>) -> io::Result<()> {
//         Ok(())
//     }

//     fn render_inverse<'section, W: Write>(&self, _section: Section<'section>, _encoder: &mut Encoder<W>) -> io::Result<()> {
//         Ok(())
//     }
// }

// impl<C: Context> SectionContext for C {
//     fn render_section<'section, W: Write>(&self, section: Section<'section>, encoder: &mut Encoder<W>) -> io::Result<()> {
//         section.render_once(self, encoder)
//     }
// }
