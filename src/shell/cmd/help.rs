use super::*; // 引入上一级模块中的所有内容，包括 Cmd 特性和 Shell 结构体等

// 定义一个 Help 结构体，用于实现显示所有命令及其描述的功能
pub struct Help;

// 为 Help 结构体实现 Cmd 特性
impl Cmd for Help {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Show all commands and its description".into() // 描述信息：显示所有命令及其描述
    }

    // 定义命令的运行逻辑
    fn run(&self, shell: &mut Shell, _: &[&str]) {
        // 遍历 Shell 中注册的所有命令
        for (&name, cmd) in shell.cmds.iter() {
            // 打印命令名称，宽度为12个字符，右对齐
            print!("{:12}", name);
            // 打印命令的描述信息
            println!("  {}", cmd.description());
        }
    }

    // 返回命令的帮助信息，这里直接复用描述信息
    fn help(&self) -> String {
        self.description() // 返回与 description 方法相同的内容
    }
}