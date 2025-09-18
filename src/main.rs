use crate::{
    document::{DocumentIndex, WordCollector},
    search::RankedSearcher,
};
use colored::Colorize;
use std::{env, path::PathBuf, process};

mod document;
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

    let query_words = WordCollector::new(&query).collect_unique();
    let search_result = rs.search(&query_words);

    for result in search_result {
        println!(
            "\n{}",
            result
                .document
                .file_path()
                .display()
                .to_string()
                .bright_green()
        );

        if let Err(err) = result.document.print_highlight_words(&query_words) {
            eprintln!("ERROR: could not highlight words: {err}")
        }
    }
}
