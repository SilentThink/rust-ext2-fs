use crate::shell::Shell;

use super::Cmd;

pub struct Mkdir;

impl Cmd for Mkdir {
    fn description(&self) -> String {
        "Create directory".into()
    }

    fn run(&self, Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        for dir in argv {
            if let Err(e) = fs.mkdir(dir) {
                println!("{}: {}", dir, e.to_string())
            }
        }
    }

    fn help(&self) -> String {
        self.description()
            + r#"

  eg: mkdir dir1 dir2 dir3"#
    }
}
