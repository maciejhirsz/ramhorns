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
fn c_simple_tera(b: &mut Bencher) {
    use tera::{Tera, Context};

    let mut tera = Tera::new("templates/includes/*").unwrap();

    tera.add_raw_template("t1", "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body|safe}}</div>").unwrap();

    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };
    let post = Context::from_serialize(&post).unwrap();

    b.iter(|| {
        black_box(tera.render("t1", &post).unwrap())
    });
}

#[bench]
fn c_simple_tera_from_serialize(b: &mut Bencher) {
    use tera::{Tera, Context};

    let mut tera = Tera::new("templates/includes/*").unwrap();

    tera.add_raw_template("t1", "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body|safe}}</div>").unwrap();

    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };
    // let post = Context::from_serialize(&post).unwrap();

    b.iter(|| {
        black_box(tera.render("t1", &Context::from_serialize(&post).unwrap()).unwrap())
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

#[bench]
fn pa_partials_ramhorns(b: &mut Bencher) {
    use ramhorns::Template;

    let tpl = Template::from_file("templates/basic.html").unwrap();
    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    b.iter(|| {
        black_box(tpl.render(&post))
    });
}

#[bench]
fn pb_partials_askama(b: &mut Bencher) {
    use askama::Template;

    #[derive(Template)]
    #[template(path = "askama.html")]
    struct Post<'a> {
        title: &'a str,
        body: &'a str,
    }

    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    b.iter(|| {
        black_box(post.render())
    });
}

#[bench]
fn pc_partials_mustache(b: &mut Bencher) {
    let tpl = mustache::compile_path("templates/bench.moustache").unwrap();

    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    b.iter(|| {
        black_box(tpl.render_to_string(&post))
    });
}

#[bench]
fn pd_partials_handlebars(b: &mut Bencher) {
    use handlebars::Handlebars;

    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("t1", "templates/basic.html").unwrap();
    handlebars.register_template_file("head.rh", "templates/head.html").unwrap();
    handlebars.register_template_file("footer.rh", "templates/footer.html").unwrap();

    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    b.iter(|| {
        black_box(handlebars.render("t1", &post))
    });
}
