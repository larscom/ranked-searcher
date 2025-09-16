use rust_stemmers::{Algorithm, Stemmer};

pub struct Lexer<'a> {
    content: &'a [char],
    stemmer: Stemmer,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self {
            content,
            stemmer: Stemmer::create(Algorithm::English),
        }
    }

    fn chop_left(&mut self) {
        while !self.content.is_empty()
            && (self.content[0].is_whitespace() || !self.content[0].is_alphanumeric())
        {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    pub fn next_word_stemmed(&mut self) -> Option<String> {
        self.next_word().map(|w| self.stemmer.stem(&w).into_owned())
    }

    pub fn next_word(&mut self) -> Option<String> {
        self.chop_left();

        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|x| x.is_numeric()).iter().collect());
        }

        if self.content[0].is_alphabetic() {
            let word = self
                .chop_while(|x| x.is_alphanumeric() || x.is_numeric())
                .iter()
                .map(|x| x.to_ascii_lowercase())
                .collect::<String>();

            return Some(word);
        }

        None
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_word_stemmed()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    #[test]
    fn test_stemmed_iterator() {
        let content = "Running runners quickly run".chars().collect::<Vec<char>>();

        for (index, word) in Lexer::new(&content).enumerate() {
            match index {
                0 => assert_eq!(word, "run"),
                1 => assert_eq!(word, "runner"),
                2 => assert_eq!(word, "quick"),
                3 => assert_eq!(word, "run"),
                _ => panic!("did not expect value at index 4"),
            }
        }
    }
}
