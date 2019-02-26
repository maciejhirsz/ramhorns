use crate::fnv;
use crate::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Braces {
	Two = 2,
	Three = 3,
}

pub struct Template<'template> {
	capacity: usize,
	chunks: Vec<&'template str>,
	vtable: Vec<Variable<'template>>,
}

impl<'template> Template<'template> {
	pub fn new(source: &'template str) -> Self {
		let capacity = source.len();
		let mut chunks = Vec::new();
		let mut vtable = Vec::new();
		let mut last = 0;

		if let Some(bytes) = source.as_bytes().get(..source.len() - 1) {
			let mut indexed = bytes.iter().map(|b| unsafe { &*(b as *const u8 as *const [u8; 2]) }).enumerate();

			while let Some((start, bytes)) = indexed.next() {
				if bytes == b"{{" {
					// Skip a byte since we got a double
					indexed.next();

					let mut kind = VKind::Escaped;
					let mut braces = Braces::Two;

					while let Some((_, bytes)) = indexed.next() {
						match bytes[0] {
							b'{' => {
								kind = VKind::Unescaped;
								braces = Braces::Three;
							},
							b'!' => kind = VKind::Comment,
							b' ' | b'\t' | b'\r' | b'\n' => continue,
							_ => {}
						}

						break;
					}

					chunks.push(&source[last..start]);

					while let Some((end, bytes)) = indexed.next() {
						if bytes == b"}}" {
							// Skip a byte since we got a double
							indexed.next();

							if braces == Braces::Three {
								// TODO: verify that there is a third brace
								indexed.next();
							}

							let name = source[start + braces as usize..end].trim();

							vtable.push(Variable::new(kind, name, usize::max_value()));

							last = end + braces as usize;
							break;
						}
					}
				}
			}
		}

		chunks.push(&source[last..]);

		Self::finalize_vtable(&mut vtable);

		Template {
			capacity,
			chunks,
			vtable,
		}
	}

	pub fn render<'ctx, Ctx: Context<'ctx>>(&self, ctx: &'ctx Ctx) -> String {
		let mut buf = String::with_capacity(self.capacity);

		let fields = ctx.to_fields();
		let fields = fields.as_ref();

		buf.push_str(self.chunks[0]);

		for (chunk, var) in self.chunks[1..].iter().zip(&self.vtable) {
			// TODO: Handle the errors
			if let Some(field) = fields.get(var.field) {
				if field.hash == var.hash {
					match var.kind {
						VKind::Escaped => Self::escape_write(&mut buf, &field.value),
						VKind::Unescaped => buf.push_str(&field.value),
						VKind::Comment => {},
					}
				}
			}

			buf.push_str(chunk);
		}

		buf
	}

	fn finalize_vtable(vtable: &mut Vec<Variable>) {
		let mut temp = Vec::with_capacity(vtable.len());

		for var in vtable.iter().filter(|var| var.kind != VKind::Comment) {
			if let Err(index) = temp.binary_search(&var.name) {
				temp.insert(index, var.name);
			}
		}

		for var in vtable.iter_mut().filter(|var| var.kind != VKind::Comment) {
			if let Ok(index) = temp.binary_search(&var.name) {
				var.field = index;
			}
		}
	}

	fn escape_write(buf: &mut String, part: &str) {
		let mut start = 0;

		for (idx, byte) in part.bytes().enumerate() {
			let replace = match byte {
				b'<' => "&lt;",
				b'>' => "&gt;",
				b'&' => "&amp;",
				b'"' => "&quot;",
				_ => continue,
			};

			buf.push_str(&part[start..idx]);
			buf.push_str(replace);

			start = idx + 1;
		}

		buf.push_str(&part[start..]);
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VKind {
	Escaped,
	Unescaped,
	Comment,
}

#[derive(Debug, PartialEq, Eq)]
struct Variable<'template> {
	pub kind: VKind,
	pub name: &'template str,
	pub hash: u64,
	pub field: usize,
}

impl<'template> Variable<'template> {
	fn new(kind: VKind, name: &'template str, field: usize) -> Self {
		let hash = fnv::hash(name);

		Variable {
			kind,
			name,
			hash,
			field,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::context::Field;

	#[test]
	fn extracts_chunks_correctly() {
		let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";
		let tpl = Template::new(source);

		assert_eq!(&tpl.chunks, &["<title>", "</title><h1>", "</h1><div>", "</div>"]);
	}

	#[test]
	fn constructs_var_table_correctly() {
		let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";
		let tpl = Template::new(source);

		assert_eq!(&tpl.vtable, &[
			Variable::new(VKind::Escaped, "title", 1), // Names are sorted alphabeticaly, so all
			Variable::new(VKind::Escaped, "title", 1), // `title` entries map to field 1, while
			Variable::new(VKind::Unescaped, "body", 0),  // all `body` entries map to field 0.
		]);
	}

	#[test]
	fn var_table_record_hashes_correctly() {
		assert_eq!(Variable::new(VKind::Escaped, "test", 42), Variable {
			kind: VKind::Escaped,
			name: "test",
			hash: 0xf9e6e6ef197c2b25,
			field: 42,
		});
	}

	#[test]
	fn simple_render() {
		struct Post<'a> {
			title: &'a str,
			body: &'a str,
		}

		impl<'a, 'ctx> Context<'ctx> for Post<'a> {
			type Fields = [Field<'ctx>; 2];

			fn to_fields(&'ctx self) -> Self::Fields {
				[
					Field::from_name("body", self.body),
					Field::from_name("title", self.title),
				]
			}
		}

		let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
		let tpl = Template::new(source);

		let rendered = tpl.render(&Post {
			title: "Hello, Ramhorns!",
			body: "This is a really simple test of the rendering!",
		});

		assert_eq!(&rendered, "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
							   <div>This is a really simple test of the rendering!</div>");
	}

	#[test]
	fn simple_render_with_comments() {
		struct Post<'a> {
			title: &'a str,
			body: &'a str,
		}

		impl<'a, 'ctx> Context<'ctx> for Post<'a> {
			type Fields = [Field<'ctx>; 2];

			fn to_fields(&'ctx self) -> Self::Fields {
				[
					Field::from_name("body", self.body),
					Field::from_name("title", self.title),
				]
			}
		}

		let source = "<title>{{ ! ignore me }}{{title}}</title>{{!-- nothing to look at here --}}<h1>{{ title }}</h1><div>{{body}}</div>";
		let tpl = Template::new(source);

		let rendered = tpl.render(&Post {
			title: "Hello, Ramhorns!",
			body: "This is a really simple test of the rendering!",
		});

		assert_eq!(&rendered, "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
							   <div>This is a really simple test of the rendering!</div>");
	}

	#[test]
	fn escaped_vs_unescaped() {
		struct Dummy;

		impl<'ctx> Context<'ctx> for Dummy {
			type Fields = [Field<'ctx>; 1];

			fn to_fields(&'ctx self) -> Self::Fields {
				[
					Field::from_name("dummy", "This is a <strong>test</strong>!"),
				]
			}
		}

		let tpl = Template::new("Escaped: {{dummy}} Unescaped: {{{dummy}}}");

		let rendered = tpl.render(&Dummy);

		assert_eq!(rendered, "Escaped: This is a &lt;strong&gt;test&lt;/strong&gt;! \
							  Unescaped: This is a <strong>test</strong>!")
	}
}
