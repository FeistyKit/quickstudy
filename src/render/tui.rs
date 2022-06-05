use std::{fmt, iter};

use crate::Render;

pub struct NCurses {
    screen: String
}

impl Drop for NCurses {
    fn drop(&mut self) {
        self.finish().unwrap()
    }
}

impl NCurses {
    pub fn get_char() -> char {
        let ch = ncurses::get_wch().unwrap();
        match ch {
            ncurses::WchResult::Char(c) => char::from_u32(c).expect("Could not decode from input!"),
            ncurses::WchResult::KeyCode(_) => todo!("Handle other key inputs!"),
        }
    }

    fn print<T: fmt::Display>(dat: T) {
        let buf = format!("{dat}");
        ncurses::addstr(&buf);
    }

    pub fn render_partially_answered<'a>(&self, answers: &[String], data: impl iter::Iterator<Item=&'a (Option<&'a str>, bool)>, current: &str) -> String {
        ncurses::clear();
        ncurses::addstr(&self.screen);

        let mut to_render = String::new();
        let mut pos = 0;

        for (text, answer) in data {
            if let Some(s) = text {
                to_render.push_str(s);
            }

            if *answer {
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

        to_render
    }
}

impl Render for NCurses {
    fn ask<'a, I>(&mut self, question: I) -> Result<Vec<String>, String> where I: iter::Iterator<Item = (Option<&'a str>, bool)>{
        let mut answers = Vec::new();
        let mut current = String::new();

        let question = question.collect::<Vec<_>>();

        self.render_partially_answered(&answers, question.iter(), &current);

        for (_, answer) in &question {
            if *answer {
                'word: loop {
                    let ch = Self::get_char();

                    match ch {
                        '\n' => {
                            answers.push(current);
                            current = String::new();

                            self.render_partially_answered(&answers, question.iter(), &current);

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

                    self.render_partially_answered(&answers, question.iter(), &current);
                }
            }
        }

        let s = self.render_partially_answered(&answers, question.iter(), &current);
        self.screen.push_str(&s);
        self.screen.push('\n');
        Ok(answers)
    }


    fn init() -> Result<Self, String> {
        ncurses::initscr();
        ncurses::cbreak();
        ncurses::noecho();

        Ok(Self {
            screen: String::new()
        })
    }

    fn finish(&mut self) -> Result<(), String> {
        ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        ncurses::clear();
        ncurses::addstr(&self.screen);

        Self::print("PROGRAM FINISHED. PRESS ANY KEY TO EXIT");

        ncurses::refresh();

        Self::get_char(); // Pause so that the person can see the finish of the program

        ncurses::endwin();

        Ok(())
    }

    fn display_error(&mut self, err: &str) {
        Self::print("ERROR: ");
        Self::print(err);
        Self::print(". Press any button to continue.\n");
        Self::get_char();
        ncurses::clear();
        ncurses::addstr(&self.screen);
    }

    fn show_result(&mut self, correction: Option<String>) {
        if let Some(correction) = correction {
            Self::print("INCORRECT. The correct answer is: \"");
            Self::print(&correction);
            Self::print("\".\n");

            let mut response = String::new();

            'response: loop {

                Self::print("Please type that out: ");

                loop {
                    let ch = Self::get_char();

                    if ch == '\n' {
                        if response.trim() == correction.trim().to_lowercase() {
                            break 'response;
                        } else {
                            continue 'response;
                        }
                    } else {
                        response.push(ch.to_lowercase().next().unwrap())
                    }
                }
            }
        }
    }
}
