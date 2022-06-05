use std::{collections, fmt, iter};

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
            Answer::SharedPool(idx) => write!(f, "{{one of the #{} set}}", idx + 1),
            Answer::OneOf(v) => {
                for (idx, possible_answer) in v.iter().enumerate() {
                    if idx > 0 {
                        write!(f, " OR ")?;
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
    src_name: &'a str,
    src: iter::Enumerate<std::str::Lines<'a>>,
    current_line: iter::Peekable<std::str::CharIndices<'a>>,
}

pub type ParseResult<T> = Result<T, (usize, String)>;

impl<'a> Parser<'a> {
    pub fn new(src: &'a str, src_name: &'a str) -> Self {
        Self {
            src_name,
            src: src.lines().enumerate(),
            current_line: "".char_indices().peekable(), // Never will be touched, and if it is, it'll throw an error.
        }
    }

    fn parse_text(&mut self) -> ParseResult<String> {
        let mut text = String::new();

        while let Some((idx, ch)) = self.current_line.peek() {
            match ch {
                '[' | ';' | '{' => return Ok(text),
                ']' => return Err((*idx, String::from("Unexpected `]`!"))),
                _ => text.push(self.current_line.next().unwrap().1),
            }
        }
        Ok(text)
    }

    fn parse_answer_pools(&mut self) -> ParseResult<Vec<Vec<String>>> {
        assert_eq!(self.current_line.next().map(|x| x.1), Some(';'));

        let mut pools = Vec::new();
        let mut current_pool = Vec::new();
        let mut current_string = String::new();

        for (idx, ch) in &mut self.current_line {
            match ch {
                ';' => {
                    if current_pool.is_empty() && current_string.is_empty() {
                        return Err((idx, "Pool cannot be empty!".to_string()));
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

    fn parse_idx_answer(&mut self) -> ParseResult<usize> {
        let start = self.current_line.next();
        assert_eq!(start.map(|x| x.1), Some('{'));

        let start = start.unwrap().0;

        let mut t = String::new();

        for (idx, ch) in self.current_line.by_ref() {
            match ch {
                '}' => {
                    return t
                        .trim()
                        .parse::<usize>()
                        .map_err(|_| (start, "Not a number!".to_string()));
                }
                _ => t.push(ch),
            }
        }
        Err((start, String::from("Expected end of answer!")))
    }

    fn parse_answer(
        &mut self,
        promised_idxs: &mut collections::HashSet<usize>,
    ) -> ParseResult<Answer> {

        if self.current_line.peek().map(|x| x.1) == Some('{') {
            let idx = self.parse_idx_answer()? - 1;
            promised_idxs.insert(idx);
            return Ok(Answer::SharedPool(idx));
        }

        let start = self.current_line.next();
        assert_eq!(start.map(|x| x.1), Some('['));

        let start_idx = start.unwrap().0;

        let mut possible_answers = vec![];
        let mut current_answer = String::new();

        for (idx, ch) in self.current_line.by_ref() {
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
                '[' => return Err((idx, String::from("Unexpected `[`!"))),
                _ => current_answer.push(ch),
            }
        }
        Err((start_idx, String::from("Unexpected end of answer!")))
    }

    fn parse_question(&mut self, line: &'a str) -> ParseResult<Question> {
        let mut dat: Vec<(Option<String>, Option<Answer>)> = Vec::new();
        let mut promised_idxs = collections::HashSet::new();
        let mut pools = None;

        self.current_line = line.char_indices().peekable();

        let mut last_idx = 0;
        let mut pool_idx = None;

        while let Some((idx, ch)) = self.current_line.peek() {
            last_idx = *idx;
            match ch {
                '[' => dat.push((None, Some(self.parse_answer(&mut promised_idxs)?))),
                '{' => {
                    let idx = self.parse_idx_answer()? - 1;
                    dat.push((None, Some(Answer::SharedPool(idx))));
                    promised_idxs.insert(idx);
                }
                ';' => {
                    pool_idx = Some(idx);
                    pools = Some(self.parse_answer_pools()?);
                },
                '}' | ']' => return Err((idx, String::from("Unexpected closing bracket!"))),
                _ => {
                    let text = self.parse_text()?;

                    if let Some((idx, ch)) = self.current_line.peek() {
                        // Has some characters left to consume
                        let ans = match ch {
                            ';' => {
                                pool_idx = Some(idx);
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
                return Err((last_idx, format!(
                    "Expected {} pools, but none were provided!",
                    promised_idxs.len()
                )));
            }

            let pools = pools.as_ref().unwrap();

            if promised_idxs.len() != pools.len() {

                if promised_idxs.len() == 1 {
                    return Err((*pool_idx.expect("The index to the start of the pools should always be set!"), format!("Expected 1 pool, but found {}!", pools.len())));
                } else {
                    return Err((*pool_idx.expect("The index to the start of the pools should always be set!"),format!(
                        "Expected {} pools, but found {}!",
                        promised_idxs.len(),
                        pools.len()
                    )));
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

    fn next(&mut self) -> Option<Self::Item> {
        let (line_number, line) = self.src.next()?;

        Some(self.parse_question(line).map_err(|(idx, msg)| format!("{}:{line_number}:{idx} {msg}", self.src_name)))
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

        for (idx, pool) in self.pools.iter().enumerate() {
            write!(f, ". Set #{}: ", idx + 1)?;
            for (item_idx, item) in pool.iter().enumerate() {
                if item_idx > 0 {
                    write!(f, ",")?;
                }
                write!(f, " {item}")?;
            }
        }
        Ok(())
    }
}

impl Question {
    pub fn check_answers(&self, answers: Vec<String>) -> Option<String> {

        let mut used_from_pools = vec![Vec::new(); self.pools.len()];
        let mut all_correct = true;

        for (expected, provided) in self.dat.iter()
                                            .filter_map(|(_, ans)| ans.as_ref())
                                            .zip(answers.iter()) {

            let correct = match expected {
                Answer::Raw(raw) => raw.trim().to_lowercase() == provided.trim().to_lowercase(),

                Answer::SharedPool(pool_idx) => {

                    let pool = self.pools.get(*pool_idx)
                                         .expect("Indexes to shared pools should have been checked when question was constructed!");

                    let mut res = false;
                    for (option_idx, option) in pool.iter().enumerate() {
                        let used = used_from_pools.get_mut(*pool_idx).unwrap();

                        if (used.is_empty()
                            || !used.contains(&option_idx))
                            && provided.trim().to_lowercase() == option.trim().to_lowercase() {

                            used.push(option_idx);
                            res = true;
                            break;
                        }
                    }
                    res
                },

                Answer::OneOf(options) => {
                    options.iter().any(|opt| opt.trim().to_lowercase() == provided.trim().to_lowercase())
                },
            };

            if !correct {
                all_correct = false;
            }
        }

        if all_correct {
            None
        } else {
            Some(format!("{self}"))
        }
    }

    pub fn renderable(&self) -> impl Iterator<Item=(Option<&str>, bool)> {
        self.dat
            .iter()
            .map(|(q, ans)| (q.as_ref().map(|s| s.as_str()), ans.is_some()))
    }
}
