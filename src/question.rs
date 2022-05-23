use std::{fmt, iter};

use crate::get_char;

#[derive(Debug, PartialEq, Eq)]
pub struct Question {
    dat: Vec<(Option<String>, Option<String>)>,
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

    fn parse_answer(&mut self) -> Result<String, String> {
        assert_eq!(self.current_line.next(), Some('['));

        let mut answer = String::new();

        while let Some(ch) = self.current_line.next() {
            match ch {
                ']' => return Ok(answer),
                '[' => return Err(String::from("Unexpected `[`!")),
                _ => answer.push(ch),
            }
        }
        Err(String::from("Unexpected end of answer!"))
    }

    fn parse_question(&mut self, line: &'a str) -> ParseResult {
        let mut dat = Vec::new();

        self.current_line = line.chars().peekable();

        while self.current_line.peek().is_some() {
            match self.current_line.peek() {
                Some('[') => dat.push((None, Some(self.parse_answer()?))),
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

        Ok(Question { dat })
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

        self.render_partially_answered(&answers, &current, &*screen);

        for (_, answer) in &self.dat {
            if let Some(a) = answer {
                'word: loop {
                    let ch = get_char();

                    match ch {
                        '\n' => {
                            if a.trim() != current {
                                correct = false;
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
