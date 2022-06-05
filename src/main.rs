use std::{env, fs};

mod render;
mod question;
mod tests;

use question::*;
#[cfg(feature = "tui")]
use render::{NCurses, Render};
#[cfg(not(feature = "tui"))]
use render::{Render, Cli};

fn main() -> Result<(), String> {
    let mut args = env::args();

    args.next(); // get rid of program name

    let args = args.collect::<Vec<String>>();

    #[cfg(feature = "tui")]
    let mut window = NCurses::init()?;
    #[cfg(not(feature = "tui"))]
    let mut window = Cli::init()?;

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
