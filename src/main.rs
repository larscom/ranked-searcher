use crate::{document::DocumentIndex, lexer::Lexer, search::RankedSearcher};
use colored::Colorize;
use std::{collections::HashSet, env, path::PathBuf, process};

mod document;
mod lexer;
mod search;

fn main() {
    let mut args = env::args();
    let _ = args.next().expect("executable location should be present");

    let query = args.next().unwrap_or_else(|| {
        eprintln!("ERROR: query is missing");
        eprintln!("usage: rs <query> [dir]");
        process::exit(1);
    });

    let dir_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("failed to get current working dir"));

    let mut document_index = DocumentIndex::new();
    document_index.index_dir(&dir_path);

    let rs = RankedSearcher::new(&document_index);

    let query_chars = query.chars().collect::<Vec<_>>();
    let query_words = Lexer::new(&query_chars).collect::<HashSet<String>>();

    println!("{:?}", &query_words);

    let search_result = rs.search(&query_words);

    for result in search_result {
        println!(
            "{}",
            result
                .document
                .file_path()
                .display()
                .to_string()
                .bright_green()
        );
        result.document.highlight_words(&query_words).unwrap();
    }
}
