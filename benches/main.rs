#![feature(test)]
extern crate test;

use test::{Bencher, black_box};
use ramhorns::context::{Context, Field};
use serde_derive::Serialize;
use askama::Template;

static SOURCE: &str = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";

#[derive(Serialize, Template)]
#[template(source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body|safe}}</div>", ext = "html")]
struct Post<'a> {
	title: &'a str,
	body: &'a str,
}

impl<'a, 'ctx> Context<'ctx> for Post<'a> {
	type Fields = [Field<'ctx>; 2];

	fn to_fields(&'ctx self) -> Self::Fields {
		[
			// Field::from_name("body", self.body),
			// Field::from_name("title", self.title),
			Field::new(0xcd4de79bc6c93295, self.body),
			Field::new(0xda31296c0c1b6029, self.title),
		]
	}
}

#[bench]
fn a_simple_ramhorns(b: &mut Bencher) {
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
fn b_simple_askama(b: &mut Bencher) {
	use askama::Template;

	let post = Post {
		title: "Hello, Ramhorns!",
		body: "This is a really simple test of the rendering!",
	};

    b.iter(|| {
    	black_box(post.render())
    });
}

#[bench]
fn c_simple_mustache(b: &mut Bencher) {
	let tpl = mustache::compile_str(SOURCE).unwrap();

	let post = Post {
		title: "Hello, Ramhorns!",
		body: "This is a really simple test of the rendering!",
	};

    b.iter(|| {
    	black_box(tpl.render_to_string(&post))
    });
}

#[bench]
fn d_simple_handlebars(b: &mut Bencher) {
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
