// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use crate::Partials;
use super::{Block, Error, Tag, Template};

const OPEN: [u8; 2] = *b"{{";
const CLOSE: [u8; 2] = *b"}}";

impl<'tpl> Template<'tpl> {
    pub(crate) fn parse(
        &mut self,
        source: &'tpl str,
        iter: &mut impl Iterator<Item = (usize, [u8; 2])>,
        last: &mut usize,
        partials: &mut impl Partials<'tpl>,
    ) -> Result<(), Error> {
        while let Some((start, bytes)) = iter.next() {
            if bytes == OPEN {
                // Skip a byte since we got a double
                iter.next();

                let mut tag = Tag::Escaped;
                let mut start_skip = 2;
                let mut end_skip = 2;

                while let Some((_, bytes)) = iter.next() {
                    match bytes[0] {
                        b'{' => {
                            tag = Tag::Unescaped;
                            end_skip = 3;
                        }
                        b'#' => tag = Tag::Section,
                        b'^' => tag = Tag::Inverse,
                        b'/' => tag = Tag::Closing,
                        b'!' => tag = Tag::Comment,
                        b'&' => tag = Tag::Unescaped,
                        b'>' => tag = Tag::Partial,
                        b' ' | b'\t' | b'\r' | b'\n' => {
                            start_skip += 1;
                            continue;
                        }
                        _ => break,
                    }

                    start_skip += 1;

                    break;
                }

                let html = &source[*last..start];
                self.capacity_hint += html.len();

                loop {
                    if let (end, CLOSE) = iter.next().ok_or_else(|| Error::UnclosedTag)? {
                        // Skip the braces
                        if end_skip == 3 {
                            match iter.next() {
                                Some((_, CLOSE)) => {}
                                _ => return Err(Error::UnclosedTag),
                            }
                        }

                        iter.next();

                        let name = source[start + start_skip..end].trim();

                        *last = end + end_skip;

                        let insert_index = self.blocks.len();

                        self.blocks.push(Block::new(html, name, tag));

                        match tag {
                            Tag::Section | Tag::Inverse => {
                                self.parse(source, iter, last, partials)?;

                                let children = (self.blocks.len() - 1 - insert_index) as u32;

                                let this = &mut self.blocks[insert_index];
                                let hash = this.hash;
                                this.children = children;

                                if let Some(last) = self.blocks.last() {
                                    if last.hash != hash || last.tag != Tag::Closing {
                                        return Err(Error::UnclosedSection(name.into()));
                                    }
                                }
                            }
                            Tag::Closing => {
                                return Ok(());
                            }
                            Tag::Partial => {
                                let partial = partials.get_partial(name)?;
                                self.blocks.extend_from_slice(&partial.blocks);
                                self.capacity_hint += partial.capacity_hint;
                            }
                            _ => {}
                        };

                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
