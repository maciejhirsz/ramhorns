![Picture of a ram with horns](ram.jpg)

# [WIP] Ramhorns

Experimental [**`{{ mustache }}`**](https://mustache.github.io/)-ish implementation.

**Ramhorns** loads and processes templates **at runtime**. It comes with a derive
macro for structs which allows templates to be rendered from native Rust data
structures without doing temporary allocations, intermediate hashmaps and what
have you.

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

# Benches

```
running 5 tests
test a_simple_ramhorns   ... bench:          74 ns/iter (+/- 3)
test b_simple_wearte     ... bench:          77 ns/iter (+/- 5)
test c_simple_askama     ... bench:         194 ns/iter (+/- 26)
test d_simple_mustache   ... bench:         754 ns/iter (+/- 31)
test e_simple_handlebars ... bench:       3,073 ns/iter (+/- 212)
```

Worth noting here is that both [**Askama**](https://github.com/djc/askama) and
[**wearte**](https://github.com/dgriffen/wearte) (which, AFAIK, is a fork of Askama)
are processing templates at compile time and generate static rust code for rendering.
This is great for performance, but it also means you can't swap out templates without
recompiling your Rust binaries. In some cases, like for a static site generator, this
is unfortunately a deal breaker.

The [**Mustache** crate](https://github.com/nickel-org/rust-mustache) is the closest
thing to **Ramhorns** in design.
