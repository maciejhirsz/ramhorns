// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use logos::Logos;
use super::Highlight;

#[derive(Logos, PartialEq, Eq, Clone, Copy)]
pub enum Rust {
    #[error]
    Error,

    #[end]
    End,

    #[regex = "[a-zA-Z_$][a-zA-Z0-9_]*"]
    Identifier,

    #[regex = "\"([^\"\\\\]|\\\\[.\n])*\""]
    #[regex = "'([^']|\\\\')'"]
    #[regex = "[0-9][0-9_]*"]
    #[regex = "0[xX][0-9a-fA-F_]+"]
    #[regex = "0[oO][0-7_]+"]
    #[regex = "0[bB][01_]+"]
    Literal,

    #[regex = r#"\?|!|\^|-|\+|\*|&|/|\||=|->|=>|_|#\[[^\]]*\]"#]
    Special,

    #[regex = r"\.|:|(&|'[a-zA-Z_][a-zA-Z0-9_]*)([ \t\n\r]*mut[ \t\n\r]+)"]
    ContextSpecial,

    #[regex = "as|break|const|continue|crate|dyn|else|extern"]
    #[regex = "false|for|if|impl|in|let|loop|match|mod|move|mut"]
    #[regex = "pub|ref|return|self|Self|static|super|trait"]
    #[regex = "true|type|unsafe|use|where|while"]
    #[regex = "abstract|async|await|become|box|do|final|macro"]
    #[regex = "override|priv|try|typeof|unsized|virtual|yield"]
    Keyword,

    #[regex = "fn|enum|struct"]
    ContextKeyword,

    #[regex = "Some|None|Ok|Err|str|bool|[ui](8|16|32|64|size)|f32|f64"]
    Common,

    #[regex = "//[^\n]*"]
    Comment,
}

impl Highlight for Rust {
    const LANG: &'static str = "rust";

    fn tag(tokens: &[Self; 2]) -> Option<&'static str> {
        use Rust::*;

        match tokens {
            [ContextKeyword, Identifier] |
            [ContextSpecial, Identifier] => Some("em"),
            [_, Identifier] => Some("var"),
            [_, Common] => Some("em"),
            [_, Literal] => Some("span"),
            [_, Special] => Some("u"),
            [_, ContextSpecial] => Some("u"),
            [_, Keyword] => Some("b"),
            [_, ContextKeyword] => Some("b"),
            [_, Comment] => Some("i"),
            _ => None,
        }
    }
}
