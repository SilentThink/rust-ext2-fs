use crossterm::{
    cursor::{MoveLeft, MoveRight, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};

use super::history::CommandHistory;
use super::completion::Completion;
use crate::fs::Fs;

/// 输入管理器，处理键盘输入和命令行编辑
pub struct InputManager {
    /// 历史命令管理器
    history: CommandHistory,
    /// 当前输入缓冲区
    buffer: String,
    /// 光标位置
    cursor_pos: usize,
    /// 提示符
    prompt: String,
    /// 当前补全候选项
    completions: Vec<String>,
    /// 当前补全索引
    completion_index: Option<usize>,
}

impl InputManager {
    /// 创建新的输入管理器
    pub fn new() -> Self {
        Self {
            history: CommandHistory::default(),
            buffer: String::new(),
            cursor_pos: 0,
            prompt: String::new(),
            completions: Vec::new(),
            completion_index: None,
        }
    }
    
    /// 读取一行输入，支持历史命令浏览和基本编辑
    pub fn read_line(&mut self, prompt: &str) -> io::Result<String> {
        self.prompt = prompt.to_string();
        self.buffer.clear();
        self.cursor_pos = 0;
        self.completions.clear();
        self.completion_index = None;
        
        // 启用原始模式以捕获键盘事件
        terminal::enable_raw_mode()?;
        
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let result = loop {
            if let Event::Key(key_event) = event::read()? {
                match self.handle_key_event(key_event, None, None)? {
                    InputResult::Continue => continue,
                    InputResult::Submit(line) => {
                        println!(); // 换行
                        
                        // 添加到历史记录
                        if !line.trim().is_empty() {
                            self.history.add_command(&line);
                        }
                        
                        break Ok(line);
                    }
                    InputResult::Interrupt => {
                        println!(); // 换行
                        break Ok(String::new());
                    }
                }
            }
        };
        
        // 确保恢复正常模式
        terminal::disable_raw_mode()?;
        result
    }

    /// 读取一行输入，支持自动补全
    pub fn read_line_with_completion(&mut self, prompt: &str, fs: &mut Fs, completion: &Completion) -> io::Result<String> {
        self.prompt = prompt.to_string();
        self.buffer.clear();
        self.cursor_pos = 0;
        self.completions.clear();
        self.completion_index = None;
        
        // 启用原始模式以捕获键盘事件
        terminal::enable_raw_mode()?;
        
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let result = loop {
            if let Event::Key(key_event) = event::read()? {
                match self.handle_key_event(key_event, Some(fs), Some(completion))? {
                    InputResult::Continue => continue,
                    InputResult::Submit(line) => {
                        println!(); // 换行
                        
                        // 添加到历史记录
                        if !line.trim().is_empty() {
                            self.history.add_command(&line);
                        }
                        
                        break Ok(line);
                    }
                    InputResult::Interrupt => {
                        println!(); // 换行
                        break Ok(String::new());
                    }
                }
            }
        };
        
        // 确保恢复正常模式
        terminal::disable_raw_mode()?;
        result
    }
    
    /// 处理键盘事件
    fn handle_key_event(&mut self, key_event: KeyEvent, fs: Option<&mut Fs>, completion: Option<&Completion>) -> io::Result<InputResult> {
        // 如果不是Tab键，清除补全状态
        if key_event.code != KeyCode::Tab {
            self.completions.clear();
            self.completion_index = None;
        }

        match key_event.code {
            // 回车键 - 提交输入
            KeyCode::Enter => {
                self.history.reset_navigation();
                Ok(InputResult::Submit(self.buffer.clone()))
            }
            
            // Ctrl+C - 中断
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                Ok(InputResult::Interrupt)
            }
            
            // Ctrl+L - 清屏
            KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                execute!(io::stdout(), Clear(ClearType::All), MoveToColumn(0))?;
                self.redraw_line()?;
                Ok(InputResult::Continue)
            }

            // Tab键 - 自动补全
            KeyCode::Tab => {
                if let (Some(fs), Some(completion)) = (fs, completion) {
                    self.handle_tab_completion(fs, completion)?;
                }
                Ok(InputResult::Continue)
            }
            
            // 上方向键 - 历史命令向上
            KeyCode::Up => {
                if let Some(cmd) = self.history.previous(&self.buffer) {
                    self.replace_buffer(&cmd)?;
                }
                Ok(InputResult::Continue)
            }
            
            // 下方向键 - 历史命令向下
            KeyCode::Down => {
                if let Some(cmd) = self.history.next() {
                    self.replace_buffer(&cmd)?;
                } else {
                    // 如果没有更多历史，清空当前行
                    self.replace_buffer("")?;
                }
                Ok(InputResult::Continue)
            }
            
            // 左方向键 - 光标左移
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    execute!(io::stdout(), MoveLeft(1))?;
                }
                Ok(InputResult::Continue)
            }
            
            // 右方向键 - 光标右移
            KeyCode::Right => {
                if self.cursor_pos < self.buffer.len() {
                    self.cursor_pos += 1;
                    execute!(io::stdout(), MoveRight(1))?;
                }
                Ok(InputResult::Continue)
            }
            
            // Home键 - 光标移到行首
            KeyCode::Home => {
                if self.cursor_pos > 0 {
                    execute!(io::stdout(), MoveLeft(self.cursor_pos as u16))?;
                    self.cursor_pos = 0;
                }
                Ok(InputResult::Continue)
            }
            
            // End键 - 光标移到行尾
            KeyCode::End => {
                let move_count = self.buffer.len() - self.cursor_pos;
                if move_count > 0 {
                    execute!(io::stdout(), MoveRight(move_count as u16))?;
                    self.cursor_pos = self.buffer.len();
                }
                Ok(InputResult::Continue)
            }
            
            // 退格键
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    
                    // 先移动光标到正确位置
                    execute!(io::stdout(), MoveLeft(1))?;
                    // 然后重绘从光标位置到行尾
                    self.redraw_from_cursor()?;
                }
                Ok(InputResult::Continue)
            }
            
            // Delete键
            KeyCode::Delete => {
                if self.cursor_pos < self.buffer.len() {
                    self.buffer.remove(self.cursor_pos);
                    self.redraw_from_cursor()?;
                }
                Ok(InputResult::Continue)
            }
            
            // 普通字符输入
            KeyCode::Char(c) => {
                self.buffer.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                
                // 如果光标在行尾，直接打印字符
                if self.cursor_pos == self.buffer.len() {
                    print!("{}", c);
                    io::stdout().flush()?;
                } else {
                    // 否则需要重绘从光标位置开始的部分
                    self.redraw_from_cursor()?;
                }
                
                Ok(InputResult::Continue)
            }
            
            // 其他键忽略
            _ => Ok(InputResult::Continue),
        }
    }

    /// 处理Tab键自动补全
    fn handle_tab_completion(&mut self, fs: &mut Fs, completion: &Completion) -> io::Result<()> {
        if self.completions.is_empty() {
            // 首次按Tab，获取补全候选项
            self.completions = completion.get_completions(fs, &self.buffer, self.cursor_pos);
            
            if self.completions.is_empty() {
                return Ok(()); // 没有可补全的内容
            }
            
            if self.completions.len() == 1 {
                // 只有一个候选项，直接补全
                let (new_buffer, new_cursor_pos) = completion.apply_completion(
                    &self.buffer, 
                    self.cursor_pos, 
                    &self.completions[0]
                );
                self.buffer = new_buffer;
                self.cursor_pos = new_cursor_pos;
                self.redraw_line()?;
                self.completions.clear();
                return Ok(());
            } else {
                // 多个候选项，尝试补全公共前缀
                let common = Completion::common_prefix(&self.completions);
                let current_partial = self.get_current_partial();
                
                if common.len() > current_partial.len() {
                    // 有公共前缀可以补全
                    let (new_buffer, new_cursor_pos) = completion.apply_completion(
                        &self.buffer, 
                        self.cursor_pos, 
                        &common
                    );
                    self.buffer = new_buffer;
                    self.cursor_pos = new_cursor_pos;
                    self.redraw_line()?;
                } else {
                    // 显示所有候选项
                    self.show_completions()?;
                }
                self.completion_index = Some(0);
            }
        } else {
            // 再次按Tab，循环显示候选项
            if let Some(ref mut index) = self.completion_index {
                *index = (*index + 1) % self.completions.len();
            } else {
                self.completion_index = Some(0);
            }
            
            if let Some(index) = self.completion_index {
                let selected = &self.completions[index];
                let (new_buffer, new_cursor_pos) = completion.apply_completion(
                    &self.buffer, 
                    self.cursor_pos, 
                    selected
                );
                self.buffer = new_buffer;
                self.cursor_pos = new_cursor_pos;
                self.redraw_line()?;
            }
        }
        
        Ok(())
    }

    /// 显示补全候选项
    fn show_completions(&self) -> io::Result<()> {
        println!(); // 换行
        
        // 按列显示候选项
        let max_width = self.completions.iter().map(|s| s.len()).max().unwrap_or(0);
        let cols = 80 / (max_width + 2).max(1);
        
        for (i, completion) in self.completions.iter().enumerate() {
            if i > 0 && i % cols == 0 {
                println!();
            }
            print!("{:<width$}  ", completion, width = max_width);
        }
        println!();
        
        // 重新显示提示符和当前输入
        self.redraw_line()?;
        
        Ok(())
    }

    /// 获取当前正在补全的部分
    fn get_current_partial(&self) -> String {
        let input_up_to_cursor = &self.buffer[..self.cursor_pos.min(self.buffer.len())];
        let words: Vec<&str> = input_up_to_cursor.split_whitespace().collect();
        
        if words.is_empty() || (words.len() == 1 && !input_up_to_cursor.ends_with(' ')) {
            words.first().map_or("", |v| v).to_string()
        } else {
            let partial = if input_up_to_cursor.ends_with(' ') {
                ""
            } else {
                words.last().map_or("", |v| v)
            };
            partial.to_string()
        }
    }
    
    /// 替换当前缓冲区内容
    fn replace_buffer(&mut self, new_content: &str) -> io::Result<()> {
        // 移动到提示符后面并清空行
        execute!(io::stdout(), MoveToColumn(self.prompt.len() as u16))?;
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        
        // 更新缓冲区
        self.buffer = new_content.to_string();
        self.cursor_pos = self.buffer.len();
        
        // 打印新内容
        print!("{}", self.buffer);
        io::stdout().flush()?;
        
        Ok(())
    }
    
    /// 从光标位置重绘到行尾
    fn redraw_from_cursor(&mut self) -> io::Result<()> {
        // 清除从光标到行尾
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        
        // 打印从光标位置到行尾的内容
        let remaining = &self.buffer[self.cursor_pos..];
        print!("{}", remaining);
        
        // 将光标移回正确位置
        if !remaining.is_empty() {
            execute!(io::stdout(), MoveLeft(remaining.len() as u16))?;
        }
        
        io::stdout().flush()?;
        Ok(())
    }
    
    /// 重绘整行
    fn redraw_line(&self) -> io::Result<()> {
        print!("{}{}", self.prompt, self.buffer);
        
        // 将光标移动到正确位置
        let cursor_offset = self.buffer.len() - self.cursor_pos;
        if cursor_offset > 0 {
            execute!(io::stdout(), MoveLeft(cursor_offset as u16))?;
        }
        
        io::stdout().flush()?;
        Ok(())
    }
    
    /// 获取历史命令管理器的引用
    pub fn history(&self) -> &CommandHistory {
        &self.history
    }
    
    /// 获取历史命令管理器的可变引用
    pub fn history_mut(&mut self) -> &mut CommandHistory {
        &mut self.history
    }
}

/// 输入处理结果
#[derive(Debug)]
enum InputResult {
    Continue,
    Submit(String),
    Interrupt,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
} 