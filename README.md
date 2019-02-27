<img src="ramhorns.svg" alt="Ramhorns logo" width="250" align="right">

# [WIP] Ramhorns

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

```rust
use ramhorns::{Template, Context};

#[derive(Context)]
struct Post<'a> {
    title: &'a str,
    body: &'a str,
}

let tpl = Template::new("<h1>{{title}}</h1><div>{{body}}</div>");

let rendered = tpl.render(&Post {
    title: "Hello Ramhorns",
    body: "Well, that was easy!",
});

assert_eq!(rendered, "<h1>Hello Ramhorns</h1><div>Well, that was easy!</div>")
```

## TODOS

+ [x] Parsing sections `{{#foo}} ... {{/foo}}`.
+ [x] Parsing inverse sections `{{^foo}} ... {{/foo}}`.
+ [ ] Rendering sections `{{#foo}} ... {{/foo}}`.
+ [ ] Rendering inverse sections `{{^foo}} ... {{/foo}}`.
+ [ ] Handle all types, not just strings, via the `Display` trait.

## Benches

```
running 5 tests
test a_simple_ramhorns   ... bench:          74 ns/iter (+/- 3)
test b_simple_wearte     ... bench:          77 ns/iter (+/- 5)
test c_simple_askama     ... bench:         194 ns/iter (+/- 26)
test d_simple_mustache   ... bench:         754 ns/iter (+/- 31)
test e_simple_handlebars ... bench:       3,073 ns/iter (+/- 212)
```

Worth noting here is that both [**Askama**](https://github.com/djc/askama) and
[**wearte**](https://github.com/dgriffen/wearte) (a fork of a fork of **Askama**)
are processing templates at compile time and generate static rust code for rendering.
This is great for performance, but it also means you can't swap out templates without
recompiling your Rust binaries. In some cases, like for a static site generator, this
is unfortunately a deal breaker.

The [**Mustache** crate](https://github.com/nickel-org/rust-mustache) is the closest
thing to **Ramhorns** in design and feature set.

## License

Ramhorns is free software, and is released under the terms of the GNU General Public
License version 3. See [LICENSE](LICENSE).
