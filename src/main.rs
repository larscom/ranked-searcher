use crate::{
    document::{DocumentIndex, Term, TermCollector},
    search::RankedSearcher,
};
use colored::Colorize;
use std::{collections::HashSet, env, path::PathBuf, process};

mod document;
mod search;

fn main() {
    let mut args = env::args();
    let _ = args.next().expect("executable location should be present");

    let query = args.next().unwrap_or_else(|| {
        eprintln!("ERROR: terms are missing");
        eprintln!("usage: rs <terms> [dir]");
        eprintln!("example: rs 'hello world'");
        process::exit(1);
    });

    let work_dir = env::current_dir().unwrap_or_else(|_| {
        eprintln!("ERROR: could not get the current working directory");
        process::exit(1);
    });

    let dir_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| work_dir.clone());

    let mut document_index = DocumentIndex::new(work_dir);
    document_index.index_dir(&dir_path);

    let rs = RankedSearcher::new(&document_index);

    let query_terms = TermCollector::new(&query).collect::<HashSet<Term>>();
    let search_result = rs.search(&query_terms);

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

        if let Err(err) = result.document.print_highlighted_terms(&query_terms) {
            eprintln!("ERROR: could not highlight terms: {err}")
        }
    }
}
