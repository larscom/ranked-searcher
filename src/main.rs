use crate::lexer::Lexer;

mod lexer;

fn main() {
    let content = "Hello world! This is clean 123 content.";
    let content_chars = content.chars().collect::<Vec<char>>();

    for t in Lexer::new(&content_chars) {
        println!("content={:?}", t)
    }
}
