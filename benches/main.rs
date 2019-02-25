#![feature(test)]
extern crate test;

use test::{Bencher, black_box};
use ramhorns::context::{Context, Field};
use serde_derive::Serialize;

#[derive(Serialize)]
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

static SOURCE: &str = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";

#[bench]
fn simple_ramhorns(b: &mut Bencher) {
	use ramhorns::Template;

	let tpl = Template::new(SOURCE);
	let post = Post {
		title: "Hello, Ramhorns!",
		body: "This is a really simple test of the rendering!",
	};

    b.iter(|| {
    	black_box(tpl.render(&post))
    });
}

#[bench]
fn simple_handlebars(b: &mut Bencher) {
	use handlebars::Handlebars;

	let mut handlebars = Handlebars::new();

	handlebars.register_template_string("t1", SOURCE).unwrap();

	let post = Post {
		title: "Hello, Ramhorns!",
		body: "This is a really simple test of the rendering!",
	};

    b.iter(|| {
    	black_box(handlebars.render("t1", &post))
    });
}
