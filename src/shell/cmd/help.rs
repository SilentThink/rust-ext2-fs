use super::*;

pub struct Help;

impl Cmd for Help {
    fn description(&self) -> String {
        "Show all commands and its description".into()
    }

    fn run(&self, shell: &mut Shell, _: &[&str]) {
        for (&name, cmd) in shell.cmds.iter() {
            print!("{:12}", name);
            println!("  {}", cmd.description());
        }
    }

    fn help(&self) -> String {
        self.description()
    }
}