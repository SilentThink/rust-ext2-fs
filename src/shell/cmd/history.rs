use super::*;
use crate::shell::Shell;

pub struct History;

impl Cmd for History {
    fn description(&self) -> String {
        "显示命令历史记录".to_string()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        let mut show_numbers = true;
        let mut limit: Option<usize> = None;
        let mut clear_history = false;

        // 解析命令行参数
        let mut i = 0;
        while i < argv.len() {
            match argv[i] {
                "-c" | "--clear" => {
                    clear_history = true;
                }
                "-n" | "--no-numbers" => {
                    show_numbers = false;
                }
                arg if arg.starts_with('-') && arg.len() > 1 => {
                    // 尝试解析 -数字 格式
                    if let Ok(num) = arg[1..].parse::<usize>() {
                        limit = Some(num);
                    } else {
                        println!("history: 未知选项 '{}'", arg);
                        println!("使用 'history -h' 查看帮助");
                        return;
                    }
                }
                arg => {
                    // 尝试解析数字参数
                    if let Ok(num) = arg.parse::<usize>() {
                        limit = Some(num);
                    } else {
                        println!("history: 无效参数 '{}'", arg);
                        return;
                    }
                }
            }
            i += 1;
        }

        if clear_history {
            shell.clear_history();
            println!("历史记录已清除");
            return;
        }

        let history = shell.get_history();
        
        if history.is_empty() {
            println!("没有历史记录");
            return;
        }

        // 应用限制
        let entries: Vec<_> = if let Some(n) = limit {
            if n == 0 {
                return;
            }
            history.iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()
        } else {
            history.iter().collect()
        };

        // 显示历史记录
        if show_numbers {
            let start_num = if let Some(_n) = limit {
                history.len().saturating_sub(entries.len()) + 1
            } else {
                1
            };

            for (i, entry) in entries.iter().enumerate() {
                println!("{:5}  {}", start_num + i, entry);
            }
        } else {
            for entry in entries {
                println!("{}", entry);
            }
        }
    }

    fn help(&self) -> String {
        r#"显示或操作命令历史记录

用法: history [选项] [数字]

选项:
  -c, --clear       清除历史记录
  -n, --no-numbers  不显示行号
  -数字             显示最近的指定条数记录
  
参数:
  数字              显示最近的指定条数记录

示例:
  history           显示所有历史记录
  history 10        显示最近的10条记录
  history -10       显示最近的10条记录
  history -n        显示历史记录但不包含行号
  history -c        清除所有历史记录"#.to_string()
    }
} 