use ncurses;
use std::{fmt, iter};

fn main() -> Result<(), String> {
    init()?;

    let q = Question::new(["q1", "q2", "q3"], ["a1", "a2", "a3"], true);

    let answer = q.ask();

    print("\n");

    print(answer);

    print("\nPROGRAM FINISHED. PRESS ANY KEY TO EXIT");

    ncurses::refresh();

    get_char(); // Pause so that the person can see the finish of the program

    ncurses::endwin();

    Ok(())
}

fn print<T: fmt::Display>(dat: T) {
    let buf = format!("{dat}");
    ncurses::addstr(&buf);
}

#[derive(Debug)]
struct Question {
    dat: Vec<(Option<String>, Option<String>)>,
}

const REGULAR_COLOR: i16 = 1;
const HIGHLIGHTED_COLOR: i16 = 2;

fn init() -> Result<(), String> {
    ncurses::initscr();
    ncurses::cbreak();
    ncurses::noecho();

    if !ncurses::has_colors() {
        return Err("Terminal does not support colours!".to_string());
    }
    ncurses::start_color();
    ncurses::init_pair(REGULAR_COLOR, ncurses::COLOR_WHITE, ncurses::COLOR_BLACK);
    ncurses::init_pair(HIGHLIGHTED_COLOR, ncurses::COLOR_BLACK, ncurses::COLOR_WHITE);

    Ok(())
}

fn get_char() -> char {
    let ch = ncurses::get_wch().unwrap();
    match ch {
        ncurses::WchResult::Char(c) => char::from_u32(c).expect("Could not decode from input!"),
        ncurses::WchResult::KeyCode(_) => todo!("Handle other key inputs!"),
    }
}

fn pad_str(i: String) -> String {
    let mut t = String::with_capacity(i.len());
    t.push(' ');
    t.push_str(i.trim());
    t.push(' ');
    t
}

impl Question {

    fn new<T: ToString>(
        questions: impl iter::IntoIterator<Item = T>,
        answers: impl iter::IntoIterator<Item = T>,
        mut answer_first: bool,
    ) -> Self {
        let mut questions = questions.into_iter().map(|x| pad_str(x.to_string()));
        let mut answers = answers.into_iter().map(|x| pad_str(x.to_string()));

        let mut dat = Vec::new();

        loop {

            let next_q = if answer_first {
                answer_first = false;
                None
            } else {
                questions.next()
            };

            let next_a = answers.next();

            if next_q.is_some() || next_a.is_some() {
                dat.push((next_q, next_a));
            } else {
                break
            }

        }

        Self {
            dat
        }

    }

    fn ask(&self, screen: &mut String) -> bool {
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

                        },
                        _ => {
                            if ch as u32 == 127 {
                                // TODO: Going back and editing previous answers
                                // if let None = current.pop() {
                                //     if let Some(ans) = answers.pop() {
                                //         current = ans;
                                //     }
                                // }
                                current.pop();
                            } else {
                                current.push(ch);
                            }
                        },
                    }

                    self.render_partially_answered(&answers, &current, &*screen);
                }
            }
        }
        correct
    }

    fn render_partially_answered(&self, answers: &[String], current: &str, scr: &str) {

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
                        let attr = ncurses::COLOR_PAIR(HIGHLIGHTED_COLOR);
                        ncurses::attron(attr);
                        ncurses::addch(' ' as u32);
                        ncurses::attroff(attr);
                    },
                    std::cmp::Ordering::Greater => to_render.push_str("___"),
                }
                pos += 1;
            }
        }

        ncurses::addstr(&to_render);
    }
}
