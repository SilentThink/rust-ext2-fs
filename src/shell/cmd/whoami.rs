use crate::utils::str;

use super::*;

pub struct Whoami;

impl Cmd for Whoami {
    fn description(&self) -> String {
        "Show current user".into()
    }

    fn run(&self, shell: &mut Shell, _argv: &[&str]) {
        let fs_desc = shell.fs.fs_desc();
        let name = fs_desc.users[shell.fs.current_user()].name;
        println!("{}", str(&name))
    }
}
