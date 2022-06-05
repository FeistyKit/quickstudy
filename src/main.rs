use std::{env, fs};

mod render;
mod question;
mod tests;

use question::*;
use render::{NCurses, Render};

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

    let mut window = NCurses::init()?;

    for maybe_question in Parser::new(&src) {
        match maybe_question {
            Err(e) => {
                eprintln!("{e}");
            },
            Ok(q) => {
                let answers = window.ask(q.renderable())?;
                let correction = q.check_answers(answers);
                window.show_result(correction);
            }
        }
    }
    Ok(())

}
