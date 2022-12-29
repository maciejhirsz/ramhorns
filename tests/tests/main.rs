use ramhorns::{Content, Ramhorns, Template};

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
fn can_render_nested_generic_types() {
    #[derive(Content)]
    struct Article {
        title: String,
        body: String,
    }

    #[derive(Content)]
    struct Page<T> {
        contents: T,
        last_updated: String,
    }

    let tpl = Template::new(
        "\
        {{#contents}}\
        <h1>{{title}}</h1>\
        <article>{{body}}</article>\
        {{/contents}}\
        <p>{{last_updated}}</p>",
    )
    .unwrap();

    let html = tpl.render(&Page {
        last_updated: "yesterday".into(),
        contents: Article {
            title: "Jam".into(),
            body: "This is an article body".into(),
        },
    });

    assert_eq!(
        html,
        "<h1>Jam</h1><article>This is an article body</article><p>yesterday</p>"
    )
}

#[test]
fn can_render_flattened_generic_types() {
    #[derive(Content)]
    struct Article {
        title: String,
        body: String,
    }

    #[derive(Content)]
    struct Page<T> {
        #[ramhorns(flatten)]
        contents: T,
        last_updated: String,
    }

    let tpl = Template::new(
        "<h1>{{title}}</h1>\
         <article>{{body}}</article>\
         <p>{{last_updated}}</p>",
    )
    .unwrap();

    let html = tpl.render(&Page {
        last_updated: "yesterday".into(),
        contents: Article {
            title: "Jam".into(),
            body: "This is an article body".into(),
        },
    });

    assert_eq!(
        html,
        "<h1>Jam</h1><article>This is an article body</article><p>yesterday</p>"
    )
}

#[test]
fn can_render_markdown() {
    #[derive(Content)]
    struct Post<'a> {
        title: &'a str,

        #[ramhorns(md)]
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
fn can_render_callback() {
    fn double<E>(s: &str, enc: &mut E) -> Result<(), E::Error>
    where
        E: ramhorns::encoding::Encoder,
    {
        enc.write_escaped(s)?;
        enc.write_escaped("+")?;
        enc.write_escaped(s)
    }

    #[derive(Content)]
    struct Post<'a> {
        #[ramhorns(callback = double)]
        body: &'a str,
    }

    let tpl = Template::new("<div>{{body}}</div>").unwrap();

    let html = tpl.render(&Post { body: "One" });

    assert_eq!(html, "<div>One+One</div>");
}

#[test]
fn can_reference_parent_content() {
    #[derive(Content)]
    struct Grandpa<'a> {
        name: &'a str,
        son: Father<'a>,
        hobbies: Vec<Hobby<'a>>,
    }

    #[derive(Content)]
    struct Father<'a> {
        title: &'a str,
        grandson: Man<'a>,
    }

    #[derive(Content)]
    struct Man<'a> {
        favourite_lang: &'a str,
    }

    #[derive(Content)]
    struct Hobby<'a> {
        hobby: &'a str,
    }

    let tpl = Template::new(
        "<h1>Grandpa</h1><p>He's got a son.{{#son}} He's got another son.{{#grandson}}
        His favourite language is {{favourite_lang}}. People call his father {{title}} and his grandfather {{name}}.
        Grandpa's hobbies are: <ul>{{#hobbies}}<li>{{hobby}}</li>{{/hobbies}}</ul>{{/grandson}}{{/son}}</p>").unwrap();

    let html = tpl.render(&Grandpa {
        name: "Jan",
        son: Father {
            title: "Sir",
            grandson: Man {
                favourite_lang: "Rust",
            },
        },
        hobbies: vec![
            Hobby {
                hobby: "watching ducks",
            },
            Hobby { hobby: "petang" },
        ],
    });

    assert_eq!(html, "<h1>Grandpa</h1><p>He\'s got a son. He\'s got another son.\n        His favourite language is Rust. People call his father Sir and his grandfather Jan.\n        Grandpa\'s hobbies are: <ul><li>watching ducks</li><li>petang</li></ul></p>");
}

#[test]
fn can_render_self_referencing_structures() {
    #[derive(Content)]
    struct Page<'a> {
        name: &'a str,
        subpages: &'a [Page<'a>],
    }

    let tpl = Template::new("{{name}}: {{#subpages}}{{name}}{{/subpages}}").unwrap();

    let rendered = tpl.render(&Page {
        name: "Hello",
        subpages: &[
            Page {
                name: "Foo",
                subpages: &[],
            },
            Page {
                name: "Bar",
                subpages: &[],
            },
        ],
    });

    assert_eq!(rendered, "Hello: FooBar");
}

