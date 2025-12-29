/// Custom builder to embed Rust data type templates in the final executable
fn main() {
    minijinja_embed::embed_templates!("templates");
}
