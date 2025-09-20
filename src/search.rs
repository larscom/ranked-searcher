use std::collections::HashSet;

use crate::{
    DocumentIndex,
    document::{Document, Term},
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
            "\n{}\n-----------------------------",
            self.document.file_path().display()
        );

        // header
        println!("{:<15} | {:>10}", "Term", "Frequency");
        println!("{:-<15}-+-{:-<10}", "", "");

        // rows
        for s in self.stats.iter() {
            println!("{:<15} | {:>10}", s.query_term, s.term_freq);
        }
    }
}

#[allow(dead_code)]
pub struct Stat {
    pub query_term: Term,
    pub term_freq: f32,
    pub document_freq: f32,
}

pub struct RankedSearcher<'a> {
    document_index: &'a DocumentIndex,
}

impl<'a> RankedSearcher<'a> {
    pub fn new(document_index: &'a DocumentIndex) -> Self {
        Self { document_index }
    }

    pub fn search(&self, terms: &HashSet<Term>) -> Vec<Result<'_>> {
        let mut found_documents = HashSet::new();

        for query_term in terms.iter() {
            if let Some(documents) = self.document_index.documents(query_term) {
                for document in documents {
                    found_documents.insert(document);
                }
            }
        }

        self.generate_results(terms, &found_documents)
    }

    fn generate_results(
        &self,
        query_terms: &HashSet<Term>,
        found_documents: &HashSet<&'a Document>,
    ) -> Vec<Result<'_>> {
        let mut result = Vec::new();

        let total_documents = self.document_index.total_document_count();

        for document in found_documents {
            let mut rank = 0f32;
            let mut stats = Vec::new();

            let total_term_count = document.total_term_count();

            for query_term in query_terms.iter() {
                let document_freq = self.document_index.document_frequency(query_term);
                let term_freq = document.term_frequency(query_term);

                rank += self.calc_tf(total_term_count, term_freq)
                    * self.calc_idf(total_documents, document_freq);

                stats.push(Stat {
                    query_term: query_term.clone(),
                    term_freq,
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

    fn calc_tf(&self, total_term_count: usize, term_freq: f32) -> f32 {
        term_freq / total_term_count as f32
    }

    fn calc_idf(&self, total_documents: usize, document_freq: f32) -> f32 {
        (total_documents as f32 / document_freq).log10()
    }
}
