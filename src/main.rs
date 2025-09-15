use std::{collections::HashMap, fs, path::PathBuf};

use crate::lexer::Lexer;

mod lexer;

type Word = String;
type Rank = f32;

#[derive(Debug)]
struct Document {
    path: PathBuf,
    word_count: usize,
}

fn main() -> Result<(), std::io::Error> {
    let mut db = HashMap::<Word, Vec<Document>>::new();

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
            let chars = content.chars().collect::<Vec<char>>();
            let words = Lexer::new(&chars).collect::<Vec<String>>();
            let word_count = words.len();

            for word in words {
                let doc = Document {
                    path: file_path.clone(),
                    word_count,
                };
                match db.get_mut(&word) {
                    Some(docs) => {
                        let has_doc = docs.iter().any(|d| d.path == file_path);
                        if !has_doc {
                            docs.push(doc);
                        }
                    }
                    None => {
                        db.insert(word, vec![doc]);
                    }
                }
            }
        }
    }

    for (word, docs) in db.iter() {
        if word == "furniture" {
            println!("Word: {}, docs: {:?}", word, docs)
        }
    }

    Ok(())
}

fn search(query: &[char], db: &HashMap<Word, Vec<Document>>) -> Vec<(Document, Rank)> {
    let mut result = Vec::new();
    let query_words = Lexer::new(query).collect::<Vec<String>>();
    let total_docs = db
        .values()
        .fold(0f32, |a, b: &Vec<Document>| a + b.len() as f32);

    // for q_word in query_words.iter() {
    //     match db.g
    // }

    // for (word, docs) in db.iter() {
    //     let mut rank = 0f32;
    //     for query_word in &query_words {
    //         rank +=
    //             compute_tf(query_word, doc) * compute_idf(query_word, self.docs.len(), &self.df);
    //     }

    //     if !rank.is_nan() {
    //         result.push((path.clone(), rank));
    //     }
    // }
    result.sort_unstable_by(|(_, rank1): &(Document, f32), (_, rank2)| {
        rank1
            .partial_cmp(rank2)
            .expect(&format!("{rank1} and {rank2} are not comparable"))
    });
    result.reverse();
    result
}

fn compute_tf(word_freq: f32, word_count: f32) -> f32 {
    word_freq / word_count
}

fn compute_idf(total_docs: f32, word_count_docs: f32) -> f32 {
    (total_docs / word_count_docs).log10()
}
