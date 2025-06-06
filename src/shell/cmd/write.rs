use crate::fs::*; // 引入文件系统模块，包含Fs等类型和函数

pub struct Write; // 定义Write结构体，用于实现向文件写入内容的功能

impl Write {
    // 主逻辑函数，用于向指定路径的文件写入内容
    fn main(fs: &mut Fs, path: &str) -> Result<()> {
        // 打开指定路径的文件
        let fd = fs.open(path)?;

        // 将文件指针移动到文件开头
        let res = fs.cut(fd, 0);
        if res.is_err() {
            // 如果移动失败，关闭文件并返回错误
            fs.close(fd)?;
            return res;
        }

        // 提示用户输入文件内容
        println!("Input content of file now, Press Ctrl+D will save the file.");
        println!("-----------------------------------------------------------");
        println!("");

        // 循环读取用户输入
        loop {
            let mut content = String::new(); // 初始化一个字符串变量用于存储用户输入
            // 从标准输入读取一行内容
            if std::io::stdin().read_line(&mut content).unwrap() == 0 {
                break; // 如果读取到的字节数为0，表示用户按下Ctrl+D，退出循环
            }
            // 将用户输入的内容写入文件
            fs.write(fd, content.as_bytes())?;
        }

        // 关闭文件
        fs.close(fd)?;

        // 提示用户文件已保存
        println!("");
        println!("-----------------------------------------------------------");
        println!("{} Saved.", path);
        Ok(())
    }
}

// 为Write结构体实现Cmd特性
impl super::Cmd for Write {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Write something into file".to_string() // 描述信息：向文件写入内容
    }

    // 定义命令的运行逻辑
    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        // 根据命令行参数数量进行处理
        match argv.len() {
            0 => println!("Should receive a file to write"), // 如果没有参数，提示需要指定文件
            1 => {
                // 如果有一个参数，调用main函数执行写入操作
                if let Err(e) = Self::main(fs, argv[0]) {
                    // 如果发生错误，打印错误信息
                    println!("{}: {}", argv[0], e);
                }
            }
            _ => println!("Too many arguments"), // 如果参数过多，提示错误
        }
    }
}