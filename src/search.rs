use std::collections::HashSet;

use crate::{
    DocumentIndex,
    document::{Document, Word},
};

pub type Rank = f32;

#[allow(dead_code)]
pub struct Result<'a> {
    pub rank: Rank,
    pub total_documents: usize,
    pub document: &'a Document,
    pub stats: Vec<Stat>,
}

#[allow(dead_code)]
impl<'a> Result<'a> {
    pub fn print_stats(&self) {
        println!(
            "\nFile: {}\n-----------------------------",
            self.document.file_path().display()
        );

        // header
        println!("{:<15} | {:>10}", "Word", "Frequency");
        println!("{:-<15}-+-{:-<10}", "", "");

        // rows
        for s in self.stats.iter() {
            println!("{:<15} | {:>10}", s.query_word, s.word_freq);
        }
    }
}

#[allow(dead_code)]
pub struct Stat {
    pub query_word: Word,
    pub word_freq: f32,
    pub document_freq: f32,
}

pub struct RankedSearcher<'a> {
    document_index: &'a DocumentIndex,
}

impl<'a> RankedSearcher<'a> {
    pub fn new(document_index: &'a DocumentIndex) -> Self {
        Self { document_index }
    }

    pub fn search(&self, words: &HashSet<String>) -> Vec<Result<'_>> {
        let mut found_documents = HashSet::new();

        for query_word in words.iter() {
            if let Some(documents) = self.document_index.documents(query_word) {
                for document in documents {
                    found_documents.insert(document);
                }
            }
        }

        self.generate_results(words, &found_documents)
    }

    fn generate_results(
        &self,
        query_words: &HashSet<String>,
        found_documents: &HashSet<&'a Document>,
    ) -> Vec<Result<'_>> {
        let mut result = Vec::new();

        let total_documents = self.document_index.total_document_count();

        for document in found_documents {
            let mut rank = 0f32;
            let mut stats = Vec::new();

            let total_word_count = document.total_word_count();

            for query_word in query_words.iter() {
                let document_freq = self.document_index.document_frequency(query_word);
                let word_freq = document.word_frequency(query_word);

                rank += self.calc_tf(total_word_count, word_freq)
                    * self.calc_idf(total_documents, document_freq);

                stats.push(Stat {
                    query_word: query_word.clone(),
                    word_freq,
                    document_freq,
                });
            }

            result.push(Result {
                rank,
                total_documents,
                document,
                stats,
            });
        }

        result.sort_unstable_by(|a, b| {
            a.rank
                .partial_cmp(&b.rank)
                .expect("f32 should be comparable")
        });

        result.reverse();
        result
    }

    fn calc_tf(&self, total_word_count: usize, word_freq: f32) -> f32 {
        word_freq / total_word_count as f32
    }

    fn calc_idf(&self, total_documents: usize, document_freq: f32) -> f32 {
        (total_documents as f32 / document_freq).log10()
    }
}
