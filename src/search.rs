use std::collections::HashSet;

use crate::{
    DocumentIndex,
    document::{self},
    lexer::Lexer,
};

pub type Rank = f32;

pub struct RankedSearch<'a> {
    document_index: &'a DocumentIndex,
}

impl<'a> RankedSearch<'a> {
    pub fn new(document_index: &'a DocumentIndex) -> Self {
        Self { document_index }
    }

    pub fn search(&self, query: &[char]) -> Vec<(&document::Document, Rank)> {
        let query_words = Lexer::new(query).collect::<HashSet<String>>();

        let mut found_documents = HashSet::new();

        for query_word in query_words.iter() {
            if let Some(documents) = self.document_index.documents(query_word) {
                for document in documents {
                    found_documents.insert(document);
                }
            }
        }

        let mut result = Vec::new();

        let total_documents = self.document_index.total_documents_count();
        for found_document in found_documents {
            let mut rank = 0f32;

            let total_word_count = found_document.get_total_word_count();

            for query_word in query_words.iter() {
                let document_freq = self
                    .document_index
                    .document_frequency(query_word)
                    .unwrap_or(1) as f32;

                let word_freq = found_document.get_word_freq(query_word).unwrap_or(0) as f32;

                rank += self.compute_tf(total_word_count as f32, word_freq)
                    * self.compute_idf(total_documents as f32, document_freq);
            }

            result.push((found_document, rank));
        }

        result.sort_unstable_by(|(_, rank1), (_, rank2)| {
            rank1.partial_cmp(rank2).expect("f32 should be comparable")
        });

        result.reverse();
        result
    }

    fn compute_tf(&self, total_word_count: f32, word_freq: f32) -> f32 {
        word_freq / total_word_count
    }

    fn compute_idf(&self, total_documents: f32, document_freq: f32) -> f32 {
        (total_documents / document_freq).log10()
    }
}
