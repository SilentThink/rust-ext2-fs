use std::io::{stdout, Write}; // 引入标准库中的 stdout 和 Write 特性，用于操作标准输出

use crossterm::terminal as ter; // 引入 crossterm 的终端操作模块，并重命名为 ter
use crossterm::cursor as cur; // 引入 crossterm 的光标操作模块，并重命名为 cur
use crossterm::QueueableCommand; // 引入 crossterm 的 QueueableCommand 特性，用于队列化终端命令

use super::*; // 引入上一级模块中的所有内容，包括 Cmd 特性和 Shell 结构体等

// 定义一个 Clear 结构体，用于实现清屏功能
pub struct Clear;

// 为 Clear 结构体实现 Cmd 特性
impl Cmd for Clear {
    // 返回命令的描述信息
    fn description(&self) -> String {
        "Clear Screen".into() // 描述信息：清屏
    }

    // 定义命令的运行逻辑
    fn run(&self, _shell: &mut Shell, _argv: &[&str]) {
        // 获取标准输出的句柄
        let mut stdout = stdout();

        // 队列化清屏命令，清除整个终端屏幕
        stdout.queue(ter::Clear(ter::ClearType::All)).unwrap();
        // 将光标移动到屏幕的左上角（0, 0）
        stdout.queue(cur::MoveTo(0, 0)).unwrap();
        // 刷新标准输出，执行队列中的命令
        stdout.flush().unwrap_or_default();
    }
}