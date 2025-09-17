use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::lexer::Lexer;

pub type Word = String;

#[derive(Debug)]
pub struct Document {
    path: PathBuf,
    total_words: usize,
    word_freq: Rc<HashMap<Word, usize>>,
}

impl Document {
    pub fn new(path: PathBuf, total_words: usize, word_freq: Rc<HashMap<Word, usize>>) -> Self {
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

    pub fn index_dir(&mut self, dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let dir = fs::read_dir(dir_path)?;

        for file in dir {
            let file = file?;
            let file_path = file.path();
            let file_type = file.file_type()?;

            if file_type.is_dir() {
                self.index_dir(&file_path)?;
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
                self.total_documents += 1;
                self.update_index(content, file_path);
            }
        }

        Ok(())
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

    fn update_index(&mut self, content: String, file_path: PathBuf) {
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
            if let Some(count) = self.document_freq.get_mut(word) {
                *count += 1;
            } else {
                self.document_freq.insert(word.to_string(), 1);
            }
        }

        let word_freq = Rc::new(word_freq);

        for word in words {
            let path = file_path.clone();
            let document = Document::new(path, total_word_count, word_freq.clone());
            match self.document_db.get_mut(&word) {
                Some(docs) => {
                    docs.insert(document);
                }
                None => {
                    self.document_db.insert(word, HashSet::from([document]));
                }
            }
        }
    }
}
