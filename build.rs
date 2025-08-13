use std::iter;

fn main() {
    slint_build::compile("ui/main.slint").unwrap();
	let _ = embed_resource::compile("app.rc", iter::empty::<&str>());
}
