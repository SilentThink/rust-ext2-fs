use std::io::{stdout, Write};

use crossterm::terminal as ter;
use crossterm::cursor as cur;
use crossterm::QueueableCommand;

use super::*;

pub struct Clear;

impl Cmd for Clear {
    fn description(&self) -> String {
        "Clear Screen".into()
    }

    fn run(&self, _shell: &mut Shell, _argv: &[&str]) {
        let mut stdout = stdout();
        stdout.queue(ter::Clear(ter::ClearType::All)).unwrap();
        stdout.queue(cur::MoveTo(0, 0)).unwrap();
        stdout.flush().unwrap_or_default();
    }
}
