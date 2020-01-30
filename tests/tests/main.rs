use ramhorns::{Content, Template, Templates};

#[derive(Content)]
struct Post<'a> {
    title: &'a str,
    body: &'a str,
}

#[test]
fn simple_render() {
    let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source).unwrap();

    let rendered = tpl.render(&Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    });

    assert_eq!(
        &rendered,
        "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
         <div>This is a really simple test of the rendering!</div>"
    );
}

#[test]
fn simple_render_to_writer() {
    let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source).unwrap();

    let mut buf = Vec::new();

    tpl.render_to_writer(
        &mut buf,
        &Post {
            title: "Hello, Ramhorns!",
            body: "This is a really simple test of the rendering!",
        },
    )
    .unwrap();

    assert_eq!(
        &buf[..],
        &b"<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
                            <div>This is a really simple test of the rendering!</div>"[..]
    );
}

#[test]
fn simple_render_hash_map() {
    use std::collections::HashMap;

    let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source).unwrap();

    let mut map = HashMap::new();

    map.insert("title", "Hello, Ramhorns!");
    map.insert(
        "body",
        "This is a test of rendering a template with a HashMap Content!",
    );

    let rendered = tpl.render(&map);

    assert_eq!(
        &rendered,
        "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
         <div>This is a test of rendering a template with a HashMap Content!</div>"
    );
}

#[test]
fn simple_render_btree_map() {
    use std::collections::BTreeMap;

    let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source).unwrap();

    let mut map = BTreeMap::new();

    map.insert("title", "Hello, Ramhorns!");
    map.insert(
        "body",
        "This is a test of rendering a template with a BTreeMap Content!",
    );

    let rendered = tpl.render(&map);

    assert_eq!(
        &rendered,
        "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
         <div>This is a test of rendering a template with a BTreeMap Content!</div>"
    );
}

#[test]
fn simple_render_with_comments() {
    let source = "<title>{{ ! ignore me }}{{title}}</title>{{!-- nothing to look at here --}}<h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source).unwrap();

    let rendered = tpl.render(&Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    });

    assert_eq!(
        &rendered,
        "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
         <div>This is a really simple test of the rendering!</div>"
    );
}

#[test]
fn escaped_vs_unescaped() {
    #[derive(Content)]
    struct Dummy {
        dummy: &'static str,
    }

    let tpl = Template::new("Escaped: {{dummy}} Unescaped: {{{dummy}}}").unwrap();

    let rendered = tpl.render(&Dummy {
        dummy: "This is a <strong>test</strong>!",
    });

    assert_eq!(
        rendered,
        "Escaped: This is a &lt;strong&gt;test&lt;/strong&gt;! \
         Unescaped: This is a <strong>test</strong>!"
    );
}

#[test]
fn escaped_vs_unescaped_ampersand() {
    #[derive(Content)]
    struct Dummy {
        dummy: &'static str,
    }

    let tpl = Template::new("Escaped: {{dummy}} Unescaped: {{& dummy}}").unwrap();

    let rendered = tpl.render(&Dummy {
        dummy: "This is a <strong>test</strong>!",
    });

    assert_eq!(
        rendered,
        "Escaped: This is a &lt;strong&gt;test&lt;/strong&gt;! \
         Unescaped: This is a <strong>test</strong>!"
    );
}

#[test]
fn handles_tuple_structs() {
    #[derive(Content)]
    struct Dummy(&'static str, &'static str, &'static str);

    let tpl = Template::new("{{1}} {{2}} {{0}}").unwrap();

    let rendered = tpl.render(&Dummy("zero", "one", "two"));

    assert_eq!(rendered, "one two zero");
}

#[test]
fn handles_unit_structs() {
    #[derive(Content)]
    struct Dummy;

    let tpl = Template::new("This is pretty silly, but why not?").unwrap();

    let rendered = tpl.render(&Dummy);

    assert_eq!(rendered, "This is pretty silly, but why not?");
}

#[test]
fn simple_render_with_strings() {
    #[derive(Content)]
    struct Post {
        title: String,
        body: String,
    }

    let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source).unwrap();

    let rendered = tpl.render(&Post {
        title: "Hello, Ramhorns!".to_string(),
        body: "This is a really simple test of the rendering!".to_string(),
    });

    assert_eq!(
        &rendered,
        "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
         <div>This is a really simple test of the rendering!</div>"
    );
}

#[test]
fn simple_render_different_types() {
    #[derive(Content)]
    struct MeaningOfLife {
        meaning: i32,
        truth: bool,
    }

    let source = "The meaning of life is {{meaning}}? {{truth}}!";
    let tpl = Template::new(source).unwrap();

    let correct = tpl.render(&MeaningOfLife {
        meaning: 42,
        truth: true,
    });

    let incorrect = tpl.render(&MeaningOfLife {
        meaning: -9001,
        truth: false,
    });

    assert_eq!(&correct, "The meaning of life is 42? true!");
    assert_eq!(&incorrect, "The meaning of life is -9001? false!");
}

#[test]
fn can_render_sections_from_bool() {
    #[derive(Content)]
    struct Conditional {
        secret: bool,
    }

    let tpl = Template::new("Hello!{{#secret}} This is a secret!{{/secret}}").unwrap();

    let show = tpl.render(&Conditional { secret: true });
    let hide = tpl.render(&Conditional { secret: false });

    assert_eq!(show, "Hello! This is a secret!");
    assert_eq!(hide, "Hello!");
}

