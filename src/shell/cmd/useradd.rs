use super::Cmd;

pub struct Useradd;

impl Cmd for Useradd {
    fn description(&self) -> String {
        "Add new user".into()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        if argv.len() != 2 {
            println!("Need tow arguments. username and password");
            return;
        }

        match shell.fs.useradd(argv[0], argv[1]) {
            Err(e) => println!("{}", e),
            Ok(_) => println!("Added user {}, password: {}", argv[0], argv[1]),
        }
    }

    fn help(&self) -> String {
        self.description() + "\n  useradd [username] [password]"
    }
}
