use std::fmt;
use std::io;

fn main() {
    let q = Question {
        texts: vec!["q1".to_string(), "q2".to_string()],
        answers: vec!["a1".to_string()],
        starts_text: true,
    };
    println!("{}", q);
}

#[derive(Debug)]
struct Question {
    texts: Vec<String>,
    answers: Vec<String>,
    starts_text: bool,
}

impl Question {
    fn ask(&self) -> bool {
        let stdin = io::stdin();
        let stdin = stdin.lock();
        todo!()
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = self.starts_text;
        let mut pos = 0;

        loop {
            if text {
                write!(f, "{} ", self.texts[pos])?;
                pos += 1;
            } else {
                write!(f, "___ ")?;
            }
            text = !text;
            if pos >= self.texts.len().max(self.answers.len()) {
                break;
            }
        }
        Ok(())
    }
}