#[test]
fn can_render_inverse_sections_from_bool() {
    #[derive(Content)]
    struct Conditional {
        secret: bool,
    }

    let tpl = Template::new("Hello!{{^secret}} This is NOT a secret!{{/secret}}").unwrap();

    let show = tpl.render(&Conditional { secret: true });
    let hide = tpl.render(&Conditional { secret: false });

    assert_eq!(show, "Hello!");
    assert_eq!(hide, "Hello! This is NOT a secret!");
}

#[test]
fn can_render_inverse_sections_for_empty_strs() {
    #[derive(Content)]
    struct Person<'a> {
        name: &'a str,
    }

    let tpl = Template::new("Hello {{name}}{{^name}}Anonymous{{/name}}!").unwrap();

    let named = tpl.render(&Person { name: "Maciej" });
    let unnamed = tpl.render(&Person { name: "" });

    assert_eq!(named, "Hello Maciej!");
    assert_eq!(unnamed, "Hello Anonymous!");
}

#[test]
fn can_render_lists_from_slices() {
    #[derive(Content)]
    struct Article<'a> {
        title: &'a str,
    }

    #[derive(Content)]
    struct Page<'a> {
        title: &'a str,
        articles: &'a [Article<'a>],
    }

    let tpl = Template::new(
        "<h1>{{title}}</h1>\
         {{#articles}}<article>{{title}}</article>{{/articles}}\
         {{^articles}}<p>No articles :(</p>{{/articles}}",
    )
    .unwrap();

    let blog = tpl.render(&Page {
        title: "Awesome Blog",
        articles: &[
            Article {
                title: "How is Ramhorns this fast?",
            },
            Article {
                title: "Look at that cat pic!",
            },
            Article {
                title: "Hello World!",
            },
        ],
    });

    let empty = tpl.render(&Page {
        title: "Sad page :(",
        articles: &[],
    });

    assert_eq!(
        blog,
        "<h1>Awesome Blog</h1>\
         <article>How is Ramhorns this fast?</article>\
         <article>Look at that cat pic!</article>\
         <article>Hello World!</article>"
    );

    assert_eq!(
        empty,
        "<h1>Sad page :(</h1>\
         <p>No articles :(</p>"
    );
}

#[test]
fn can_render_lists_from_vecs() {
    #[derive(Content)]
    struct Article {
        title: String,
    }

    #[derive(Content)]
    struct Page {
        title: String,
        articles: Vec<Article>,
    }

    let tpl = Template::new(
        "<h1>{{title}}</h1>\
         {{#articles}}<article>{{title}}</article>{{/articles}}\
         {{^articles}}<p>No articles :(</p>{{/articles}}",
    )
    .unwrap();

    let blog = tpl.render(&Page {
        title: "Awesome Blog".into(),
        articles: vec![
            Article {
                title: "How is Ramhorns this fast?".into(),
            },
            Article {
                title: "Look at that cat pic!".into(),
            },
            Article {
                title: "Hello World!".into(),
            },
        ],
    });

    let empty = tpl.render(&Page {
        title: "Sad page :(".into(),
        articles: vec![],
    });

    assert_eq!(
        blog,
        "<h1>Awesome Blog</h1>\
         <article>How is Ramhorns this fast?</article>\
         <article>Look at that cat pic!</article>\
         <article>Hello World!</article>"
    );

    assert_eq!(
        empty,
        "<h1>Sad page :(</h1>\
         <p>No articles :(</p>"
    );
}

#[test]
fn can_render_markdown() {
    #[derive(Content)]
    struct Post<'a> {
        title: &'a str,

        #[md]
        body: &'a str,
    }

    let tpl = Template::new("<h1>{{title}}</h1><div>{{body}}</div>").unwrap();

    let html = tpl.render(&Post {
        title: "This is *the* title",
        body: "This is *the* __body__!",
    });

    assert_eq!(html, "<h1>This is *the* title</h1><div><p>This is <em>the</em> <strong>body</strong>!</p>\n</div>");
}

#[test]
fn simple_partials() {
    let tpl = Template::from_file("templates/layout.html").unwrap();
    let html = tpl.render(&"");

    assert_eq!(html, "<head><h1>Head</h1></head>");
}

#[test]
fn simple_partials_folder() {
    use std::fs::read_to_string;

    let tpls = Templates::from_folder("templates").unwrap();
    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    assert_eq!(
        tpls.get("basic.html").unwrap().render(&post),
        read_to_string("templates/basic.result").unwrap().trim_end()
    );
    assert_eq!(
        tpls.get("another.html").unwrap().render(&post),
        read_to_string("templates/another.result")
            .unwrap()
            .trim_end()
    );
}

#[test]
fn illegal_partials() {
    use ramhorns::Error;

    let tpl1 = Template::new("<div>{{>templates/layout.html}}</div>");
    let tpl2 = Template::from_file("templates/illegal.hehe");

    if let Err(Error::PartialsDisabled) = tpl1 {
    } else {
        panic!("Partials loaded while parsing from &str");
    }

    if let Err(Error::IllegalPartial(name)) = tpl2 {
        assert_eq!(name, "../Cargo.toml".into());
    } else {
        panic!("Partials loaded out of the allowed directory");
    }
}
