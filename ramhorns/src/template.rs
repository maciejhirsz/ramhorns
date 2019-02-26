// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use fnv::FnvHasher;
use std::hash::Hasher;

pub struct Template<'tpl> {
	/// Processed `Block`s of the template
	section: Section<'tpl>,

	/// Tailing html that isn't part of any `Block`
	tail: &'tpl str,
}

impl<'tpl> Template<'tpl> {
	pub fn new(source: &'tpl str) -> Self {
		let mut iter = source.as_bytes()
			.get(..source.len() - 1)
			.unwrap_or(&[])
			.iter()
			.map(|b| unsafe { &*(b as *const u8 as *const [u8; 2]) })
			.enumerate();

		let mut section = Section::new();
		let mut last = 0;

		section.parse(source, &mut iter, &mut last, None);

		let tail = &source[last..];

		Template {
			section,
			tail,
		}
	}

	pub fn capacity_hint(&self) -> usize {
		self.section.capacity_hint + self.tail.len()
	}

	pub fn render<Context: crate::Context>(&self, ctx: &Context) -> String {
		let mut capacity = ctx.capacity_hint(self);

		// Add extra 25% extra capacity for HTML escapes and an odd double variable use.
		capacity += capacity / 4;

		let mut buf = String::with_capacity(capacity);

		self.section.render_once(ctx, &mut buf);

		buf.push_str(self.tail);
		buf
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
	Escaped,
	Unescaped,
	Section(usize),
	Inverse(usize),
	Closing,
	Comment,
}

#[derive(Debug, PartialEq, Eq)]
struct Block<'tpl> {
	html: &'tpl str,
	name: &'tpl str,
	hash: u64,
	tag: Tag,
}

impl<'tpl> Block<'tpl> {
	fn new(html: &'tpl str, name: &'tpl str, tag: Tag) -> Self {
		let mut hasher = FnvHasher::default();

		hasher.write(name.as_bytes());

		let hash = hasher.finish();

		Block {
			html,
			name,
			hash,
			tag,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct Section<'tpl> {
	blocks: Vec<Block<'tpl>>,
	capacity_hint: usize,
}

impl<'tpl> Section<'tpl> {
	pub fn render_once<Context: crate::Context>(&self, ctx: &Context, buf: &mut String) {
		let mut index = 0;

		while let Some(block) = self.blocks.get(index) {
			buf.push_str(block.html);

			match block.tag {
				Tag::Escaped => ctx.render_escaped(block.hash, buf),
				Tag::Unescaped => ctx.render_unescaped(block.hash, buf),
				Tag::Section(count) => index += count, // ctx.render_section(block.hash, section, buf),
				Tag::Inverse(count) => index += count, //ctx.render_inverse(block.hash, section, buf),
				Tag::Closing |
				Tag::Comment => {},
			}

			index += 1;
		}
	}

	fn new() -> Self {
		Section {
			blocks: Vec::new(),
			capacity_hint: 0,
		}
	}

	fn parse<Iter>(&mut self, source: &'tpl str, iter: &mut Iter, last: &mut usize, until: Option<&'tpl str>) -> usize
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
						},
						b'#' => tag = Tag::Section(0),
						b'^' => tag = Tag::Inverse(0),
						b'/' => tag = Tag::Closing,
						b'!' => tag = Tag::Comment,
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

				while let Some((end, bytes)) = iter.next() {
					if bytes == b"}}" {
						// Skip a byte since we got a double
						iter.next();

						if end_skip == 3 {
							// TODO: verify that there is a third brace
							iter.next();
						}

						let name = source[start + start_skip..end].trim();

						*last = end + end_skip;

						let insert_index = self.blocks.len();

						self.capacity_hint += html.len();
						self.blocks.insert(insert_index, Block::new(html, name, tag));

						match tag {
							Tag::Section(_) |
							Tag::Inverse(_) => {
								let count = self.parse(source, iter, last, Some(name));

								match self.blocks[insert_index].tag {
									Tag::Section(ref mut c) |
									Tag::Inverse(ref mut c) => *c = count,
									_ => {},
								}
							},
							Tag::Closing => {
								if until.map(|until| until != name).unwrap_or(false) {
									// TODO: handle error here
								}

								return self.blocks.len() - blocks_at_start;
							},
							_ => {},
						};

						break;
					}
				}
			}
		}

		if until.is_some() {
			// TODO: handle error here
		}

		self.blocks.len() - blocks_at_start
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn block_hashes_correctly() {
		assert_eq!(Block::new("", "test", Tag::Escaped), Block {
			html: "",
			name: "test",
			hash: 0xf9e6e6ef197c2b25,
			tag: Tag::Escaped,
		});
	}

	#[test]
	fn constructs_blocks_correctly() {
		let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";
		let tpl = Template::new(source);

		assert_eq!(&tpl.section.blocks, &[
			Block::new("<title>", "title", Tag::Escaped),
			Block::new("</title><h1>", "title", Tag::Escaped),
			Block::new("</h1><div>", "body", Tag::Unescaped),
		]);

		assert_eq!(tpl.tail, "</div>");
	}

	#[test]
	fn constructs_nested_sections_correctly() {
		let source = "<body><h1>{{ title }}</h1>{{#posts}}<article>{{name}}</article>{{/posts}}{{^posts}}<p>Nothing here :(</p>{{/posts}}</body>";
		let tpl = Template::new(source);


		assert_eq!(&tpl.section.blocks, &[
			Block::new("<body><h1>", "title", Tag::Escaped),
			Block::new("</h1>", "posts", Tag::Section(2)),
			Block::new("<article>", "name", Tag::Escaped),
			Block::new("</article>", "posts", Tag::Closing),
			Block::new("", "posts", Tag::Inverse(1)),
			Block::new("<p>Nothing here :(</p>", "posts", Tag::Closing),
		]);

		assert_eq!(tpl.tail, "</body>");
	}
}
