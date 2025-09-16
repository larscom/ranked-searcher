use std::{collections::HashMap, path::PathBuf, rc::Rc};

pub type Word = String;

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub total_word_count: usize,
    pub word_freq: Rc<HashMap<Word, usize>>,
}

impl Document {
    pub fn new(
        path: PathBuf,
        total_word_count: usize,
        word_freq: Rc<HashMap<Word, usize>>,
    ) -> Self {
        Self {
            path,
            total_word_count,
            word_freq,
        }
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
