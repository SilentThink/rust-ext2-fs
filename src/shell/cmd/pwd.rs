use super::Cmd; // 引入上一级模块中的 Cmd 特性

pub struct Pwd; // 定义 Pwd 结构体，用于实现打印当前工作目录的功能

impl Cmd for Pwd {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Print current working directory".into() // 描述信息：打印当前工作目录
    }

    // 定义命令的运行逻辑
    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        // 检查命令行参数
        match argv.get(0) {
            Some(&"-h") => println!("{}", self.help()), // 如果参数是 "-h"，打印帮助信息
            _ => {
                // 否则，打印当前工作目录
                println!("{}", shell.fs.pwd());
            }
        }
    }

    // 返回命令的帮助信息
    fn help(&self) -> String {
        // 帮助信息：描述命令的用法和功能
        self.description() + r#"

pwd
Print the full filename of the current working directory."#
    }
}