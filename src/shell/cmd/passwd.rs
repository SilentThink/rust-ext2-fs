use std::io::{stdin, stdout, Write}; // 引入标准库中的输入输出相关模块

use crate::fs::ByteArray; // 引入文件系统模块中的 ByteArray 类型

use super::*; // 引入上一级模块中的所有内容，包括 Cmd 特性和 Shell 结构体等

pub struct Passwd; // 定义 Passwd 结构体，用于实现更改用户密码的功能

impl Cmd for Passwd {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Change user's password".to_string() // 描述信息：更改用户的密码
    }

    // 定义命令的运行逻辑
    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        // 获取当前用户
        let mut user = shell.fs.current_user();

        // 如果提供了用户名参数
        if argv.len() == 1 {
            let mut to_search = None; // 初始化一个变量，用于存储找到的用户索引
            // 遍历用户列表，查找指定用户名
            for (i, u) in shell.fs.fs_desc().users.iter().enumerate() {
                if argv[0].into_array().unwrap() == u.name { // 将用户名参数转换为数组并与用户列表中的名字进行比较
                    to_search = Some(i); // 如果找到匹配的用户，记录其索引
                    break;
                }
            }
            // 根据查找结果更新用户变量
            match to_search {
                Some(i) => user = i, // 如果找到了用户，更新 user 为找到的用户索引
                None => {
                    println!("User {} is not exists.", argv[0]); // 如果未找到用户，打印错误信息并退出
                    return;
                }
            }
        }
        // 如果参数数量过多
        if argv.len() > 1 {
            println!("Too many arguments"); // 打印错误信息
            return;
        }

        // 初始化两个字符串变量，用于存储用户输入的新密码
        let mut password1 = String::new();
        let mut password2 = String::new();

        // 提示用户输入新密码
        print!("New password: ");
        stdout().flush().unwrap(); // 刷新标准输出，确保提示信息立即显示
        stdin().read_line(&mut password1).unwrap(); // 读取用户输入的新密码

        // 提示用户再次输入密码
        print!("Press again:  ");
        stdout().flush().unwrap(); // 刷新标准输出
        stdin().read_line(&mut password2).unwrap(); // 读取用户再次输入的密码

        // 检查两次输入的密码是否一致
        if password1 != password2 {
            println!("Confirm failed"); // 如果不一致，打印确认失败信息并退出
            return;
        }

        // 调用文件系统的 passwd 方法更改用户密码
        if let Err(e) = shell.fs.passwd(user, &password1.trim()) {
            println!("{}", e); // 如果更改密码时发生错误，打印错误信息
        };
    }

    // 返回命令的帮助信息
    fn help(&self) -> String {
        self.description() + "\n passwd [username]" // 帮助信息：描述命令的用法
    }
}