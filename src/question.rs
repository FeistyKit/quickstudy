use std::{fmt, iter};

use crate::get_char;

#[derive(Debug, PartialEq, Eq)]
enum Answer {
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
    dat: Vec<(Option<String>, Option<Answer>)>,
    pools: Vec<Vec<String>>,
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
        todo!()
    }

    fn parse_idx_answer(&mut self) -> Result<usize, String> {
        assert_eq!(self.current_line.next(), Some('{'));

        let mut t = String::new();

        while let Some(ch) = self.current_line.next() {
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
        assert_eq!(self.current_line.next(), Some('['));

        let mut possible_answers = vec![];
        let mut current_answer = String::new();

        while let Some(ch) = self.current_line.next() {
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

        while self.current_line.peek().is_some() {
            match self.current_line.peek() {
                Some('[') => dat.push((None, Some(self.parse_answer()?))),
                Some('{') => {
                    let idx = self.parse_idx_answer()?;
                    promised_idxs.push(idx);
                }
                Some(';') => pools = Some(self.parse_answer_pools()?),
                Some(_) => {
                    let text = self.parse_text()?;

                    if self.current_line.peek().is_some() {
                        // Has some characters left to consume
                        dat.push((Some(text), Some(self.parse_answer()?)));
                    } else {
                        // finished
                        dat.push((Some(text), None));
                    }
                }
                None => unreachable!(),
            }
        }

        Ok(Question {
            dat,
            pools: pools.unwrap_or(Vec::new()),
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
    // TODO(#2): The cursor renders as well
    // Either we should move the cursor while typing or hide it.
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
                to_render.push_str(&s);
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
