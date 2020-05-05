// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use logos::Logos;
use arrayvec::ArrayVec;

use crate::Partials;
use super::{Block, Error, Tag, Template};

#[derive(Logos)]
#[logos(extras = Braces)]
enum Opening {
    #[token("{{", |_| Tag::Escaped)]
    #[token("{{&", |_| Tag::Unescaped)]
    #[token("{{{", |lex| {
        // Flag that we will expect 3 closing braces
        lex.extras = Braces::Three;

        Tag::Unescaped
    })]
    #[token("{{#", |_| Tag::Section)]
    #[token("{{^", |_| Tag::Inverse)]
    #[token("{{/", |_| Tag::Closing)]
    #[token("{{>", |_| Tag::Partial)]
    #[token("{{!", |_| Tag::Comment)]
    Match(Tag),

    #[regex(r"[^{]+", logos::skip)]
    #[token("{", logos::skip)]
    #[error]
    Err,
}

#[derive(Logos)]
#[logos(extras = Braces)]
enum Closing {
    #[token("}}", |lex| {
        // Force fail the match if we expected 3 braces
        lex.extras != Braces::Three
    })]
    #[token("}}}")]
    Match,

    #[regex(r"[^}]+", logos::skip)]
    #[error]
    Err
}

/// Marker of how many braces we expect to match
#[derive(PartialEq, Eq, Clone, Copy)]
enum Braces {
    Two = 2,
    Three = 3,
}

impl Default for Braces {
    #[inline]
    fn default() -> Self {
        Braces::Two
    }
}

impl<'tpl> Template<'tpl> {
    pub(crate) fn parse(
        &mut self,
        source: &'tpl str,
        partials: &mut impl Partials<'tpl>,
    ) -> Result<usize, Error> {
        let mut last = 0;
        let mut lex = Opening::lexer(source);
        let mut stack = ArrayVec::<[usize; 16]>::new();

        while let Some(token) = lex.next() {
            let tag = match token {
                Opening::Match(tag) => tag,
                Opening::Err => return Err(Error::UnclosedTag),
            };

            // Grab HTML from before the token
            // TODO: add lex.before() that yields source slice
            // in front of the token:
            //
            // let html = &lex.before()[last..];
            let html = &lex.source()[last..lex.span().start];
            self.capacity_hint += html.len();
            last = lex.span().end;

            // Morphing the lexer to match the closing
            // braces and grab the name
            let mut closing = lex.morph();

            match closing.next() {
                Some(Closing::Match) => (),
                _ => return Err(Error::UnclosedTag),
            };

            // Ditto about lex.before()
            let name = source[last..closing.span().start].trim();

            // Add the number of braces that we were expecting,
            // not the number we got:
            //
            // `{{foo}}}` should not consume the last `}`
            last = closing.span().start + closing.extras as usize;
            lex = closing.morph();
            lex.extras = Braces::Two;

            // Push a new block
            let tail_idx = self.blocks.len();
            let block = Block::new(html, name, tag);
            let hash = block.hash;

            self.blocks.push(block);

            match tag {
                Tag::Section | Tag::Inverse => {
                    stack.try_push(tail_idx)?;
                }
                Tag::Closing => {
                    let head_idx = stack.pop().ok_or_else(|| Error::UnopenedSection(name.into()))?;
                    let head = &mut self.blocks[head_idx];

                    head.children = (tail_idx - head_idx) as u32;

                    if head.hash != hash {
                        return Err(Error::UnclosedSection(head.name.into()));
                    }
                }
                Tag::Partial => {
                    let partial = partials.get_partial(name)?;
                    self.blocks.extend_from_slice(&partial.blocks);
                    self.capacity_hint += partial.capacity_hint;
                }
                _ => {}
            };
        }

        Ok(last)
    }
}
