use std::{collections, fmt, iter};

use crate::get_char;

#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    Raw(String),
    SharedPool(usize), // Index into list of list of options.
    OneOf(Vec<String>),
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Answer::Raw(s) => write!(f, "{s}"),
            Answer::SharedPool(idx) => write!(f, "{{{idx}}}"),
            Answer::OneOf(v) => {
                for (idx, possible_answer) in v.iter().enumerate() {
                    if idx > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{possible_answer}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Question {
    pub dat: Vec<(Option<String>, Option<Answer>)>,
    pub pools: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct Parser<'a> {
    src: std::str::Lines<'a>,
    current_line: iter::Peekable<std::str::Chars<'a>>,
}

pub type ParseResult = Result<Question, String>;

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.lines(),
            current_line: "".chars().peekable(), // Never will be touched, and if it is, it'll throw an error.
        }
    }

    fn parse_text(&mut self) -> Result<String, String> {
        let mut text = String::new();

        while let Some(ch) = self.current_line.peek() {
            match ch {
                '[' | ';' | '{' => return Ok(text),
                ']' => return Err(String::from("Unexpected `]`!")),
                _ => text.push(self.current_line.next().unwrap()),
            }
        }
        Ok(text)
    }

    fn parse_answer_pools(&mut self) -> Result<Vec<Vec<String>>, String> {
        assert_eq!(self.current_line.next(), Some(';'));

        let mut pools = Vec::new();
        let mut current_pool = Vec::new();
        let mut current_string = String::new();

        for ch in &mut self.current_line {
            match ch {
                ';' => {
                    if current_pool.is_empty() && current_string.is_empty() {
                        return Err("Pool cannot be empty!".to_string());
                    }
                    if !current_string.is_empty() {
                        current_pool.push(current_string);
                        current_string = String::new();
                    }
                    pools.push(current_pool);
                    current_pool = Vec::new();
                }
                ',' => {
                    current_pool.push(current_string);
                    current_string = String::new();
                }
                _ => current_string.push(ch),
            }
        }
        if !current_string.is_empty() {
            current_pool.push(current_string);
        }
        if !current_pool.is_empty() {
            pools.push(current_pool);
        }
        Ok(pools)
    }

    fn parse_idx_answer(&mut self) -> Result<usize, String> {
        assert_eq!(self.current_line.next(), Some('{'));

        let mut t = String::new();

        for ch in self.current_line.by_ref() {
            match ch {
                '}' => {
                    return t
                        .trim()
                        .parse::<usize>()
                        .map_err(|_| "Not a number!".to_string());
                }
                _ => t.push(ch),
            }
        }
        Err(String::from("Expected end of answer!"))
    }

    fn parse_answer(
        &mut self,
        promised_idxs: &mut collections::HashSet<usize>,
    ) -> Result<Answer, String> {
        if self.current_line.peek() == Some(&'{') {
            let idx = self.parse_idx_answer()? - 1;
            promised_idxs.insert(idx);
            return Ok(Answer::SharedPool(idx));
        }
        assert_eq!(self.current_line.next(), Some('['));

        let mut possible_answers = vec![];
        let mut current_answer = String::new();

        for ch in self.current_line.by_ref() {
            match ch {
                '|' => {
                    possible_answers.push(current_answer.trim().to_string());
                    current_answer = String::new();
                }
                ']' => {
                    if possible_answers.is_empty() {
                        return Ok(Answer::Raw(current_answer.trim().to_string()));
                    } else {
                        possible_answers.push(current_answer.trim().to_string());
                        return Ok(Answer::OneOf(possible_answers));
                    }
                }
                '[' => return Err(String::from("Unexpected `[`!")),
                _ => current_answer.push(ch),
            }
        }
        Err(String::from("Unexpected end of answer!"))
    }

    fn parse_question(&mut self, line: &'a str) -> ParseResult {
        let mut dat: Vec<(Option<String>, Option<Answer>)> = Vec::new();
        let mut promised_idxs = collections::HashSet::new();
        let mut pools = None;

        self.current_line = line.chars().peekable();

        while let Some(ch) = self.current_line.peek() {
            match ch {
                '[' => dat.push((None, Some(self.parse_answer(&mut promised_idxs)?))),
                '{' => {
                    let idx = self.parse_idx_answer()?;
                    dat.push((None, Some(Answer::SharedPool(idx - 1))));
                    promised_idxs.insert(idx);
                }
                ';' => pools = Some(self.parse_answer_pools()?),
                _ => {
                    let text = self.parse_text()?;

                    if let Some(ch) = self.current_line.peek() {
                        // Has some characters left to consume
                        let ans = match ch {
                            ';' => {
                                pools = Some(self.parse_answer_pools()?);
                                None
                            }
                            '[' | '{' => Some(self.parse_answer(&mut promised_idxs)?),
                            _ => unreachable!("The only things that would stop the parsing of text have already been matched")
                        };
                        dat.push((Some(text), ans));
                    } else {
                        // finished
                        dat.push((Some(text), None));
                    }
                }
            }
        }

        if !promised_idxs.is_empty() {
            if pools.is_none() {
                return Err(format!(
                    "Expected {} pools, but none were provided!",
                    promised_idxs.len()
                ));
            }
            let pools = pools.as_ref().unwrap();
            if promised_idxs.len() != pools.len() {
                if promised_idxs.len() == 1 {
                    return Err(format!("Expected 1 pool, but found {}!", pools.len()));
                } else {
                    return Err(format!(
                        "Expected {} pools, but found {}!",
                        promised_idxs.len(),
                        pools.len()
                    ));
                }
            }
        }

        let mut pools = pools.unwrap_or_default();
        for pool in &mut pools {
            for item in pool {
                *item = item.trim().to_string();
            }
        }

        Ok(Question { dat, pools })
    }
}

impl<'a> iter::Iterator for Parser<'a> {
    type Item = Result<Question, String>;

    fn next(&mut self) -> Option<ParseResult> {
        let line = self.src.next()?;

        Some(self.parse_question(line))
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (q, a) in &self.dat {
            if let Some(s) = q {
                write!(f, "{s}")?;
            }
            if let Some(a) = a {
                write!(f, "{a}")?;
            }
        }
        Ok(())
    }
}

impl Question {
    pub fn ask(&self, answers: Vec<String>) -> bool {

        let mut used_from_pools = vec![Vec::new(); self.pools.len()];

        for (expected, provided) in self.dat.iter()
                                            .filter_map(|(_, ans)| ans.as_ref())
                                            .zip(answers.iter()) {

            let correct = match expected {
                Answer::Raw(raw) => raw.trim().to_lowercase() == provided.trim().to_lowercase(),

                Answer::SharedPool(pool_idx) => {

                    let pool = self.pools.get(*pool_idx)
                                         .expect("Indexes to shared pools should have been checked when question was constructed!");

                    pool.iter().enumerate().filter(|(option_idx, option)| {
                        let used = used_from_pools.get_mut(*pool_idx).unwrap();

                        if used.is_empty()
                            || !used.contains(option_idx)
                            && provided.trim().to_lowercase() == option.trim().to_lowercase() {

                            used.push(*option_idx);
                            true
                        } else {
                            false
                        }
                    }).next().is_some()
                },

                Answer::OneOf(options) => {
                    options.iter().any(|opt| opt.trim().to_lowercase() == provided.trim().to_lowercase())
                },
            };

            if !correct {
                return false;
            }
        }
        true
    }

}
