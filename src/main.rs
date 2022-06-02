use std::{env, fmt, fs};

mod question;
mod tests;

use question::*;

fn main() -> Result<(), String> {
    let mut args = env::args();

    args.next(); // get rid of program name

    let src: String = args
        .filter_map(|path| match fs::read_to_string(&path) {
            Err(e) => {
                eprintln!("Could not read file `{path}`: {e}!");
                None
            }
            Ok(s) => Some(s),
        })
        .collect();

    init()?;

    let mut screen = String::new();

    for maybe_question in Parser::new(&src) {
        match maybe_question {
            Err(e) => eprintln!("{e}"),
            Ok(q) => {
                if !q.ask(&mut screen) {
                    screen.push('\n');
                    screen.push_str(&format!("WRONG! Correct answer: {q}\n"));
                }
            }
        }
    }

    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    ncurses::clear();
    ncurses::addstr(&screen);

    print("PROGRAM FINISHED. PRESS ANY KEY TO EXIT");

    ncurses::refresh();

    get_char(); // Pause so that the person can see the finish of the program

    ncurses::endwin();

    Ok(())
}

fn print<T: fmt::Display>(dat: T) {
    let buf = format!("{dat}");
    ncurses::addstr(&buf);
}

fn init() -> Result<(), String> {
    ncurses::initscr();
    ncurses::cbreak();
    ncurses::noecho();

    Ok(())
}

pub fn get_char() -> char {
    let ch = ncurses::get_wch().unwrap();
    match ch {
        ncurses::WchResult::Char(c) => char::from_u32(c).expect("Could not decode from input!"),
        ncurses::WchResult::KeyCode(_) => todo!("Handle other key inputs!"),
    }
}
