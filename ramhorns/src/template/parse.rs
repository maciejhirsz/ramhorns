// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use arrayvec::ArrayVec;
use logos::Logos;
#[cfg(feature = "indexes")]
use std::convert::TryFrom;

use super::{hash_name, Block, Error, Template};
use crate::Partials;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
#[logos(
    skip r"[^{]+",
    skip r"\{",
    extras = Braces,
    error = ParseError,
)]
pub enum Tag {
    /// `{{escaped}}` tag
    #[token("{{")]
    Escaped,

    /// `{{{unescaped}}}` tag
    #[token("{{&")]
    #[token("{{{", |lex| lex.extras = Braces::Three)]
    Unescaped,

    /// `{{#section}}` opening tag (with number of subsequent blocks it contains)
    #[token("{{#")]
    Section,

    /// `{{^inverse}}` section opening tag (with number of subsequent blocks it contains)
    #[token("{{^")]
    Inverse,

    /// `{{/closing}}` section tag
    #[token("{{/")]
    Closing,

    /// Index based list traversal
    #[cfg(feature = "indexes")]
    Indexed(Indexed),

    /// `{{!comment}}` tag
    #[token("{{!")]
    Comment,

    /// `{{>partial}}` tag
    #[token("{{>")]
    Partial,

    /// Tailing html
    Tail,
}

/// Inclusive/Exclusive index based tag.
#[cfg(feature = "indexes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Indexed {
    /// `{{#-index}}` opening tag for an exact index
    Include(Index),
    /// `{{^-index}}` opening tag for all other indexes
    Exclude(Index),
}
#[cfg(feature = "indexes")]
impl Indexed {
    /// Marks whether this index is truthy.
    /// Returns true when this index is to be rendered.
    #[inline]
    pub fn is_truthy(&self, len: usize, index: usize) -> bool {
        match self {
            Indexed::Include(index_) => index_.nth(len) == index,
            Indexed::Exclude(index_) => index_.nth(len) != index,
        }
    }
}

/// Index specification for an index based tag.
#[cfg(feature = "indexes")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Index {
    /// The first element of the list.
    First,
    /// The last element of the list.
    Last,
    /// The nth element of the list.
    Nth(usize),
}
#[cfg(feature = "indexes")]
impl Index {
    /// Get the index numeral value for a list of the given length.
    pub fn nth(&self, len: usize) -> usize {
        match self {
            Index::First => 0,
            Index::Last => len - 1,
            Index::Nth(n) => *n,
        }
    }
}
#[cfg(feature = "indexes")]
impl TryFrom<&str> for Index {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "first" => Self::First,
            "last" => Self::Last,
            maybe_number => Self::Nth(
                maybe_number
                    .parse()
                    .map_err(|_| Error::IndexParse(maybe_number.to_string()))?,
            ),
        })
    }
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Error {
        Error::UnclosedTag
    }
}

#[derive(Logos)]
#[logos(
    skip r"[ ]+",
    extras = Braces,
)]
enum Closing {
    #[token("}}", |lex| {
        // Force fail the match if we expected 3 braces
        lex.extras != Braces::Three
    })]
    #[token("}}}")]
    Match,

    #[regex(r"[^ \}]+")]
    Ident,
}

/// Marker of how many braces we expect to match
#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum Braces {
    #[default]
    Two = 2,
    Three = 3,
}

