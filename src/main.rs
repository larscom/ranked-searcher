use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use crate::lexer::Lexer;

mod lexer;

type Word = String;
type Rank = f32;

#[derive(Debug)]
struct Document {
    path: PathBuf,
    total_word_count: usize,
    word_freq: Arc<HashMap<Word, usize>>,
}

fn main() -> Result<(), std::io::Error> {
    let mut document_db = HashMap::<Word, Vec<Document>>::new();
    let mut document_freq = HashMap::<Word, usize>::new();
    let mut total_documents = 0;

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

        let file_content = fs::read_to_string(&file_path);

        if let Ok(content) = file_content {
            total_documents += 1;

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

            let word_freq = Arc::new(word_freq);

            for word in words {
                let path = file_path.clone();
                let document = Document {
                    path,
                    total_word_count,
                    word_freq: word_freq.clone(),
                };
                match document_db.get_mut(&word) {
                    Some(docs) => {
                        let has_document = docs.iter().any(|d| d.path == file_path);
                        if !has_document {
                            docs.push(document);
                        }
                    }
                    None => {
                        document_db.insert(word, vec![document]);
                    }
                }
            }
        }
    }

    let query = "furniture";
    let query_chars = query.chars().collect::<Vec<_>>();

    let search_result = search(&query_chars, total_documents, &document_db, &document_freq);

    for (document, rank) in search_result {
        println!("({}) Path: {}", rank, document.path.display())
    }

    Ok(())
}

fn search<'a>(
    query: &[char],
    total_documents: usize,
    document_db: &'a HashMap<Word, Vec<Document>>,
    document_freq: &HashMap<Word, usize>,
) -> Vec<(&'a Document, Rank)> {
    let query_words = Lexer::new(query).collect::<Vec<String>>();

    let mut found_documents = Vec::new();

    for query_word in query_words.iter() {
        if let Some(documents) = document_db.get(query_word) {
            for document in documents {
                found_documents.push(document);
            }
        }
    }

    let mut result = Vec::new();

    for found_document in found_documents {
        let mut rank = 0f32;
        for query_word in query_words.iter() {
            rank += compute_tf(query_word, found_document)
                * compute_idf(query_word, total_documents, document_freq);
        }
        result.push((found_document, rank));
    }

    result.sort_unstable_by(|(_, rank1), (_, rank2)| {
        rank1.partial_cmp(rank2).expect("f32 should be comparable")
    });

    result.reverse();
    result
}

fn compute_tf(word: &str, document: &Document) -> f32 {
    let n = document.total_word_count as f32;
    let m = document.word_freq.get(word).cloned().unwrap_or(0) as f32;
    m / n
}

fn compute_idf(word: &str, total_documents: usize, document_freq: &HashMap<Word, usize>) -> f32 {
    let n = total_documents as f32;
    let m = document_freq.get(word).cloned().unwrap_or(1) as f32;
    (n / m).log10()
}
