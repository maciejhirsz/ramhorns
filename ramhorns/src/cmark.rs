// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use pulldown_cmark::Parser;

use crate::encoding::Encoder;

pub fn encode<E: Encoder>(source: &str, encoder: &mut E) -> Result<(), E::Error> {
    let parser = Parser::new(source);

    encoder.write_html(parser)
}
