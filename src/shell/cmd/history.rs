use super::*;

pub struct History;

impl Cmd for History {
    fn description(&self) -> String {
        "显示命令历史记录".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        // 检查参数
        if argv.is_empty() {
            // 显示所有历史记录
            self.show_history(shell);
            return;
        }

        match argv[0] {
            "-c" | "--clear" => {
                // 清空历史记录
                self.clear_history(shell);
            }
            "-n" => {
                // 显示最近n条记录
                if argv.len() > 1 {
                    if let Ok(n) = argv[1].parse::<usize>() {
                        self.show_recent_history(shell, n);
                    } else {
                        println!("错误：无效的数字参数");
                    }
                } else {
                    println!("用法: history -n <数量>");
                }
            }
            "-h" | "--help" => {
                println!("{}", self.help());
            }
            _ => {
                println!("未知选项：{}", argv[0]);
                println!("{}", self.help());
            }
        }
    }

    fn help(&self) -> String {
        format!("{}\n{}", 
            self.description(),
            r#"用法:
  history         显示所有历史记录
  history -c      清空历史记录
  history -n <N>  显示最近N条记录
  history -h      显示此帮助信息
  
注意: 在命令行中使用上下方向键可以浏览历史命令"#)
    }
}

impl History {
    fn show_history(&self, _shell: &Shell) {
        let commands = self.load_history_from_file();
        
        if commands.is_empty() {
            println!("没有历史记录");
            return;
        }
        
        for (i, cmd) in commands.iter().enumerate() {
            println!("{}: {}", i + 1, cmd);
        }
    }
    
    fn show_recent_history(&self, _shell: &Shell, n: usize) {
        let commands = self.load_history_from_file();
        
        if commands.is_empty() {
            println!("没有历史记录");
            return;
        }
        
        let start = if commands.len() > n { commands.len() - n } else { 0 };
        
        println!("显示最近 {} 条历史记录:", n);
        for (i, cmd) in commands.iter().enumerate().skip(start) {
            println!("{}: {}", i + 1, cmd);
        }
    }
    
    fn clear_history(&self, _shell: &Shell) {
        // 删除历史文件
        if std::fs::remove_file(".shell_history").is_ok() {
            println!("历史记录已清空");
        } else {
            println!("清空历史记录失败或历史文件不存在");
        }
    }
    
    /// 从文件加载历史记录
    fn load_history_from_file(&self) -> Vec<String> {
        if let Ok(content) = std::fs::read_to_string(".shell_history") {
            content.lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }
} 