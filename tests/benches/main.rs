#![feature(test)]
extern crate test;

use test::{Bencher, black_box};
use ramhorns::Content;
use serde_derive::Serialize;
use askama::Template;

static SOURCE: &str = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>";

#[derive(Content, Serialize, Template)]
#[template(source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body|safe}}</div>", ext = "html")]
struct Post<'a> {
    title: &'a str,
    body: &'a str,
}

#[bench]
fn a_simple_ramhorns(b: &mut Bencher) {
    use ramhorns::Template;

    let tpl = Template::new(SOURCE).unwrap();
    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    b.iter(|| {
        black_box(tpl.render(&post))
    });
}

#[bench]
fn b_simple_wearte(b: &mut Bencher) {
    use wearte::Template;

    #[derive(Serialize, Template)]
    #[template(source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{{body}}}</div>", ext = "html")]
    struct Post<'a> {
        title: &'a str,
        body: &'a str,
    }

    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    b.iter(|| {
        black_box(post.call())
    });
}

#[bench]
fn c_simple_askama(b: &mut Bencher) {
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
fn d_simple_mustache(b: &mut Bencher) {
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
fn e_simple_handlebars(b: &mut Bencher) {
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
