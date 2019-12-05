// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use super::{Block, Error, Tag, Template, Partials};
use std::path::Path;

impl<'tpl> Template<'tpl> {
    pub(crate) fn parse<Iter>(
        &mut self,
        source: &'tpl str,
        iter: &mut Iter,
        last: &mut usize,
        until: Option<&'tpl str>,
        dir: &Path,
        partials: &mut Partials<'tpl>,
    ) -> Result<usize, Error>
    where
        Iter: Iterator<Item = (usize, &'tpl [u8; 2])>,
    {
        let blocks_at_start = self.blocks.len();

        while let Some((start, bytes)) = iter.next() {
            if bytes == b"{{" {
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
                        b'#' => tag = Tag::Section(0),
                        b'^' => tag = Tag::Inverse(0),
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

                loop {
                    if let (end, b"}}") = iter.next().ok_or_else(|| Error::UnclosedTag)? {
                        // Skip the braces
                        if end_skip == 3 {
                            match iter.next() {
                                Some((_, b"}}")) => {}
                                _ => return Err(Error::UnclosedTag),
                            }
                        }

                        iter.next();

                        let name = source[start + start_skip..end].trim();

                        *last = end + end_skip;

                        let insert_index = self.blocks.len();

                        self.capacity_hint += html.len();
                        self.blocks
                            .insert(insert_index, Block::new(html, name, tag));

                        match tag {
                            Tag::Section(_) | Tag::Inverse(_) => {
                                let count =
                                    self.parse(source, iter, last, Some(name), dir, partials)?;

                                match self.blocks[insert_index].tag {
                                    Tag::Section(ref mut c) | Tag::Inverse(ref mut c) => *c = count,
                                    _ => {}
                                }
                            }
                            Tag::Closing => {
                                if let Some(until) = until {
                                    if until != name {
                                        return Err(Error::UnclosedSection(until.into()));
                                    }
                                }

                                return Ok(self.blocks.len() - blocks_at_start);
                            }
                            Tag::Partial => {
                                let path = dir.join(name);
                                if !partials.contains_key(name) {
                                    let template = Template::load(
                                        std::fs::read_to_string(&path)?,
                                        dir,
                                        partials,
                                    )?;
                                    partials.insert(name.into(), template);
                                };
                                let partial = &partials[name];
                                self.blocks.extend(&partial.blocks);
                                self.blocks.push(Block::new(partial.tail, name, tag));
                                self.capacity_hint += partial.capacity_hint + partial.tail.len();
                            }
                            _ => {}
                        };

                        break;
                    }
                }
            }
        }

        if let Some(until) = until {
            return Err(Error::UnclosedSection(until.into()));
        }

        Ok(self.blocks.len() - blocks_at_start)
    }
}
