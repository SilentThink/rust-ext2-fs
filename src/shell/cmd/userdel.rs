use super::Cmd;

pub struct UserDel;

impl Cmd for UserDel {
    fn description(&self) -> String {
        "Delete user".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        if argv.len() != 1 {
            println!("Need one argument as username");
            return;
        }

        if let Err(e) = shell.fs.userdel(argv[0]) {
            println!("{}", e);
        }
    }

    fn help(&self) -> String {
        self.description() + "\n userdel [username]"
    }
}
