use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process,
    rc::Rc,
};

use crate::lexer::Lexer;

mod document;
mod lexer;
mod search;

fn process_dir(
    dir_path: &Path,
    total_documents: &mut usize,
    document_db: &mut HashMap<document::Word, HashSet<document::Document>>,
    document_freq: &mut HashMap<document::Word, usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = fs::read_dir(dir_path)?;

    for file in dir {
        let file = file?;
        let file_path = file.path();
        let file_type = file.file_type()?;

        if file_type.is_dir() {
            process_dir(&file_path, total_documents, document_db, document_freq)?;
            continue;
        };

        let dot_file = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("."))
            .unwrap_or(false);

        if dot_file {
            continue;
        }

        let file_content = fs::read_to_string(&file_path);

        if let Ok(content) = file_content {
            *total_documents += 1;

            let chars = content.chars().collect::<Vec<char>>();
            let words = Lexer::new(&chars).collect::<Vec<String>>();
            let mut word_freq = HashMap::new();

            let mut total_word_count = 0;

            for word in words.iter() {
                if let Some(count) = word_freq.get_mut(word) {
                    *count += 1;
                } else {
                    word_freq.insert(word.to_string(), 1);
                }
                total_word_count += 1;
            }

            for word in word_freq.keys() {
                if let Some(count) = document_freq.get_mut(word) {
                    *count += 1;
                } else {
                    document_freq.insert(word.to_string(), 1);
                }
            }

            let word_freq = Rc::new(word_freq);

            for word in words {
                let path = file_path.clone();
                let document = document::Document::new(path, total_word_count, word_freq.clone());
                match document_db.get_mut(&word) {
                    Some(docs) => {
                        docs.insert(document);
                    }
                    None => {
                        document_db.insert(word, HashSet::from([document]));
                    }
                }
            }
        }
    }

    Ok(())
}

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

    let mut document_db = HashMap::<document::Word, HashSet<document::Document>>::new();
    let mut document_freq = HashMap::<document::Word, usize>::new();
    let mut total_documents = 0;

    process_dir(
        &dir_path,
        &mut total_documents,
        &mut document_db,
        &mut document_freq,
    )?;

    let rs = search::RankedSearch::new(total_documents, &document_db, &document_freq);

    let query_chars = query.chars().collect::<Vec<_>>();
    let search_result = rs.search(&query_chars);

    for (document, rank) in search_result.into_iter().take(25) {
        println!("({}) Path: {}", rank, document.path.display())
    }

    Ok(())
}
