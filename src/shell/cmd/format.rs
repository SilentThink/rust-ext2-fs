use std::io::Write;
use super::*;

pub struct Format;

impl Cmd for Format {
    fn description(&self) -> String {
        "Format your disk".into()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut Shell, _argv: &[&str]) {
        println!("!!! This opretion will wipe all data in this disk");
        print!("!!! Continue ? [Y/N]   ");
        std::io::stdout().flush().unwrap();

        let mut i = String::new();
        std::io::stdin().read_line(&mut i).unwrap();

        if i.trim() == "Y" || i.trim() == "y" {
            match crate::fs::Fs::format() {
                Ok(f) => *fs = f,
                Err(e) => println!("{}", e.to_string()),
            }
        }
    }
}
