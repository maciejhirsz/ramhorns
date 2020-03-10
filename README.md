<img src="https://raw.githubusercontent.com/maciejhirsz/ramhorns/master/ramhorns.svg?sanitize=true" alt="Ramhorns logo" width="250" align="right">

# Ramhorns

[![Travis shield](https://travis-ci.org/maciejhirsz/ramhorns.svg)](https://travis-ci.org/maciejhirsz/ramhorns)
[![Crates.io version shield](https://img.shields.io/crates/v/ramhorns.svg)](https://crates.io/crates/ramhorns)
[![Crates.io license shield](https://img.shields.io/crates/l/ramhorns.svg)](https://crates.io/crates/ramhorns)

Fastest [**Mustache**](https://mustache.github.io/) template engine implementation
in pure Rust.

**Ramhorns** loads and processes templates **at runtime**. It comes with a derive macro
which allows for templates to be rendered from native Rust data structures without doing
temporary allocations, intermediate `HashMap`s or what have you.

With a touch of magic 🎩, the power of friendship 🥂, and a sparkle of
[FNV hashing](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function)
✨, render times easily compete with static template engines like
[**Askama**](https://github.com/djc/askama).

What else do you want, a sticker?

### Cargo.toml

```toml
[dependencies]
ramhorns = "0.5"
```

### Example

```rust
use ramhorns::{Template, Content};

#[derive(Content)]
struct Post<'a> {
    title: &'a str,
    teaser: &'a str,
}

#[derive(Content)]
struct Blog<'a> {
    title: String,        // Strings are cool
    posts: Vec<Post<'a>>, // &'a [Post<'a>] would work too
}

// Standard Mustache action here
let source = "<h1>{{title}}</h1>\
              {{#posts}}<article><h2>{{title}}</h2><p>{{teaser}}</p></article>{{/posts}}\
              {{^posts}}<p>No posts yet :(</p>{{/posts}}";

let tpl = Template::new(source).unwrap();

let rendered = tpl.render(&Blog {
    title: "My Awesome Blog!".to_string(),
    posts: vec![
        Post {
            title: "How I tried Ramhorns and found love 💖",
            teaser: "This can happen to you too",
        },
        Post {
            title: "Rust is kinda awesome",
            teaser: "Yes, even the borrow checker! 🦀",
        },
    ]
});

assert_eq!(rendered, "<h1>My Awesome Blog!</h1>\
                      <article>\
                          <h2>How I tried Ramhorns and found love 💖</h2>\
                          <p>This can happen to you too</p>\
                      </article>\
                      <article>\
                          <h2>Rust is kinda awesome</h2>\
                          <p>Yes, even the borrow checker! 🦀</p>\
                      </article>");
```

### Features so far

+ Rendering common types, such as `&str`, `String`, `bool`s, and numbers into `{{variables}}`.
+ Unescaped printing with `{{{tripple-brace}}}` or `{{&ampersant}}`.
+ Rendering sections `{{#foo}} ... {{/foo}}`.
+ Rendering inverse sections `{{^foo}} ... {{/foo}}`.
+ Rendering partials `{{>file.html}}`.
+ Zero-copy [CommonMark](https://commonmark.org/) rendering from fields marked with `#[md]`.

### Benches

```
test a_simple_ramhorns      ... bench:          84 ns/iter (+/- 2)
test b_simple_askama        ... bench:         193 ns/iter (+/- 5)
test c_simple_tera          ... bench:         448 ns/iter (+/- 13)
test d_simple_mustache      ... bench:         713 ns/iter (+/- 33)
test e_simple_handlebars    ... bench:       1,015 ns/iter (+/- 33)
```

Worth noting here is that [**Askama**](https://github.com/djc/askama) is processing
templates at compile time and generate static rust code for rendering. This is great
for performance, but it also means you can't swap out templates without recompiling
your Rust binaries. In some cases, like for a static site generator, this is
unfortunately a deal breaker.

The [**Mustache** crate](https://github.com/nickel-org/rust-mustache) is the closest
thing to **Ramhorns** in design and feature set.

### License

Ramhorns is free software, and is released under the terms of the GNU General Public
License version 3. See [LICENSE](LICENSE).
