use crate::fnv;
use crate::context::{Context, Field};

pub struct Template<'template> {
	capacity: usize,
	chunks: Vec<&'template str>,
	var_table: VarTable<'template>,
}

impl<'template> Template<'template> {
	pub fn new(source: &'template str) -> Self {
		let capacity = source.len();
		let mut chunks = Vec::new();
		let mut vars = Vec::new();
		let mut last = 0;

		if let Some(bytes) = source.as_bytes().get(..source.len() - 1) {
			let mut indexed = bytes.iter().map(|b| unsafe { &*(b as *const u8 as *const [u8; 2]) }).enumerate();

			while let Some((start, bytes)) = indexed.next() {
				if bytes == b"{{" {
					// Skip a byte since we got a double
					indexed.next();

					chunks.push(&source[last..start]);

					while let Some((end, bytes)) = indexed.next() {
						if bytes == b"}}" {
							// Skip a byte since we got a double
							indexed.next();

							vars.push(source[start + 2..end].trim());

							last = end + 2;
							break;
						}
					}
				}
			}
		}

		chunks.push(&source[last..]);

		let var_table = VarTable::from_names(&vars);

		Template {
			capacity,
			chunks,
			var_table,
		}
	}

	fn render<'ctx, Ctx: Context<'ctx>>(&self, ctx: &'ctx Ctx) -> String {
		let mut buf = String::with_capacity(self.capacity);

		let fields = ctx.to_fields();
		let fields = fields.as_ref();

		buf.push_str(self.chunks[0]);

		for (chunk, record) in self.chunks[1..].iter().zip(&self.var_table.table) {
			// TODO: Handle the errors
			if let Some(field) = fields.get(record.field) {
				if field.hash == record.hash {
					buf.push_str(field.value);
				}
			}

			buf.push_str(chunk);
		}

		buf
	}
}

#[derive(Debug, PartialEq, Eq)]
struct VarTableRecord<'template> {
	pub name: &'template str,
	pub hash: u64,
	pub field: usize,
}

impl<'template> VarTableRecord<'template> {
	fn new(name: &'template str, field: usize) -> Self {
		let hash = fnv::hash(name);

		VarTableRecord {
			name,
			hash,
			field,
		}
	}
}

struct VarTable<'template> {
	table: Vec<VarTableRecord<'template>>
}

impl<'template> VarTable<'template> {
	pub fn from_names(names: &[&'template str]) -> Self {
		let mut temp = Vec::with_capacity(names.len());

		for name in names {
			if let Err(index) = temp.binary_search(name) {
				temp.insert(index, name);
			}
		}

		let table = names.iter()
			.map(|name| {
				let field = temp.binary_search(name).expect("Inserted all, can't fail; qed");

				VarTableRecord::new(name, field)
			})
			.collect();

		VarTable {
			table,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn extracts_chunks_correctly() {
		let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
		let tpl = Template::new(source);

		assert_eq!(&tpl.chunks, &["<title>", "</title><h1>", "</h1><div>", "</div>"]);
	}

	#[test]
	fn constructs_var_table_correctly() {
		let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
		let tpl = Template::new(source);

		assert_eq!(&tpl.var_table.table, &[
			VarTableRecord::new("title", 1), // Names are sorted alphabeticaly, so all
			VarTableRecord::new("title", 1), // `title` entries map to field 1, while
			VarTableRecord::new("body", 0),  // all `body` entries map to field 0.
		]);
	}

	#[test]
	fn var_table_record_hashes_correctly() {
		assert_eq!(VarTableRecord::new("test", 42), VarTableRecord {
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
					Field {
						hash: fnv::hash("body"),
						value: self.body,
					},
					Field {
						hash: fnv::hash("title"),
						value: self.title,
					}
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
}
