use pulldown_cmark::Parser;

use crate::encoding::Encoder;

pub fn encode<E: Encoder>(source: &str, encoder: &mut E) -> Result<(), E::Error> {
    let parser = Parser::new(source);

    encoder.write_html(parser)
}
