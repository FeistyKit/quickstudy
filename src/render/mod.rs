mod tui;

pub use tui::NCurses;

use std::iter;

pub trait Render: Sized {
    fn ask<'a, I>(&mut self, question: I) -> Result<Vec<String>, String>
        where I: iter::Iterator<Item = (Option<&'a str>, bool)>;

    fn show_result(&mut self, correction: Option<String>);

    fn init() -> Result<Self, String>;

    fn display_error(&mut self, _err: &str) {}

    fn finish(&mut self) -> Result<(), String> {
        Ok(())
    }
}

