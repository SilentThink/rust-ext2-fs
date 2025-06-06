// 导入所需的模块
use crate::{utils::str, shell::Shell};
use super::Cmd;

// 定义 Cat 结构体，用于实现文件内容显示功能
pub struct Cat;

impl Cmd for Cat {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Show content of file".into()
    }

    // 实现命令的运行逻辑
    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        // 遍历所有参数（文件路径）
        for arg in argv {
            // 尝试打开文件
            match shell.fs.open(arg) {
                Ok(fd) => {
                    // 文件打开成功，获取文件描述符 fd
                    loop {
                        // 创建缓冲区用于存储读取的数据
                        let mut buf = [0u8; 512];
                        // 尝试从文件中读取数据
                        match shell.fs.read(fd, &mut buf) {
                            Ok(bytes) => {
                                // 如果读取到的字节数为 0，表示已到文件末尾
                                if bytes == 0 {
                                    break;
                                } else {
                                    // 将读取到的字节转换为字符串并打印
                                    print!("{}", str(&buf))
                                }
                            }
                            Err(e) => {
                                // 读取过程中发生错误，打印错误信息
                                println!("{}: {}", arg, e);
                                break;
                            }
                        }
                    }
                    // 关闭文件
                    shell.fs.close(fd).unwrap()
                }
                Err(e) => {
                    // 文件打开失败，打印错误信息
                    println!("{}: {}", arg, e)
                },
            }
        }
    }
}
