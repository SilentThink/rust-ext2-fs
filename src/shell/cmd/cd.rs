use super::Cmd;

pub struct Cd;

impl Cmd for Cd {
    fn description(&self) -> String {
        "Change work directory".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        match argv.get(0) {
            None => println!("Need a filename"),
            Some(path) => {
                if let Err(msg) = shell.fs.chdir(path) {
                    println!("{}", msg.to_string())
                }
            }
        }
    }

    fn help(&self) -> String {
        self.description()
            + r"#
        cd <dir>

        -h show help message#"
    }
}
