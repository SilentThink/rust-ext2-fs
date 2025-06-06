use crate::shell::Shell;

use super::Cmd;

pub struct Mkdir;

impl Cmd for Mkdir {
    // 返回命令的描述信息
    fn description(&self) -> String {
        // 返回命令的描述信息
        "Create directory".into()
    }

    // 实现命令的运行逻辑
    fn run(&self, Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        // 遍历参数
        for dir in argv {
            // 如果创建目录失败，打印错误信息
            if let Err(e) = fs.mkdir(dir) {
                println!("{}: {}", dir, e.to_string())
            }
        }
    }

    // 返回命令的帮助信息
    fn help(&self) -> String {
        // 返回命令的描述信息
        self.description()
            // 返回命令的帮助信息
            + r#"

        // 返回命令的帮助信息
  eg: mkdir dir1 dir2 dir3"#
    }
}
