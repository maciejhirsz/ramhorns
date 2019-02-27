use ramhorns::{Template, Context};

#[derive(Context)]
struct Post<'a> {
    title: &'a str,
    body: &'a str,
}

#[test]
fn simple_render() {
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
    #[derive(Context)]
    struct Dummy {
        dummy: &'static str
    }

    let tpl = Template::new("Escaped: {{dummy}} Unescaped: {{{dummy}}}");

    let rendered = tpl.render(&Dummy {
        dummy: "This is a <strong>test</strong>!",
    });

    assert_eq!(rendered, "Escaped: This is a &lt;strong&gt;test&lt;/strong&gt;! \
                          Unescaped: This is a <strong>test</strong>!");
}

#[test]
fn handles_tuple_structs() {
    #[derive(Context)]
    struct Dummy(&'static str, &'static str, &'static str);

    let tpl = Template::new("{{1}} {{2}} {{0}}");

    let rendered = tpl.render(&Dummy("zero", "one", "two"));

    assert_eq!(rendered, "one two zero");
}


#[test]
fn handles_unit_structs() {
    #[derive(Context)]
    struct Dummy;

    let tpl = Template::new("This is pretty silly, but why not?");

    let rendered = tpl.render(&Dummy);

    assert_eq!(rendered, "This is pretty silly, but why not?");
}

#[test]
fn simple_render_with_strings() {
    #[derive(Context)]
    struct Post {
        title: String,
        body: String,
    }

    let source = "<title>{{title}}</title><h1>{{ title }}</h1><div>{{body}}</div>";
    let tpl = Template::new(source);

    let rendered = tpl.render(&Post {
        title: "Hello, Ramhorns!".to_string(),
        body: "This is a really simple test of the rendering!".to_string(),
    });

    assert_eq!(&rendered, "<title>Hello, Ramhorns!</title><h1>Hello, Ramhorns!</h1>\
                           <div>This is a really simple test of the rendering!</div>");
}

#[test]
fn can_render_sections_from_bool() {
    #[derive(Context)]
    struct Conditional {
        secret: bool,
    }

    let tpl = Template::new("Hello!{{#secret}} This is a secret!{{/secret}}");

    let show = tpl.render(&Conditional {
        secret: true,
    });
    let hide = tpl.render(&Conditional {
        secret: false,
    });

    assert_eq!(show, "Hello! This is a secret!");
    assert_eq!(hide, "Hello!");
}

#[test]
fn can_render_inverse_sections_from_bool() {
    #[derive(Context)]
    struct Conditional {
        secret: bool,
    }

    let tpl = Template::new("Hello!{{^secret}} This is NOT a secret!{{/secret}}");

    let show = tpl.render(&Conditional {
        secret: true,
    });
    let hide = tpl.render(&Conditional {
        secret: false,
    });

    assert_eq!(show, "Hello!");
    assert_eq!(hide, "Hello! This is NOT a secret!");
}

#[test]
fn can_render_inverse_sections_for_empty_strs() {
    #[derive(Context)]
    struct Person<'a> {
        name: &'a str,
    }

    let tpl = Template::new("Hello {{name}}{{^name}}Anonymous{{/name}}!");

    let named = tpl.render(&Person {
        name: "Maciej",
    });
    let unnamed = tpl.render(&Person {
        name: "",
    });

    assert_eq!(named, "Hello Maciej!");
    assert_eq!(unnamed, "Hello Anonymous!");
}

#[test]
fn can_render_lists_from_slices() {
    #[derive(Context)]
    struct Article<'a> {
        title: &'a str,
    }

    #[derive(Context)]
    struct Page<'a> {
        title: &'a str,
        articles: &'a [Article<'a>]
    }

    let tpl = Template::new("<h1>{{title}}</h1>\
                             {{#articles}}<article>{{title}}</article>{{/articles}}\
                             {{^articles}}<p>No articles :(</p>{{/articles}}");


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

    assert_eq!(blog, "<h1>Awesome Blog</h1>\
                      <article>How is Ramhorns this fast?</article>\
                      <article>Look at that cat pic!</article>\
                      <article>Hello World!</article>");

    assert_eq!(empty, "<h1>Sad page :(</h1>\
                       <p>No articles :(</p>");
}

#[test]
fn can_render_lists_from_vecs() {
    #[derive(Context)]
    struct Article {
        title: String,
    }

    #[derive(Context)]
    struct Page {
        title: String,
        articles: Vec<Article>,
    }

    let tpl = Template::new("<h1>{{title}}</h1>\
                             {{#articles}}<article>{{title}}</article>{{/articles}}\
                             {{^articles}}<p>No articles :(</p>{{/articles}}");


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

    assert_eq!(blog, "<h1>Awesome Blog</h1>\
                      <article>How is Ramhorns this fast?</article>\
                      <article>Look at that cat pic!</article>\
                      <article>Hello World!</article>");

    assert_eq!(empty, "<h1>Sad page :(</h1>\
                       <p>No articles :(</p>");
}
