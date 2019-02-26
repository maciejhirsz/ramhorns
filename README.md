![Picture of a ram with horns](ram.jpg)

# [WIP] Ramhorns

Experimental [`{{ mustache }}`](https://mustache.github.io/)-ish implementation.

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
test a_simple_ramhorns   ... bench:          69 ns/iter (+/- 5)
test b_simple_wearte     ... bench:          78 ns/iter (+/- 8)
test c_simple_askama     ... bench:         186 ns/iter (+/- 26)
test d_simple_mustache   ... bench:         716 ns/iter (+/- 45)
test e_simple_handlebars ... bench:       2,979 ns/iter (+/- 129)
```

Worth noting here is that both [**Askama**](https://github.com/djc/askama) and
[**wearte**](https://github.com/dgriffen/wearte) (which, AFAIK, is a fork of Askama)
are process templates on compile time and bundle them into the binary. This is great
for performance, but it also means you can't swap out templates without recompiling
your Rust binaries. For static site generators this is a no-go.

The [**mustache** crate](https://github.com/nickel-org/rust-mustache) is the closest
thing to **Ramhorns** in design.
