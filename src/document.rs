use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use rayon::iter::{ParallelBridge, ParallelIterator};
use walkdir::WalkDir;

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
        let document_db: HashMap<Word, HashSet<Document>> = HashMap::new();
        let document_db = Mutex::new(document_db);

        let document_freq: HashMap<Word, usize> = HashMap::new();
        let document_freq = Mutex::new(document_freq);

        let total_documents = AtomicUsize::new(0);

        WalkDir::new(dir_path)
            .into_iter()
            .filter_entry(|e| {
                let is_node_modules = e
                    .file_name()
                    .to_str()
                    .map(|s| s == "node_modules")
                    .unwrap_or(false);

                !is_node_modules
            })
            .filter_map(Result::ok)
            .filter(|e| {
                let is_file = e.file_type().is_file();
                let is_dot_file = e
                    .file_name()
                    .to_str()
                    .map(|s| s.starts_with("."))
                    .unwrap_or(false);

                is_file && !is_dot_file
            })
            .par_bridge()
            .filter_map(|entry| {
                let path = entry.path().to_path_buf();
                fs::read_to_string(&path)
                    .ok()
                    .map(|content| (content, path))
            })
            .for_each(|(content, path)| {
                total_documents.fetch_add(1, Ordering::Relaxed);

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

                let mut document_freq = document_freq.lock().unwrap();
                for word in word_freq.keys() {
                    if let Some(count) = document_freq.get_mut(word) {
                        *count += 1;
                    } else {
                        document_freq.insert(word.to_string(), 1);
                    }
                }

                let word_freq = Arc::new(word_freq);

                let mut document_db = document_db.lock().unwrap();
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
            });

        self.total_documents = total_documents.load(Ordering::Relaxed);
        self.document_db = document_db.into_inner().unwrap();
        self.document_freq = document_freq.into_inner().unwrap();
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
}
