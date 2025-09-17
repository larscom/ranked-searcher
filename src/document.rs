use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use ignore::WalkBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::lexer::Lexer;

pub type Word = String;

#[derive(Debug)]
pub struct Document {
    path: PathBuf,
    total_words: usize,
    word_freq: Arc<HashMap<Word, usize>>,
}

impl Document {
    pub fn new(path: PathBuf, total_words: usize, word_freq: Arc<HashMap<Word, usize>>) -> Self {
        Self {
            path,
            total_words,
            word_freq,
        }
    }

    pub fn word_frequency(&self, word: &Word) -> f32 {
        self.word_freq.get(word).cloned().unwrap_or(0) as f32
    }

    pub fn total_word_count(&self) -> usize {
        self.total_words
    }

    pub fn file_path(&self) -> &Path {
        &self.path
    }
}

impl std::cmp::PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::cmp::Eq for Document {}

impl std::hash::Hash for Document {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Default)]
pub struct DocumentIndex {
    total_documents: usize,
    document_db: HashMap<Word, HashSet<Document>>,
    document_freq: HashMap<Word, usize>,
}

impl DocumentIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index_dir(&mut self, dir_path: &Path) {
        let document_db = Mutex::new(HashMap::<Word, HashSet<Document>>::new());
        let document_freq = Mutex::new(HashMap::<Word, usize>::new());
        let total_documents = AtomicUsize::new(0);

        WalkBuilder::new(dir_path)
            .build()
            .filter_map(Result::ok)
            .par_bridge()
            .filter_map(|entry| {
                let path = entry.path().to_path_buf();
                fs::read_to_string(&path)
                    .ok()
                    .map(|content| (content, path))
            })
            .for_each(|(content, path)| {
                total_documents.fetch_add(1, Ordering::Relaxed);
                self.update_index(content, path, &document_db, &document_freq);
            });

        match document_db.into_inner() {
            Ok(document_db) => self.document_db = document_db,
            Err(err) => eprintln!("ERROR: could not consume document_db mutex: {err}"),
        }
        match document_freq.into_inner() {
            Ok(document_freq) => self.document_freq = document_freq,
            Err(err) => eprintln!("ERROR: could not consume document_freq mutex: {err}"),
        }
        self.total_documents = total_documents.load(Ordering::Relaxed);
    }

    pub fn documents(&self, word: &Word) -> Option<&HashSet<Document>> {
        self.document_db.get(word)
    }

    pub fn document_frequency(&self, word: &Word) -> f32 {
        self.document_freq.get(word).cloned().unwrap_or(1) as f32
    }

    pub fn total_document_count(&self) -> usize {
        self.total_documents
    }

    fn update_index(
        &self,
        content: String,
        path: PathBuf,
        document_db: &Mutex<HashMap<Word, HashSet<Document>>>,
        document_freq: &Mutex<HashMap<Word, usize>>,
    ) {
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

        match document_freq.lock() {
            Ok(mut document_freq) => {
                for word in word_freq.keys() {
                    if let Some(count) = document_freq.get_mut(word) {
                        *count += 1;
                    } else {
                        document_freq.insert(word.to_string(), 1);
                    }
                }
            }
            Err(err) => {
                eprintln!("ERROR: could not achieve lock on document_freq mutex: {err}")
            }
        }

        let word_freq = Arc::new(word_freq);

        match document_db.lock() {
            Ok(mut document_db) => {
                for word in words {
                    let path = path.clone();
                    let document = Document::new(path, total_word_count, word_freq.clone());
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
            Err(err) => {
                eprintln!("ERROR: could not achieve lock on document_db mutex: {err}")
            }
        }
    }
}
