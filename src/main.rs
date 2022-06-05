use std::{env, fs};

mod render;
mod question;
mod tests;

use question::*;
use render::{NCurses, Render};

fn main() -> Result<(), String> {
    let mut args = env::args();

    args.next(); // get rid of program name

    let args = args.collect::<Vec<String>>();

    let mut window = NCurses::init()?;

    for path in args {
        match fs::read_to_string(&path) {
            Err(e) => {
                return Err(format!("Could not read file `{path}`: {e}!"));
            }
            Ok(src) => {
                for maybe_question in Parser::new(&src, &path) {
                    match maybe_question {
                        Err(e) => {
                            eprintln!("{e}");
                            window.display_error(&e);
                        },
                        Ok(q) => {
                            let answers = window.ask(q.renderable())?;
                            let correction = q.check_answers(answers);
                            window.show_result(correction);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
