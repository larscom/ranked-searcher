use std::{collections::HashSet, fs};

use crate::lexer::Lexer;

mod lexer;

fn main() -> Result<(), std::io::Error> {
    let mut unique_words = HashSet::new();

    let content = fs::read_to_string("./pg84.txt")?;
    // let content = "Hello World, Chapter 24";

    let content_chars = content.chars().collect::<Vec<char>>();

    for t in Lexer::new(&content_chars) {
        println!("content={:?}", t);
        unique_words.insert(t);
    }

    println!("unique words: {}", unique_words.len());

    Ok(())
}
