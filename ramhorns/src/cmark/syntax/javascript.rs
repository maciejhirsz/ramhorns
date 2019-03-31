// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use logos::Logos;
use super::{Highlight, Kind};

#[derive(Logos, PartialEq, Eq, Clone, Copy)]
pub enum JavaScript {
    #[error]
    Error,

    #[end]
    End,

    #[regex = "[a-zA-Z_$][a-zA-Z0-9_]*"]
    Identifier,

    #[regex = "\"([^\"\\\\]|\\\\[.\n])*\""]
    #[regex = "`([^`]|\\\\`)*`"]
    #[regex = "'([^']|\\\\')'"]
    #[regex = "[0-9][0-9]*"]
    #[regex = "0[xX][0-9a-fA-F]+"]
    #[regex = "0[oO][0-7]+"]
    #[regex = "0[bB][01]+"]
    Literal,

    #[regex = r#"\?|:|!|\^|-|\+|\*|&|/|\|<|>|=|=>|_"#]
    Glyph,

    #[regex = r"\."]
    GlyphCtx,

    #[regex = "arguments|async|await|break|case|catch|const|continue"]
    #[regex = "debugger|default|delete|do|else|enum|eval|export|extends"]
    #[regex = "false|finally|for|if|implements|import|in|instanceof"]
    #[regex = "interface|let|long|native|new|null|package|private"]
    #[regex = "protected|public|return|static|super|switch|this|throw"]
    #[regex = "true|try|typeof|var|void|while|with|yield"]
    Keyword,

    #[regex = "function|class"]
    KeywordCtx,

    #[regex = "undefined|Object|Array|Number|String|NaN|Infinity|Date|Math"]
    Special,

    #[regex = "//[^\n]*"]
    Comment,
}

impl Highlight for JavaScript {
    const LANG: &'static str = "js";

    fn kind(tokens: &[Self; 2]) -> Kind {
        use JavaScript::*;

        match tokens {
            [KeywordCtx, Identifier] |
            [GlyphCtx, Identifier]   |
            [_, Special]             => Kind::SpecialIdentifier,
            [_, Identifier]          => Kind::Identifier,
            [_, Literal]             => Kind::Literal,
            [_, Glyph]               |
            [_, GlyphCtx]            => Kind::Glyph,
            [_, Keyword]             |
            [_, KeywordCtx]          => Kind::Keyword,
            [_, Comment]             => Kind::Comment,
            _                        => Kind::None,
        }
    }
}
