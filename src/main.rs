use std::{collections::HashSet, fs, path::PathBuf};

use crate::lexer::Lexer;

mod lexer;

struct Document {
    path: PathBuf,
    word_count: usize,
}

fn main() -> Result<(), std::io::Error> {
    let mut unique_words = HashSet::new();

    let dir = fs::read_dir("./test_data")?;

    for file in dir {
        let file_path = file?.path();

        let dot_file = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("."))
            .unwrap_or(false);

        if dot_file {
            continue;
        }

        let file_content = fs::read_to_string(file_path);

        if let Ok(content) = file_content {
            let chars = content.chars().collect::<Vec<char>>();

            for t in Lexer::new(&chars) {
                unique_words.insert(t);
            }
        }
    }

    println!("unqiue words: {}", unique_words.len());

    Ok(())
}

fn compute_tf(word_freq: f32, word_count: f32) -> f32 {
    word_freq / word_count
}

fn compute_idf(total_docs: f32, word_count_docs: f32) -> f32 {
    (total_docs / word_count_docs).log10()
}
