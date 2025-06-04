use crate::fs::{Fs, FileType, DirEntryIterItem, Item};
use crate::utils;
use std::collections::BTreeMap;

/// 自动补全管理器
pub struct Completion {
    /// 所有可用的命令
    commands: Vec<String>,
}

impl Completion {
    /// 创建新的自动补全管理器
    pub fn new(command_map: &BTreeMap<&'static str, Box<dyn super::cmd::Cmd + Send + Sync>>) -> Self {
        let commands = command_map.keys().map(|&cmd| cmd.to_string()).collect();
        Self { commands }
    }

    /// 获取给定输入的所有可能补全
    pub fn get_completions(&self, fs: &mut Fs, input: &str, cursor_pos: usize) -> Vec<String> {
        // 分析输入，确定需要补全的类型
        let (completion_type, partial) = self.analyze_input(input, cursor_pos);
        
        match completion_type {
            CompletionType::Command => self.complete_command(&partial),
            CompletionType::FilePath { current_dir } => self.complete_file_path(fs, &partial, &current_dir),
        }
    }

    /// 分析输入类型，确定需要补全什么
    fn analyze_input(&self, input: &str, cursor_pos: usize) -> (CompletionType, String) {
        let input_up_to_cursor = &input[..cursor_pos.min(input.len())];
        let words: Vec<&str> = input_up_to_cursor.split_whitespace().collect();
        
        if words.is_empty() || (words.len() == 1 && !input_up_to_cursor.ends_with(' ')) {
            // 补全命令
            let partial = words.first().map_or("", |v| v).to_string();
            (CompletionType::Command, partial)
        } else {
            // 补全文件路径
            let partial = if input_up_to_cursor.ends_with(' ') {
                String::new()
            } else {
                words.last().map_or("", |v| v).to_string()
            };
            
            // 确定当前目录
            let current_dir = if partial.contains('/') {
                let path_parts: Vec<&str> = partial.rsplitn(2, '/').collect();
                if path_parts.len() == 2 {
                    path_parts[1].to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            (CompletionType::FilePath { current_dir }, partial)
        }
    }

    /// 补全命令名
    fn complete_command(&self, partial: &str) -> Vec<String> {
        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(partial))
            .cloned()
            .collect()
    }

    /// 补全文件路径
    fn complete_file_path(&self, fs: &mut Fs, partial: &str, current_dir: &str) -> Vec<String> {
        let mut completions = Vec::new();
        
        // 确定要搜索的目录
        let search_dir = if current_dir.is_empty() {
            None // 当前目录
        } else {
            Some(current_dir)
        };
        
        // 确定文件名前缀
        let filename_prefix = if partial.contains('/') {
            partial.split('/').last().unwrap_or("")
        } else {
            partial
        };

        // 获取目录内容
        if let Ok(parse_result) = fs.path_parse(search_dir.unwrap_or("")) {
            if let Ok(dir_iter) = parse_result.dir_entry.iter(fs) {
                for item in dir_iter {
                    if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                        let filename = utils::str(&entry.name);
                        
                        // 跳过当前目录和父目录
                        if filename == "." || filename == ".." {
                            continue;
                        }
                        
                        // 检查是否匹配前缀
                        if filename.starts_with(filename_prefix) {
                            let full_name = if current_dir.is_empty() {
                                filename.to_string()
                            } else {
                                format!("{}/{}", current_dir, filename)
                            };
                            
                            // 如果是目录，添加斜杠
                            let file_type: FileType = entry.file_type.into();
                            let completion = if file_type == FileType::Dir {
                                format!("{}/", full_name)
                            } else {
                                full_name
                            };
                            
                            completions.push(completion);
                        }
                    }
                }
            }
        }
        
        completions.sort();
        completions
    }

    /// 应用补全到输入字符串
    pub fn apply_completion(&self, input: &str, cursor_pos: usize, completion: &str) -> (String, usize) {
        let input_up_to_cursor = &input[..cursor_pos.min(input.len())];
        let input_after_cursor = &input[cursor_pos.min(input.len())..];
        
        let words: Vec<&str> = input_up_to_cursor.split_whitespace().collect();
        
        if words.is_empty() || (words.len() == 1 && !input_up_to_cursor.ends_with(' ')) {
            // 替换命令
            let new_input = format!("{} {}", completion, input_after_cursor);
            let new_cursor_pos = completion.len() + 1;
            (new_input, new_cursor_pos)
        } else {
            // 替换文件路径
            let partial = if input_up_to_cursor.ends_with(' ') {
                ""
            } else {
                words.last().map_or("", |v| v)
            };
            
            let prefix_len = input_up_to_cursor.len() - partial.len();
            let new_input = format!("{}{}{}", &input[..prefix_len], completion, input_after_cursor);
            let new_cursor_pos = prefix_len + completion.len();
            
            (new_input, new_cursor_pos)
        }
    }

    /// 计算公共前缀
    pub fn common_prefix(completions: &[String]) -> String {
        if completions.is_empty() {
            return String::new();
        }
        
        if completions.len() == 1 {
            return completions[0].clone();
        }
        
        let first = &completions[0];
        let mut common = String::new();
        
        for (i, ch) in first.chars().enumerate() {
            if completions.iter().all(|comp| {
                comp.chars().nth(i).map_or(false, |c| c == ch)
            }) {
                common.push(ch);
            } else {
                break;
            }
        }
        
        common
    }
}

/// 补全类型
#[derive(Debug)]
enum CompletionType {
    /// 补全命令名
    Command,
    /// 补全文件路径
    FilePath { current_dir: String },
} 