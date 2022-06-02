use std::{fmt, iter};

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
                '[' => return Ok(text),
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
                    if current_pool.is_empty() {
                        return Err("Pool cannot be empty!".to_string());
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

    fn parse_answer(&mut self) -> Result<Answer, String> {
        dbg!(self.current_line.peek());
        if self.current_line.peek() == Some(&'{') {
            return Ok(Answer::SharedPool(self.parse_idx_answer()?));
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
        let mut promised_idxs = Vec::new();
        let mut pools = None;

        self.current_line = line.chars().peekable();

        while let Some(ch) = self.current_line.peek() {
            match ch {
                '[' => dat.push((None, Some(self.parse_answer()?))),
                '{' => {
                    let idx = self.parse_idx_answer()?;
                    dat.push((None, Some(Answer::SharedPool(idx - 1))));
                    promised_idxs.push(idx);
                }
                ';' => pools = Some(self.parse_answer_pools()?),
                _ => {
                    let text = self.parse_text()?;

                    if self.current_line.peek().is_some() {
                        // Has some characters left to consume
                        dat.push((Some(text), Some(self.parse_answer()?)));
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

        Ok(Question {
            dat,
            pools: pools.unwrap_or_default(),
        })
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
    pub fn ask(&self, screen: &mut String) -> bool {
        let mut answers = Vec::new();
        let mut current = String::new();
        let mut correct = true;
        let mut used_from_pools = vec![vec![]; self.pools.len()];

        self.render_partially_answered(&answers, &current, &*screen);

        for (_, answer) in &self.dat {
            if let Some(a) = answer {
                'word: loop {
                    let ch = get_char();

                    match ch {
                        '\n' => {
                            match a {
                                Answer::Raw(r) => {
                                    if r.trim() != current.trim() {
                                        correct = false;
                                    }
                                }
                                Answer::SharedPool(p) => {
                                    let pool = self.pools.get(*p).expect("SharedPool answer variant should always hold a valid index!");
                                    let mut gotten_correct_from_pool = false;
                                    for (idx, correct_from_pool) in pool.iter().enumerate() {
                                        if current.trim() == correct_from_pool.trim()
                                            && !used_from_pools[*p].contains(&idx)
                                        {
                                            gotten_correct_from_pool = true;
                                            used_from_pools[*p].push(idx);
                                            break;
                                        }
                                    }
                                    correct = gotten_correct_from_pool;
                                }
                                Answer::OneOf(possible_answers) => {
                                    correct = correct
                                        && possible_answers
                                            .iter()
                                            .any(|ans| ans.trim() == current.trim());
                                }
                            }

                            answers.push(current);
                            current = String::new();

                            self.render_partially_answered(&answers, &current, &*screen);

                            break 'word;
                        }
                        _ => {
                            if ch as u32 == 127 {
                                // TODO(#1): Going back and editing previous answers
                                // if let None = current.pop() {
                                //     if let Some(ans) = answers.pop() {
                                //         current = ans;
                                //     }
                                // }
                                current.pop();
                            } else {
                                current.push(ch);
                            }
                        }
                    }

                    self.render_partially_answered(&answers, &current, &*screen);
                }
            }
        }

        screen.push_str(&format!("{}\n", self));
        correct
    }

    fn render_partially_answered(&self, answers: &[String], current: &str, scr: &str) {
        ncurses::clear();
        ncurses::addstr(scr);

        let mut to_render = String::new();
        let mut pos = 0;

        for (text, answer) in &self.dat {
            if let Some(s) = text {
                to_render.push_str(s);
            }

            if answer.is_some() {
                match pos.cmp(&answers.len()) {
                    std::cmp::Ordering::Less => to_render.push_str(&answers[pos]),
                    std::cmp::Ordering::Equal => {
                        to_render.push_str(current);
                        ncurses::addstr(&to_render);
                        to_render.clear();
                    }
                    std::cmp::Ordering::Greater => to_render.push_str("___"),
                }
                pos += 1;
            }
        }

        let mut x = 0;
        let mut y = 0;

        ncurses::getyx(ncurses::stdscr(), &mut y, &mut x);

        ncurses::addstr(&to_render);

        ncurses::mv(y, x);
    }
}
