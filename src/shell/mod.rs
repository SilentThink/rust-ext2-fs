pub mod cmd;

use std::io::Write;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use self::cmd::Cmds;
use super::fs::Fs;
use crossterm::style::Stylize;

pub struct Shell {
    pub fs: Fs,
    pub cmds: Cmds,
}

impl Shell {
    pub fn new() -> Self {
        let fs = match Fs::init() {
            Ok(fs) => fs,
            Err(err) => {
                println!("Err: {}", err.to_string().red());
                println!("Format disk? [y/n]");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                match input.trim() {
                    "Y" | "y" => Fs::format().unwrap(),
                    _ => std::process::exit(1),
                }
            }
        };

        Self {
            fs,
            cmds: cmd::cmds(),
        }
    }

    pub fn init_cmds(&mut self) {}

    pub fn run(&mut self) {
        println!(
            "\n {} ",
            " Welcome to simple unix like filesystem. ".on_white().black()
        );
        println!("");

        if !cmd::login::Login::login(&mut self.fs) {
            return;
        }

        // 创建rustyline编辑器实例
        let mut rl = match DefaultEditor::new() {
            Ok(editor) => editor,
            Err(err) => {
                println!("Failed to create readline editor: {}", err);
                // 如果创建失败，回退到简单的输入方式
                self.run_simple();
                return;
            }
        };

        // 尝试加载历史记录文件
        let history_file = "fs_history.txt";
        if rl.load_history(history_file).is_err() {
            // 如果历史文件不存在或加载失败，忽略错误
            println!("No previous history found or failed to load history.");
        }

        let cmds = self.cmds.clone();

        println!("提示：使用上下方向键浏览历史命令，Ctrl+C退出");
        println!("");

        loop {
            let prompt = format!("[{}] ", self.fs.pwd().green());
            
            match rl.readline(&prompt) {
                Ok(input) => {
                    let input = input.trim();
                    
                    // 如果输入为空，跳过
                    if input.is_empty() {
                        continue;
                    }

                    // 添加到历史记录
                    if let Err(err) = rl.add_history_entry(input) {
                        eprintln!("Failed to add history entry: {}", err);
                    }

                    // 解析命令
                    let mut argv: Vec<&str> = input
                        .split(' ')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if argv.is_empty() {
                        continue;
                    }

                    let cmd_name = argv.remove(0);

                    // 处理特殊命令
                    match cmd_name {
                        "exit" => {
                            // 保存历史记录并退出
                            if let Err(err) = rl.save_history(history_file) {
                                eprintln!("Failed to save history: {}", err);
                            }
                            break;
                        }
                        _ => {
                            // 执行普通命令
                            match cmds.get(cmd_name) {
                                Some(cmd) => match argv.contains(&"-h") {
                                    true => println!("{}", cmd.help()),
                                    false => cmd.run(self, &argv),
                                },
                                None => println!(
                                    "{}",
                                    "Command not found. Press `help` or `?` to see all command".red()
                                ),
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // 用户按下Ctrl+C
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // 用户按下Ctrl+D
                    println!("Ctrl+D pressed, exiting...");
                    break;
                }
                Err(err) => {
                    // 其他错误
                    println!("Readline error: {}", err);
                    break;
                }
            }
        }

        // 保存历史记录
        if let Err(err) = rl.save_history(history_file) {
            eprintln!("Failed to save history: {}", err);
        } else {
            println!("History saved to {}", history_file);
        }

        println!("Bye.");
    }

    // 简单输入方式的回退实现（用于rustyline初始化失败的情况）
    fn run_simple(&mut self) {
        let cmds = self.cmds.clone();

        loop {
            print!("[{}] ", self.fs.pwd().green());
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let mut argv: Vec<&str> = input
                .trim()
                .split(' ')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if argv.len() < 1 {
                continue;
            }

            match cmds.get(argv.remove(0)) {
                Some(cmd) => match argv.contains(&"-h") {
                    true => println!("{}", cmd.help()),
                    false => cmd.run(self, &argv),
                },
                None => println!(
                    "{}",
                    "Command not found. Press `help` or `?` to see all command".red()
                ),
            }
        }
    }
}
