// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use crate::encoding::Encoder;
use pulldown_cmark::{Event, Tag};
use logos::Logos;

mod rust;

pub struct SyntaxPreprocessor<'a, I: Iterator<Item = Event<'a>>> {
    parent: I,
}

impl<'a, I: Iterator<Item = Event<'a>>> SyntaxPreprocessor<'a, I> {
    pub fn new(parent: I) -> Self {
        Self {
            parent
        }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for SyntaxPreprocessor<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (lang, highlight) = match self.parent.next()? {
            Event::Start(Tag::CodeBlock(lang)) => {
                match &*lang {
                    "rust" => (lang, highlight::<rust::Rust>),
                    _ => return Some(Event::Start(Tag::CodeBlock(lang))),
                }
            },
            other => return Some(other),
        };

        let mut code = String::new();

        for event in &mut self.parent {
            match event {
                Event::Text(text) => code.push_str(&text),
                Event::End(Tag::CodeBlock(ref l)) if *l == lang => break,
                other => println!("Unexpected event {:#?}", other),
            }
        }

        let html = highlight(&code);

        Some(Event::InlineHtml(html.into()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    None,
    Glyph,
    Literal,
    Identifier,
    SpecialIdentifier,
    Keyword,
    Comment,
}

static HIGHLIGHT_TAG: [Option<&'static str>; 7] = {
    let mut tags = [None; 7];

    tags[Kind::Glyph as usize] = Some("u");
    tags[Kind::Literal as usize] = Some("span");
    tags[Kind::Identifier as usize] = Some("var");
    tags[Kind::SpecialIdentifier as usize] = Some("em");
    tags[Kind::Keyword as usize] = Some("b");
    tags[Kind::Comment as usize] = Some("i");

    tags
};

pub trait Highlight: Sized {
    const LANG: &'static str;

    fn kind(tokens: &[Self; 2]) -> Kind;
}

pub fn highlight<'a, Token>(source: &'a str) -> String
where
    Token: Highlight + Logos + logos::source::WithSource<&'a str> + Eq + Copy,
{
    let mut buf = String::with_capacity(source.len());
    let mut open = Kind::None;
    let mut last = 0usize;

    let mut lex = Token::lexer(source);

    buf.push_str("<pre><code class=\"language-");
    buf.push_str(Token::LANG);
    buf.push_str("\">");

    let mut tokens = [Token::ERROR; 2];

    while lex.token != Token::END {
        tokens[0] = tokens[1];
        tokens[1] = lex.token;

        let kind = Token::kind(&tokens);

        if open != kind {
            // Close previous tag
            if let Some(tag) = HIGHLIGHT_TAG[open as usize] {
                buf.push_str("</");
                buf.push_str(tag);
                buf.push_str(">");
            }

            // Include trivia
            let _ = buf.write_escaped(&source[last..lex.range().start]);

            // Open new tag
            if let Some(tag) = HIGHLIGHT_TAG[kind as usize] {
                buf.push_str("<");
                buf.push_str(tag);
                buf.push_str(">");
            }

            open = kind;

            let _ = buf.write_escaped(lex.slice());
        } else {
            // Include trivia
            let _ = buf.write_escaped(&source[last..lex.range().end]);
        }

        last = lex.range().end;

        lex.advance();
    }

    // Close tail tag
    if let Some(tag) = HIGHLIGHT_TAG[open as usize] {
        buf.push_str("</");
        buf.push_str(tag);
        buf.push_str(">");
    }

    buf.push_str("</code></pre>");

    buf
}
