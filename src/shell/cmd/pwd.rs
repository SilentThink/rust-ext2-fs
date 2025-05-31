use super::Cmd;

pub struct Pwd;

impl Cmd for Pwd {
    fn description(&self) -> String {
        "Print name of current working directory".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        if argv.len() > 0 {
            println!("To many argument");
        } else {
            match shell.fs.pwd() {
                Ok(path) => println!("{}", path),
                Err(e) => println!("{}", e.to_string()),
            }
        }
    }

    fn help(&self) -> String {
        self.description()
    }
}
