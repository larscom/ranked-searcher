use std::{env, path::PathBuf, process};

use crate::{document::DocumentIndex, search::RankedSearcher};

mod document;
mod lexer;
mod search;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    document_index.index_dir(&dir_path)?;

    let rs = RankedSearcher::new(&document_index);

    let query_chars = query.chars().collect::<Vec<_>>();
    let search_result = rs.search(&query_chars);

    for result in search_result {
        result.print_stats();
    }

    Ok(())
}
