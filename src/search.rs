use std::collections::{HashMap, HashSet};

use crate::{document, lexer::Lexer};

pub type Rank = f32;

pub struct RankedSearch<'a> {
    total_documents: usize,
    document_db: &'a HashMap<document::Word, HashSet<document::Document>>,
    document_freq: &'a HashMap<document::Word, usize>,
}

impl<'a> RankedSearch<'a> {
    pub fn new(
        total_documents: usize,
        document_db: &'a HashMap<document::Word, HashSet<document::Document>>,
        document_freq: &'a HashMap<document::Word, usize>,
    ) -> Self {
        Self {
            total_documents,
            document_db,
            document_freq,
        }
    }

    pub fn search(&self, query: &[char]) -> Vec<(&document::Document, Rank)> {
        let query_words = Lexer::new(query).collect::<HashSet<String>>();

        let mut found_documents = HashSet::new();

        for query_word in query_words.iter() {
            if let Some(documents) = self.document_db.get(query_word) {
                for document in documents {
                    found_documents.insert(document);
                }
            }
        }

        let mut result = Vec::new();

        for found_document in found_documents {
            let mut rank = 0f32;
            for query_word in query_words.iter() {
                rank += self.compute_tf(query_word, found_document) * self.compute_idf(query_word);
            }

            result.push((found_document, rank));
        }

        result.sort_unstable_by(|(_, rank1), (_, rank2)| {
            rank1.partial_cmp(rank2).expect("f32 should be comparable")
        });

        result.reverse();
        result
    }

    fn compute_tf(&self, word: &str, document: &document::Document) -> f32 {
        let n = document.total_word_count as f32;
        let m = document.word_freq.get(word).cloned().unwrap_or(0) as f32;
        m / n
    }

    fn compute_idf(&self, word: &str) -> f32 {
        let n = self.total_documents as f32;
        let m = self.document_freq.get(word).cloned().unwrap_or(1) as f32;
        (n / m).log10()
    }
}
