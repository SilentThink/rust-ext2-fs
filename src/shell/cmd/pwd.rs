use super::Cmd;

pub struct Pwd;

impl Cmd for Pwd {
    fn description(&self) -> String {
        "Print current working directory".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        match argv.get(0) {
            Some(&"-h") => println!("{}", self.help()),
            _ => {
                println!("{}", shell.fs.pwd());
            }
        }
    }

    fn help(&self) -> String {
        self.description() + r#"

pwd
Print the full filename of the current working directory."#
    }
}