impl<'tpl> Template<'tpl> {
    pub(crate) fn parse(
        &mut self,
        source: &'tpl str,
        partials: &mut impl Partials<'tpl>,
    ) -> Result<usize, Error> {
        let mut last = 0;
        let mut lex = Tag::lexer(source);
        let mut stack = ArrayVec::<usize, 16>::new();

        while let Some(tag) = lex.next() {
            let tag = tag?;

            // Grab HTML from before the token
            // TODO: add lex.before() that yields source slice
            // in front of the token:
            //
            // let html = &lex.before()[last..];
            let mut html = &lex.source()[last..lex.span().start];
            self.capacity_hint += html.len();

            // Morphing the lexer to match the closing
            // braces and grab the name
            let mut closing = lex.morph();
            let tail_idx = self.blocks.len();

            let _tok = closing.next();
            if !matches!(Some(Closing::Ident), _tok) {
                return Err(Error::UnclosedTag);
            }
            let mut name = closing.slice();
            match tag {
                Tag::Escaped | Tag::Unescaped => {
                    loop {
                        match closing.next() {
                            Some(Ok(Closing::Ident)) => {
                                self.blocks.push(Block::new(html, name, Tag::Section));
                                name = closing.slice();
                                html = "";
                            }
                            Some(Ok(Closing::Match)) => {
                                self.blocks.push(Block::new(html, name, tag));
                                break;
                            }
                            _ => return Err(Error::UnclosedTag),
                        }
                    }

                    let d = self.blocks.len() - tail_idx - 1;
                    for i in 0..d {
                        self.blocks[tail_idx + i].children = (d - i) as u32;
                    }
                }
                Tag::Section | Tag::Inverse => loop {
                    match closing.next() {
                        Some(Ok(Closing::Ident)) => {
                            stack.try_push(self.blocks.len())?;
                            self.blocks.push(Block::new(html, name, Tag::Section));
                            name = closing.slice();
                            html = "";
                        }
                        Some(Ok(Closing::Match)) => {
                            stack.try_push(self.blocks.len())?;
                            #[cfg(feature = "indexes")]
                            let tag = match name.strip_prefix("-") {
                                Some(index) if tag == Tag::Section => {
                                    Tag::Indexed(Indexed::Include(Index::try_from(index)?))
                                }
                                Some(index) => {
                                    Tag::Indexed(Indexed::Exclude(Index::try_from(index)?))
                                }
                                None => tag,
                            };
                            self.blocks.push(Block::new(html, name, tag));
                            break;
                        }
                        _ => return Err(Error::UnclosedTag),
                    }
                },
                Tag::Closing => {
                    #[cfg(feature = "indexes")]
                    self.blocks.push(Block::nameless(html, tag));
                    #[cfg(not(feature = "indexes"))]
                    self.blocks.push(Block::nameless(html, Tag::Closing));

                    let mut pop_section = |name| {
                        let hash = hash_name(name);

                        let head_idx = stack
                            .pop()
                            .ok_or_else(|| Error::UnopenedSection(name.into()))?;
                        let head = &mut self.blocks[head_idx];
                        head.children = (tail_idx - head_idx) as u32;

                        if head.hash != hash {
                            return Err(Error::UnclosedSection(head.name.into()));
                        }
                        Ok(())
                    };

                    pop_section(name)?;
                    loop {
                        match closing.next() {
                            Some(Ok(Closing::Ident)) => {
                                pop_section(closing.slice())?;
                            }
                            Some(Ok(Closing::Match)) => break,
                            _ => return Err(Error::UnclosedTag),
                        }
                    }
                }
                Tag::Partial => {
                    match closing.next() {
                        Some(Ok(Closing::Match)) => {}
                        _ => return Err(Error::UnclosedTag),
                    }

                    self.blocks.push(Block::nameless(html, tag));
                    let partial = partials.get_partial(name)?;
                    self.blocks.extend_from_slice(&partial.blocks);
                    self.capacity_hint += partial.capacity_hint;
                }
                _ => {
                    loop {
                        match closing.next() {
                            Some(Ok(Closing::Ident)) => continue,
                            Some(Ok(Closing::Match)) => break,
                            _ => return Err(Error::UnclosedTag),
                        }
                    }
                    self.blocks.push(Block::nameless(html, tag));
                }
            };

            // Add the number of braces that we were expecting,
            // not the number we got:
            //
            // `{{foo}}}` should not consume the last `}`
            last = closing.span().start + closing.extras as usize;
            lex = closing.morph();
            lex.extras = Braces::Two;
        }

        Ok(last)
    }
}
