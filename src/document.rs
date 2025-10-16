use colored::Colorize;
use ignore::WalkBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use unicode_segmentation::{UnicodeSegmentation, UnicodeWords};
use zip::ZipArchive;

pub type Term = String;

#[derive(Debug)]
pub struct Document {
    parser: fn(&Path) -> Option<String>,
    path: PathBuf,
    total_terms: usize,
    term_freq: Arc<HashMap<Term, usize>>,
}

impl Document {
    pub fn new(
        parser: fn(&Path) -> Option<String>,
        path: PathBuf,
        total_terms: usize,
        term_freq: Arc<HashMap<Term, usize>>,
    ) -> Self {
        Self {
            parser,
            path,
            total_terms,
            term_freq,
        }
    }

    pub fn print_highlighted_terms(&self, terms: &HashSet<Term>) -> Result<(), Box<dyn Error>> {
        if let Some(content) = (self.parser)(&self.path) {
            let pattern = format!(
                "(?i)\\b({})\\b",
                terms
                    .iter()
                    .map(|w| regex::escape(w))
                    .collect::<Vec<_>>()
                    .join("|")
            );
            let re = Regex::new(&pattern)?;

            for (num, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    let highlighted = re.replace_all(line, |caps: &regex::Captures| {
                        caps[0].bright_blue().bold().to_string()
                    });
                    println!("{}:{}", (num + 1).to_string().bright_yellow(), highlighted);
                }
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

fn read_text_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

// TODO: extract text from pdf
fn read_pdf_file(_path: &Path) -> Option<String> {
    None
}

fn read_docx_file(path: &Path) -> Option<String> {
    File::open(path)
        .ok()
        .map(BufReader::new)
        .and_then(|reader| ZipArchive::new(reader).ok())
        .as_mut()
        .map(|archive| {
            let mut content = String::new();
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).expect("file by index");
                let name = file.name().to_string();

                if name.starts_with("word/") && name.ends_with(".xml") {
                    let mut xml = String::new();
                    file.read_to_string(&mut xml)
                        .expect("file should be valid UTF-8");
                    if let Ok(doc) = roxmltree::Document::parse(&xml) {
                        for text in doc
                            .descendants()
                            .filter(|n| n.has_tag_name("t"))
                            .filter_map(|n| n.text())
                        {
                            content.push_str(text);
                            content.push('\n');
                        }
                    }
                }
            }
            content
        })
}

#[derive(Default)]
pub struct DocumentIndex {
    work_dir: PathBuf,
    total_documents: usize,
    document_db: HashMap<Term, HashSet<Document>>,
    document_freq: HashMap<Term, usize>,
}

struct ParsedResult {
    content: String,
    path: PathBuf,
    parser: fn(&Path) -> Option<String>,
}

impl ParsedResult {
    pub fn new(content: String, path: PathBuf, parser: fn(&Path) -> Option<String>) -> Self {
        Self {
            content,
            path,
            parser,
        }
    }
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
                let path = entry.path();
                path.extension()
                    .and_then(|ext: &std::ffi::OsStr| ext.to_str())
                    .and_then(|ext| match ext {
                        "pdf" => read_pdf_file(path)
                            .map(|c| ParsedResult::new(c, path.to_path_buf(), read_pdf_file)),
                        "docx" => read_docx_file(path)
                            .map(|c| ParsedResult::new(c, path.to_path_buf(), read_docx_file)),
                        _ => read_text_file(path)
                            .map(|c| ParsedResult::new(c, path.to_path_buf(), read_text_file)),
                    })
            })
            .for_each(|r| {
                total_documents.fetch_add(1, Ordering::Relaxed);
                self.update_index(r, &document_db, &document_freq);
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
        parsed_result: ParsedResult,
        document_db: &Mutex<HashMap<Term, HashSet<Document>>>,
        document_freq: &Mutex<HashMap<Term, usize>>,
    ) {
        let terms = TermCollector::new(&parsed_result.content).collect::<Vec<Term>>();
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
                    let path = parsed_result.path.clone();
                    let relative_path = path.strip_prefix(&self.work_dir).unwrap_or(&path);
                    let document = Document::new(
                        parsed_result.parser,
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
