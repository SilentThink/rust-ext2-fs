pub mod cmd;

use std::io::Write;

use self::cmd::Cmds;
use super::fs::Fs;
use crossterm::style::Stylize;

pub struct Shell {
    pub fs: Fs,
    pub cmds: Cmds,
}

impl Shell {
    pub fn new() -> Self {
        let fs = match Fs::init() {
            Ok(fs) => fs,
            Err(err) => {
                println!("Err: {}", err.to_string().red());
                println!("Format disk? [y/n]");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                match input.trim() {
                    "Y" | "y" => Fs::format().unwrap(),
                    _ => std::process::exit(1),
                }
            }
        };

        Self {
            fs,
            cmds: cmd::cmds(),
        }
    }

    pub fn init_cmds(&mut self) {}

    pub fn run(&mut self) {
        println!(
            "\n {} ",
            " Welcome to simple unix like filesystem. ".on_white().black()
        );
        println!("");

        if !cmd::login::Login::login(&mut self.fs) {
            return;
        }

        let cmds = self.cmds.clone();

        loop {
            print!("[{}] ", self.fs.pwd().green());
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let mut argv: Vec<&str> = input
                .trim()
                .split(' ')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if argv.len() < 1 {
                continue;
            }

            match cmds.get(argv.remove(0)) {
                Some(cmd) => match argv.contains(&"-h") {
                    true => println!("{}", cmd.help()),
                    false => cmd.run(self, &argv),
                },
                None => println!(
                    "{}",
                    "Command not found. Press `help` or `?` to see all command".red()
                ),
            }
        }
    }
}
