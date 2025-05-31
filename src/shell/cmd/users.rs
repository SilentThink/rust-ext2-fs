use super::*;
use crate::utils;

pub struct Users;

impl Cmd for Users {
    fn description(&self) -> String {
        "Show users name and password".into()
    }

    fn run(&self, shell: &mut Shell, _argv: &[&str]) {
        println!("{:16} {:16}", "User Name", "Password");
        for user in shell.fs.fs_desc().users.iter().filter(|u| u.name[0] != 0) {
            println!(
                "{:16} {:16}",
                utils::str(&user.name),
                utils::str(&user.password)
            );
        }
    }
}
