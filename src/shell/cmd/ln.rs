use super::Cmd;
use crate::fs::core::*;

pub struct Ln;

impl Cmd for Ln {
    fn description(&self) -> String {
        "Create hard links".to_string()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        if argv.len() < 2 {
            println!("Usage: ln <target> <link_name>");
            return;
        }

        if argv[0] == "-h" {
            println!("{}", self.help());
            return;
        }

        let target = argv[0];
        let link_name = argv[1];

        match shell.fs.link(target, link_name) {
            Ok(_) => println!("Created hard link '{}' -> '{}'", link_name, target),
            Err(e) => println!("Error creating hard link: {}", e),
        }
    }

    fn help(&self) -> String {
        self.description() + r#"
Usage: ln <target> <link_name>

Creates a hard link named <link_name> that points to <target>.
Hard links share the same inode as the original file.
When a file has multiple hard links, the file is only deleted when all links are removed.
Note: Hard links to directories are not allowed.
"#
    }
} 