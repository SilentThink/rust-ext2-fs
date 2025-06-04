use crate::fs::Fs;

mod cmd;
mod history;
mod input;
mod completion;

use self::cmd::Cmds;
use self::input::InputManager;
use self::completion::Completion;
use crossterm::style::Stylize;

pub struct Shell {
    pub fs: Fs,
    pub cmds: Cmds,
    input_manager: InputManager,
    completion: Completion,
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

        let cmds = cmd::cmds();
        let completion = Completion::new(&cmds);

        Self {
            fs,
            cmds,
            input_manager: InputManager::new(),
            completion,
        }
    }

    pub fn init_cmds(&mut self) {}

    pub fn run(&mut self) {
        println!(
            "\n {} ",
            " Welcome to simple unix like filesystem. ".on_white().black()
        );
        println!("提示: 使用上下方向键浏览历史命令，Tab键自动补全，Ctrl+L清屏，Ctrl+C退出");
        println!("");

        if !cmd::login::Login::login(&mut self.fs) {
            return;
        }

        let cmds = self.cmds.clone();

        loop {
            let prompt = format!("[{}] ", self.fs.pwd().green());
            
            // 使用新的输入管理器读取输入，支持自动补全
            let input = match self.input_manager.read_line_with_completion(&prompt, &mut self.fs, &self.completion) {
                Ok(input) => input,
                Err(e) => {
                    eprintln!("输入错误: {}", e);
                    continue;
                }
            };
            
            // 检查是否是中断信号（Ctrl+C）
            if input.is_empty() {
                continue;
            }
            
            let mut argv: Vec<&str> = input
                .trim()
                .split(' ')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if argv.len() < 1 {
                continue;
            }

            let cmd_name = argv.remove(0);
            
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
