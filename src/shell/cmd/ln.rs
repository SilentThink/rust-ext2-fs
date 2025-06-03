use super::Cmd;

pub struct Ln;

impl Cmd for Ln {
    fn description(&self) -> String {
        "Create links (hard links by default, symbolic links with -s option)".to_string()
    }

    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        if argv.len() < 2 {
            println!("Usage: ln [-s] <target> <link_name>");
            return;
        }

        if argv[0] == "-h" {
            println!("{}", self.help());
            return;
        }

        let mut is_symbolic = false;
        let mut target_idx = 0;

        // 检查是否有-s选项
        if argv[0] == "-s" {
            is_symbolic = true;
            target_idx = 1;
            
            if argv.len() < 3 {
                println!("Usage: ln -s <target> <link_name>");
                return;
            }
        }

        let target = argv[target_idx];
        let link_name = argv[target_idx + 1];

        if is_symbolic {
            // 创建软链接
            match shell.fs.symlink(target, link_name) {
                Ok(_) => println!("Created symbolic link '{}' -> '{}'", link_name, target),
                Err(e) => println!("Error creating symbolic link: {}", e),
            }
        } else {
            // 创建硬链接
            match shell.fs.link(target, link_name) {
                Ok(_) => println!("Created hard link '{}' -> '{}'", link_name, target),
                Err(e) => println!("Error creating hard link: {}", e),
            }
        }
    }

    fn help(&self) -> String {
        self.description() + r#"
Usage: ln [-s] <target> <link_name>

Creates a link named <link_name> that points to <target>.

Options:
  -s    Create a symbolic link instead of a hard link

Hard links:
  - Share the same inode as the original file
  - When a file has multiple hard links, the file is only deleted when all links are removed
  - Cannot link to directories
  - Cannot span across different filesystems

Symbolic links (symlinks):
  - Point to the pathname of another file
  - Can link to directories
  - Can span across different filesystems
  - Become invalid if the target file is moved or deleted
"#
    }
} 