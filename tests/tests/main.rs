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
						  Unescaped: This is a <strong>test</strong>!")
}

#[test]
fn handles_tuple_structs() {
	#[derive(Context)]
	struct Dummy(&'static str, &'static str, &'static str);

	let tpl = Template::new("{{1}} {{2}} {{0}}");

	let rendered = tpl.render(&Dummy("zero", "one", "two"));

	assert_eq!(rendered, "one two zero")
}


#[test]
fn handles_unit_structs() {
	#[derive(Context)]
	struct Dummy;

	let tpl = Template::new("This is pretty silly, but why not?");

	let rendered = tpl.render(&Dummy);

	assert_eq!(rendered, "This is pretty silly, but why not?")
}