#[test]
fn can_render_fields_from_parents() {
    #[derive(Content)]
    struct Father<'a> {
        father: &'a str,
        son: Son<'a>,
    }

    #[derive(Content)]
    struct Son<'a> {
        name: &'a str,
    }

    let tpl = Template::new("{{#son}}{{name}}'s father is {{father}}.{{/son}}").unwrap();

    let rendered = tpl.render(&Father {
        father: "Bob",
        son: Son { name: "Charlie" },
    });

    assert_eq!(rendered, "Charlie's father is Bob.");
}

#[test]
fn struct_with_many_types() {
    #[derive(Content)]
    struct Complicated<'a> {
        name: &'a str,
        size: u8,
        position: i16,
        tags: Vec<&'a str>,
        nickname: Option<&'a str>,
        children: &'a [Complicated<'a>],
    }

    let tpl = Template::new("This requires nothing but {{name}}.").unwrap();

    let rendered = tpl.render(&Complicated {
        name: "Name",
        size: 1,
        position: 2,
        tags: vec!["tag1", "tag2"],
        nickname: Some("nick"),
        children: &[Complicated {
            name: "Child",
            size: 0,
            position: 3,
            tags: Vec::with_capacity(0),
            nickname: None,
            children: &[],
        }],
    });

    assert_eq!(rendered, "This requires nothing but Name.");
}

#[test]
fn struct_with_many_sections() {
    #[derive(Content)]
    struct A(u8);
    #[derive(Content)]
    struct B(f64);
    #[derive(Content)]
    struct C(&'static str);
    #[derive(Content)]
    struct D(bool);

    #[derive(Content)]
    struct Page {
        a: A,
        b: B,
        c: C,
        d: D,
        other: &'static [Page],
    }

    let tpl = Template::new("<h1>{{#d}}{{#0}}{{#c}}{{0}}{{/c}}{{/0}}{{/d}} world!</h1>").unwrap();

    let rendered = tpl.render(&Page {
        a: A(1),
        b: B(2.0),
        c: C("Hello"),
        d: D(true),
        other: &[],
    });

    assert_eq!(rendered, "<h1>Hello world!</h1>");
}

#[test]
fn derive_attributes() {
    #[derive(Content)]
    struct Post<'a> {
        #[ramhorns(skip)]
        _title: &'a str,
        #[ramhorns(rename = "head")]
        body: &'a str,
    }

    let tpl = Template::new("<h1>{{_title}}</h1><head>{{head}}</head>").unwrap();

    let html = tpl.render(&Post {
        _title: "This is the title",
        body: "This is actually head!",
    });

    assert_eq!(html, "<h1></h1><head>This is actually head!</head>");

    #[derive(Content)]
    #[ramhorns(rename_all = "camelCase")]
    struct RenameAll<'a> {
        snake_name_one: &'a str,
        snake_name_two: &'a str,
    }

    let tpl = Template::new("{{snakeNameOne}}{{snakeNameTwo}}").unwrap();

    let render = tpl.render(&RenameAll {
        snake_name_one: "1",
        snake_name_two: "2",
    });

    assert_eq!(render, "12");
}

#[test]
fn derive_flatten() {
    #[derive(Content)]
    pub struct Parent<'a> {
        title: &'a str,
        #[ramhorns(flatten)]
        child: Child<'a>,
    }

    #[derive(Content)]
    pub struct Child<'a> {
        body: &'a str,
    }

    let tpl = Template::new("<h1>{{title}}</h1><head>{{body}}</head>").unwrap();

    let html = tpl.render(&Parent {
        title: "This is the title",
        child: Child {
            body: "This is the body",
        },
    });

    assert_eq!(
        html,
        "<h1>This is the title</h1><head>This is the body</head>"
    );
}

#[test]
fn simple_partials() {
    let mut tpls: Ramhorns = Ramhorns::lazy("templates").unwrap();
    let tpl = tpls.from_file("layout.html").unwrap();
    let html = tpl.render(&"");

    assert_eq!(html, "<head><h1>Head</h1></head>");
}

#[test]
fn simple_partials_folder() {
    use std::fs::read_to_string;

    let tpls: Ramhorns = Ramhorns::from_folder("templates").unwrap();
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
fn simple_partials_extend() {
    use std::fs::read_to_string;

    let mut tpls: Ramhorns = Ramhorns::from_folder("templates").unwrap();
    tpls.extend_from_folder("more_templates").unwrap();
    let post = Post {
        title: "Hello, Ramhorns!",
        body: "This is a really simple test of the rendering!",
    };

    assert_eq!(
        tpls.get("basic2.html").unwrap().render(&post),
        read_to_string("more_templates/basic2.result")
            .unwrap()
            .trim_end()
    );
}

#[test]
fn illegal_partials() {
    use ramhorns::Error;

    let mut tpls: Ramhorns = Ramhorns::lazy("templates").unwrap();

    let tpl1 = Template::new("<div>{{>templates/layout.html}}</div>");
    let tpl2 = tpls.from_file("illegal.hehe");

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
