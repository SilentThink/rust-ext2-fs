use std::io::{stdin, stdout, Write};

use crate::fs::ByteArray;

use super::*;

pub struct Passwd;

impl Cmd for Passwd {
    fn description(&self) -> String {
        "Change user's password".to_string()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        let mut user = shell.fs.current_user();

        if argv.len() == 1 {
            let mut to_search = None;
            for (i, u) in shell.fs.fs_desc().users.iter().enumerate() {
                if argv[0].into_array().unwrap() == u.name {
                    to_search = Some(i);
                    break;
                }
            }
            match to_search {
                Some(i) => user = i,
                None => {
                    println!("User {} is not exists.", argv[0]);
                    return;
                }
            }
        }
        if argv.len() > 1 {
            println!("Too many arguments");
            return;
        }

        let mut password1 = String::new();
        let mut password2 = String::new();

        print!("New password: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut password1).unwrap();

        print!("Press again:  ");
        stdout().flush().unwrap();
        stdin().read_line(&mut password2).unwrap();

        if password1 != password2 {
            println!("Comfirm failed");
            return;
        }

        if let Err(e) = shell.fs.passwd(user, &password1.trim()) {
            println!("{}", e);
        };
    }

    fn help(&self) -> String {
        self.description() + "\n passwd [username]"
    }
}
