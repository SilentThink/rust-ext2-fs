use std::io::Write;

use crossterm::style::Stylize;

use super::Cmd;
use crate::fs::Fs;

pub struct Login;

impl Login {
    fn username() -> String {
        println!("{}", ":: default user: root, password: 123".green());

        print!("username: ");
        std::io::stdout().flush().unwrap();
        let mut username = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        return username.trim().into();
    }

    fn passwd() -> String {
        print!("password: ");
        std::io::stdout().flush().unwrap();
        let mut password = String::new();
        std::io::stdin().read_line(&mut password).unwrap();
        return password.trim().into();
    }

    pub fn login_with_name(fs: &mut Fs, username: &str) -> bool {
        let password = Self::passwd();

        if let Err(e) = fs.login(&username.trim(), &password.trim()) {
            println!("{}", e);
            return false;
        }

        let username = username.trim();
        let path_to_switch = match username == "root" {
            true => "/root".into(),
            false => format!("/home/{}", username),
        };

        if let Err(e) = fs.chdir(&path_to_switch) {
            println!("Can't chdir to {}: {}", path_to_switch, e);
        }
        return true;
    }

    pub fn login(fs: &mut Fs) -> bool {
        Self::login_with_name(fs, &Self::username())
    }
}

impl Cmd for Login {
    fn description(&self) -> String {
        "Login with a username".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        match argv.len() {
            0 => Self::login(&mut shell.fs),
            1 => Self::login_with_name(&mut shell.fs, argv[0]),
            _ => {
                println!("Too many argument");
                return;
            }
        };
    }

    fn help(&self) -> String {
        self.description() + "\n login [username]"
    }
}
