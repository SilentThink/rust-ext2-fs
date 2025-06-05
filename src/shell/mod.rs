pub mod cmd;

use std::io::Write;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Config};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;

use self::cmd::Cmds;
use super::fs::Fs;
use super::fs::{DirEntryIterItem, Item};
use super::fs::utils;
use crossterm::style::Stylize;

// 支持命令和文件名补全的补全器
pub struct SimpleCompleter {
    commands: Vec<String>,
}

impl SimpleCompleter {
    fn new(cmds: &Cmds) -> Self {
        let commands = cmds.keys().map(|&s| s.to_string()).collect();
        Self { commands }
    }

    // 获取当前目录下的文件名和目录名
    fn get_current_dir_files(fs: &Fs) -> Vec<String> {
        let mut files = Vec::new();
        
        // 使用当前目录解析当前路径
        if let Ok(parsed_path) = fs.path_parse("") {
            if let Ok(dir_entries) = parsed_path.dir_entry.iter(fs) {
                for item in dir_entries {
                    if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                        let filename = utils::str(&entry.name).to_string();
                        // 跳过 "." 和 ".." 条目
                        if filename != "." && filename != ".." {
                            files.push(filename);
                        }
                    }
                }
            }
        }
        
        files
    }
}

// 自定义Helper结构体，包含文件系统引用
pub struct FileSystemHelper {
    completer: SimpleCompleter,
    fs_ref: *const Fs, // 使用原始指针来避免所有权问题
}

impl FileSystemHelper {
    fn new(cmds: &Cmds, fs: &Fs) -> Self {
        Self {
            completer: SimpleCompleter::new(cmds),
            fs_ref: fs as *const Fs,
        }
    }
}

impl Completer for FileSystemHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut completions = Vec::new();
        
        // 获取光标前的内容
        let before_cursor = &line[..pos];
        let words: Vec<&str> = before_cursor.split_whitespace().collect();
        
        // 确定是否是第一个词（命令），还是参数
        if words.is_empty() || (words.len() == 1 && !before_cursor.ends_with(' ')) {
            // 补全命令
            let prefix = if words.is_empty() { "" } else { words[0] };
            let start_pos = pos - prefix.len();
            
            for cmd in &self.completer.commands {
                if cmd.starts_with(prefix) {
                    completions.push(Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    });
                }
            }
            
            Ok((start_pos, completions))
        } else {
            // 补全文件名或目录名
            // 获取当前正在输入的词（最后一个未完成的参数）
            let current_word = if before_cursor.ends_with(' ') {
                ""
            } else {
                // 找到最后一个空格后的内容
                before_cursor.split_whitespace().last().unwrap_or("")
            };
            
            let start_pos = pos - current_word.len();
            
            // 安全地访问文件系统引用
            unsafe {
                let fs = &*self.fs_ref;
                let files = SimpleCompleter::get_current_dir_files(fs);
                
                for filename in files {
                    if filename.starts_with(current_word) {
                        completions.push(Pair {
                            display: filename.clone(),
                            replacement: filename,
                        });
                    }
                }
            }
            
            Ok((start_pos, completions))
        }
    }
}

impl Hinter for FileSystemHelper {
    type Hint = String;
}

impl Highlighter for FileSystemHelper {}

impl Validator for FileSystemHelper {}

impl Helper for FileSystemHelper {}

pub struct Shell {
    pub fs: Fs,
    pub cmds: Cmds,
    pub history: Vec<String>,
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
            history: Vec::new(),
        }
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn add_to_history(&mut self, command: String) {
        // 避免添加重复的连续命令
        if let Some(last) = self.history.last() {
            if last == &command {
                return;
            }
        }
        
        // 限制历史记录大小，保留最近的1000条
        if self.history.len() >= 1000 {
            self.history.remove(0);
        }
        
        self.history.push(command);
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
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(rustyline::CompletionType::List)
            .build();
            
        let mut rl = match Editor::with_config(config) {
            Ok(editor) => editor,
            Err(err) => {
                println!("Failed to create readline editor: {}", err);
                // 如果创建失败，回退到简单的输入方式
                self.run_simple();
                return;
            }
        };

        // 设置补全器，支持命令和文件名补全
        let helper = FileSystemHelper::new(&self.cmds, &self.fs);
        rl.set_helper(Some(helper));

        // 尝试加载历史记录文件
        let history_file = "fs_history.txt";
        if rl.load_history(history_file).is_err() {
            // 如果历史文件不存在或加载失败，忽略错误
            println!("No previous history found or failed to load history.");
        }

        let cmds = self.cmds.clone();

        println!("提示：使用上下方向键浏览历史命令，Tab键自动补全命令，Ctrl+C退出");
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

                    // 将命令添加到我们自己的历史记录中
                    self.add_to_history(input.to_string());

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
            let input = input.trim();
            
            // 如果输入为空，跳过
            if input.is_empty() {
                continue;
            }
            
            // 将命令添加到历史记录中
            self.add_to_history(input.to_string());
            
            let mut argv: Vec<&str> = input
                .split(' ')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if argv.len() < 1 {
                continue;
            }

            let cmd_name = argv.remove(0);

            // 处理退出命令
            if cmd_name == "exit" {
                break;
            }

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

