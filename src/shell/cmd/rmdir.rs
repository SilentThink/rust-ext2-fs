use super::Cmd;

pub struct RmDir;

impl Cmd for RmDir {
    fn description(&self) -> String {
        "Delete empty directory".to_string()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        for name in argv {
            if let Err(e) = fs.rmdir(name) {
                println!("{}: {}", name, e);
            }
        }
    }
}
