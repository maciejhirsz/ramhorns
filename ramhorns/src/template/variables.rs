use super::{Block, Tag};

use std::fmt;

/// An iterator over variables, created by calling the `variables` method
/// on a `Template`.
#[derive(Clone, PartialEq, Eq)]
pub struct Variables<'var> {
    pub(crate) blocks: &'var [Block<'var>],
}

impl<'var> Iterator for Variables<'var> {
    type Item = (&'var str, Option<Variables<'var>>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let block = self.blocks.get(0)?;

            self.blocks = &self.blocks[1..];

            let nested = match block.tag {
                Tag::Closing | Tag::Comment => continue,
                Tag::Unescaped | Tag::Escaped => None,
                Tag::Section(len) | Tag::Inverse(len) => {
                    let blocks = &self.blocks[..len];

                    self.blocks = &self.blocks[len..];

                    Some(Variables {
                        blocks
                    })
                }
            };

            return Some((block.name, nested));
        }
    }
}

impl<'var> fmt::Debug for Variables<'var> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
