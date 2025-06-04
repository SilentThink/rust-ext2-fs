use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Write};

/// 历史命令管理器
pub struct CommandHistory {
    /// 历史命令列表
    commands: VecDeque<String>,
    /// 当前浏览位置
    current_index: Option<usize>,
    /// 最大历史记录数量
    max_size: usize,
    /// 历史文件路径
    history_file: String,
    /// 临时输入缓冲
    temp_input: String,
}

impl CommandHistory {
    /// 创建新的历史命令管理器
    pub fn new(max_size: usize, history_file: &str) -> Self {
        let mut history = CommandHistory {
            commands: VecDeque::with_capacity(max_size),
            current_index: None,
            max_size,
            history_file: history_file.to_string(),
            temp_input: String::new(),
        };
        
        // 从文件加载历史记录
        history.load_from_file();
        history
    }
    
    /// 添加新命令到历史记录
    pub fn add_command(&mut self, command: &str) {
        if command.trim().is_empty() {
            return;
        }
        
        // 避免连续重复的命令
        if let Some(last) = self.commands.back() {
            if last == command {
                return;
            }
        }
        
        // 如果超过最大容量，移除最旧的记录
        if self.commands.len() >= self.max_size {
            self.commands.pop_front();
        }
        
        self.commands.push_back(command.to_string());
        self.current_index = None; // 重置浏览位置
        
        // 保存到文件
        self.save_to_file();
    }
    
    /// 向上浏览历史命令（更早的命令）
    pub fn previous(&mut self, current_input: &str) -> Option<String> {
        if self.commands.is_empty() {
            return None;
        }
        
        // 如果是第一次按上键，保存当前输入
        if self.current_index.is_none() {
            self.temp_input = current_input.to_string();
            self.current_index = Some(self.commands.len() - 1);
        } else if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
            } else {
                // 已经在最早的命令了
                return None;
            }
        }
        
        self.commands.get(self.current_index.unwrap()).cloned()
    }
    
    /// 向下浏览历史命令（更新的命令）
    pub fn next(&mut self) -> Option<String> {
        if self.current_index.is_none() {
            return None; // 没有在浏览历史
        }
        
        if let Some(index) = self.current_index {
            if index < self.commands.len() - 1 {
                self.current_index = Some(index + 1);
                self.commands.get(index).cloned()
            } else {
                // 回到当前输入
                self.current_index = None;
                Some(self.temp_input.clone())
            }
        } else {
            None
        }
    }
    
    /// 重置浏览状态
    pub fn reset_navigation(&mut self) {
        self.current_index = None;
        self.temp_input.clear();
    }
    
    /// 获取所有历史命令
    pub fn get_all(&self) -> Vec<String> {
        self.commands.iter().cloned().collect()
    }
    
    /// 清空历史记录
    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_index = None;
        self.temp_input.clear();
        
        // 清空文件
        if let Ok(mut file) = std::fs::File::create(&self.history_file) {
            let _ = file.write_all(b"");
        }
    }
    
    /// 从文件加载历史记录
    fn load_from_file(&mut self) {
        if let Ok(file) = std::fs::File::open(&self.history_file) {
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                if let Ok(command) = line {
                    if !command.trim().is_empty() && self.commands.len() < self.max_size {
                        self.commands.push_back(command);
                    }
                }
            }
        }
    }
    
    /// 保存历史记录到文件
    fn save_to_file(&self) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.history_file)
        {
            for command in &self.commands {
                let _ = writeln!(file, "{}", command);
            }
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(1000, ".shell_history")
    }
} 