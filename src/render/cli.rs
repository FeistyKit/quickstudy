use super::Render;

use std::{process, io::{self, BufRead, Write}, string};

pub struct Cli {
    stdin: io::StdinLock<'static>
}

impl Render for Cli {
    fn ask<'a, I>(&mut self, question: I) -> Result<Vec<String>, String>
        where I: std::iter::Iterator<Item = (Option<&'a str>, bool)> {

        let to_display = question.map(|(q, ans)| {
            let mut s = None;
            if let Some(q) = q {
                s = Some(q.to_string());
            }
            if ans {
                s.get_or_insert(String::new()).push_str("___");
            }
            s.expect("Renderer should never be given an item without a question or an answer")
        }).collect::<String>();

        assert_ne!(to_display.trim(), "");

        #[cfg(windows)]
        process::Command::new("cls").output().expect("Could not clear screen");
        #[cfg(not(windows))]
        process::Command::new("clear").output().expect("Could not clear screen");

        print!("{to_display}: ");
        io::stdout().flush().expect("Could not flush stdout!");

        let mut buf = String::new();

        self.stdin.read_line(&mut buf).expect("Could not read from standard input!");

        Ok(buf.trim().split(',').map(string::ToString::to_string).collect())
    }

    fn show_result(&mut self, correction: Option<String>) {
        if let Some(correction) = correction {
            println!("INCORRECT! The correct answer is `{correction}`!");
            loop {
                print!("Please type that out: ");
                io::stdout().flush().expect("Could not flush stdout!");

                let mut buf = String::new();

                self.stdin.read_line(&mut buf).expect("Could not read from standard input!");

                if correction.trim().to_lowercase() == buf.trim().to_lowercase() {
                    break;
                }
            }
        }
    }

    fn finish(&mut self) -> Result<(), String> {
        println!("Done!");
        io::stdout().flush().expect("Could not flush stdout!");
        Ok(())
    }

    fn init() -> Result<Self, String> {
        let stdin = io::stdin();
        Ok(Self {
            stdin: stdin.lock()
        })
    }
}

impl Drop for Cli {
    fn drop(&mut self) {
        self.finish().unwrap()
    }
}
