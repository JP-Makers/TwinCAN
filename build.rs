
fn main() {
    slint_build::compile("ui/main.slint").unwrap();
	embed_resource::compile("app.rc", std::iter::empty::<&str>());
}
