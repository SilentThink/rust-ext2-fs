use super::Cmd;

pub struct Cd;

impl Cmd for Cd {   
    fn description(&self) -> String {
        // 返回命令的描述信息
        "Change work directory".into()
    }

    // 实现命令的运行逻辑
    fn run(&self, shell: &mut crate::shell::Shell, argv: &[&str]) {
        // 获取第一个参数
        match argv.get(0) {
            // 如果参数为空，打印错误信息
            None => println!("Need a filename"),
            // 如果参数不为空，尝试切换工作目录
            Some(path) => {
                // 尝试切换工作目录
                if let Err(msg) = shell.fs.chdir(path) {
                    // 如果切换失败，打印错误信息
                    println!("{}", msg.to_string())
                }
            }
        }
    }

    // 返回命令的帮助信息
    fn help(&self) -> String {
        // 返回命令的描述信息
        self.description()
            // 返回命令的帮助信息
            + r"#
            // 返回命令的帮助信息
        cd <dir>

        -h show help message#"
    }
}
