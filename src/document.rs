use colored::Colorize;
use ignore::WalkBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use unicode_segmentation::{UnicodeSegmentation, UnicodeWords};

pub type Term = String;

#[derive(Debug)]
pub struct Document {
    path: PathBuf,
    total_terms: usize,
    term_freq: Arc<HashMap<Term, usize>>,
}

impl Document {
    pub fn new(path: PathBuf, total_terms: usize, term_freq: Arc<HashMap<Term, usize>>) -> Self {
        Self {
            path,
            total_terms,
            term_freq,
        }
    }

    pub fn print_highlighted_terms(&self, terms: &HashSet<Term>) -> Result<(), Box<dyn Error>> {
        let reader = BufReader::new(File::open(&self.path)?);
        let pattern = format!(
            "(?i)\\b({})\\b",
            terms
                .iter()
                .map(|w| regex::escape(w))
                .collect::<Vec<_>>()
                .join("|")
        );
        let re = Regex::new(&pattern)?;

        for (num, line) in reader.lines().enumerate() {
            let line = line?;
            if re.is_match(&line) {
                let highlighted = re.replace_all(&line, |caps: &regex::Captures| {
                    caps[0].bright_blue().bold().to_string()
                });
                println!("{}:{}", (num + 1).to_string().bright_yellow(), highlighted);
            }
        }

        Ok(())
    }

    pub fn term_frequency(&self, term: &Term) -> f32 {
        self.term_freq.get(term).cloned().unwrap_or(0) as f32
    }

    pub fn total_term_count(&self) -> usize {
        self.total_terms
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
    work_dir: PathBuf,
    total_documents: usize,
    document_db: HashMap<Term, HashSet<Document>>,
    document_freq: HashMap<Term, usize>,
}

impl DocumentIndex {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            work_dir,
            ..Default::default()
        }
    }

    pub fn index_dir(&mut self, path: &Path) {
        let document_db = Mutex::new(HashMap::<Term, HashSet<Document>>::new());
        let document_freq = Mutex::new(HashMap::<Term, usize>::new());
        let total_documents = AtomicUsize::new(0);

        WalkBuilder::new(path)
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

    pub fn documents(&self, term: &Term) -> Option<&HashSet<Document>> {
        self.document_db.get(term)
    }

    pub fn document_frequency(&self, term: &Term) -> f32 {
        self.document_freq.get(term).cloned().unwrap_or(1) as f32
    }

    pub fn total_document_count(&self) -> usize {
        self.total_documents
    }

    fn update_index(
        &self,
        content: String,
        path: PathBuf,
        document_db: &Mutex<HashMap<Term, HashSet<Document>>>,
        document_freq: &Mutex<HashMap<Term, usize>>,
    ) {
        let terms = TermCollector::new(&content).collect::<Vec<Term>>();
        let mut term_freq = HashMap::new();

        let mut total_term_count = 0;

        for term in terms.iter() {
            if let Some(count) = term_freq.get_mut(term) {
                *count += 1;
            } else {
                term_freq.insert(term.to_string(), 1);
            }
            total_term_count += 1;
        }

        match document_freq.lock() {
            Ok(mut document_freq) => {
                for term in term_freq.keys() {
                    if let Some(count) = document_freq.get_mut(term) {
                        *count += 1;
                    } else {
                        document_freq.insert(term.to_string(), 1);
                    }
                }
            }
            Err(err) => {
                eprintln!("ERROR: could not achieve lock on document_freq mutex: {err}")
            }
        }

        let term_freq = Arc::new(term_freq);

        match document_db.lock() {
            Ok(mut document_db) => {
                for term in terms {
                    let path = path.clone();
                    let relative_path = path.strip_prefix(&self.work_dir).unwrap_or(&path);
                    let document = Document::new(
                        relative_path.to_path_buf(),
                        total_term_count,
                        term_freq.clone(),
                    );
                    match document_db.get_mut(&term) {
                        Some(docs) => {
                            docs.insert(document);
                        }
                        None => {
                            document_db.insert(term, HashSet::from([document]));
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

pub struct TermCollector<'a> {
    iter: UnicodeWords<'a>,
}

impl<'a> TermCollector<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            iter: content.unicode_words(),
        }
    }

    pub fn next_term(&mut self) -> Option<Term> {
        self.iter.next().map(|v| v.to_string().to_ascii_lowercase())
    }
}

impl<'a> Iterator for TermCollector<'a> {
    type Item = Term;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_term()
    }
}
