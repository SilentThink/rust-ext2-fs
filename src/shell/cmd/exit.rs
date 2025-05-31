use std::process::exit;

use super::Cmd;

pub struct Exit;

impl Cmd for Exit {
    fn description(&self) -> String {
        "Exit shell".to_string()
    }

    fn run(&self, shell: &mut crate::shell::Shell, _argv: &[&str]) {
        shell.fs.exit();
        println!("Bye.");
        exit(0);
    }

    fn help(&self) -> String {
        "".into()
    }
}
