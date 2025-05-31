pub struct Touch;

impl super::Cmd for Touch {
    fn description(&self) -> String {
        "Create a new file".to_string()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        for arg in argv {
            if let Err(e) = fs.create(arg) {
                if let std::io::ErrorKind::AlreadyExists = e.kind() {
                } else {
                    println!("{}: {}", arg, e.to_string())
                }
            }
        }
    }
}
