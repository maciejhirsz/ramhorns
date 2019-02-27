<img src="ramhorns.svg" alt="Ramhorns logo" width="250" align="right">

# Ramhorns

Experimental [**Mustache**](https://mustache.github.io/) template engine implementation
in pure Rust.

**Ramhorns** loads and processes templates **at runtime**. It comes with a derive macro
which allows for templates to be rendered from native Rust data structures without doing
temporary allocations, intermediate `HashMap`s or what have you.

With a touch of magic 🎩, the power of friendship 🥂, and a sparkle of
[FNV hashing](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function)
✨, render times easily compete with static template engines like
[**Askama**](https://github.com/djc/askama).

What else do you want, a sticker?

### Cargo

```
[dependencies]
ramhorns = "0.1"
```

### Example

```rust
use ramhorns::{Template, Context};

#[derive(Context)]
struct Post<'a> {
    title: &'a str,
    teaser: &'a str,
}

#[derive(Context)]
struct Blog<'a> {
    title: String,        // Strings are cool
    posts: Vec<Post<'a>>, // &'a [Post<'a>] would work too
}

// Standard Mustache action here
let source ="<h1>{{title}}</h1>\
             {{#posts}}<article><h2>{{title}}</h2><p>{{teaser}}</p></article>{{/posts}}\
             {{^posts}}<p>No posts yet :(</p>{{/posts}}";

let tpl = Template::new(source);

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

### TODOS

+ [x] Rendering `&str` and `String` into `{{variables}}`.
+ [x] Unescaped printing with `{{{tripple-brace}}}`.
+ [x] Rendering sections `{{#foo}} ... {{/foo}}`.
+ [x] Rendering inverse sections `{{^foo}} ... {{/foo}}`.
+ [ ] Handle printing of all types, not just strings, via the `Display` trait.

### Benches

```
running 5 tests
test a_simple_ramhorns   ... bench:          97 ns/iter (+/- 4)
test b_simple_wearte     ... bench:          95 ns/iter (+/- 7)
test c_simple_askama     ... bench:         213 ns/iter (+/- 10)
test d_simple_mustache   ... bench:         842 ns/iter (+/- 41)
test e_simple_handlebars ... bench:       3,272 ns/iter (+/- 224)
```

Worth noting here is that both [**Askama**](https://github.com/djc/askama) and
[**wearte**](https://github.com/dgriffen/wearte) (a fork of a fork of **Askama**)
are processing templates at compile time and generate static rust code for rendering.
This is great for performance, but it also means you can't swap out templates without
recompiling your Rust binaries. In some cases, like for a static site generator, this
is unfortunately a deal breaker.

The [**Mustache** crate](https://github.com/nickel-org/rust-mustache) is the closest
thing to **Ramhorns** in design and feature set.

### License

Ramhorns is free software, and is released under the terms of the GNU General Public
License version 3. See [LICENSE](LICENSE).
