use super::Cmd;

pub struct RmDir;

impl Cmd for RmDir {
    fn description(&self) -> String {
        "Delete directory (use -r for recursive deletion)".to_string()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        if argv.is_empty() {
            println!("rmdir: missing operand");
            println!("Usage: rmdir [-r] directory...");
            return;
        }

        let mut recursive = false;
        let mut start_index = 0;

        // 检查是否有 -r 参数
        if argv.len() > 0 && argv[0] == "-r" {
            recursive = true;
            start_index = 1;
        }

        if start_index >= argv.len() {
            println!("rmdir: missing operand");
            println!("Usage: rmdir [-r] directory...");
            return;
        }

        // 删除指定的目录
        for name in &argv[start_index..] {
            let result = if recursive {
                fs.rmdir_recursive(name)
            } else {
                fs.rmdir(name)
            };

            if let Err(e) = result {
                println!("{}: {}", name, e);
            }
        }
    }
}
