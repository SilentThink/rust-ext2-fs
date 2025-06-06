use crate::{utils::str, shell::Shell};
use super::Cmd;

pub struct Grep;

impl Cmd for Grep {
    // 返回命令的描述信息
    fn description(&self) -> String {
        // 返回命令的描述信息
        "Search for patterns in files".into()
    }

    // 返回命令的帮助信息
    fn help(&self) -> String {
        // 返回命令的帮助信息
        "grep [OPTIONS] PATTERN [FILE...]
Search for PATTERN in each FILE or standard input.

OPTIONS:
  -i, --ignore-case     Ignore case distinctions
  -n, --line-number     Print line numbers with output lines
  -v, --invert-match    Invert the sense of matching, to select non-matching lines
  -c, --count           Print only a count of matching lines per file
  -l, --files-with-matches  Print only names of files with matching lines
  -H, --with-filename   Print the file name for each match
  -h, --no-filename     Suppress the file name prefix on output

EXAMPLES:
  grep hello file.txt         Search for 'hello' in file.txt
  grep -i Hello file.txt      Case-insensitive search
  grep -n pattern file.txt    Show line numbers
  grep -v pattern file.txt    Show lines that don't match
  grep -c pattern file.txt    Count matching lines".into()
    }

    // 实现命令的运行逻辑
    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        // 如果参数为空，打印错误信息并返回
        if argv.is_empty() {
            // 打印错误信息
            println!("grep: missing pattern");
            println!("Try 'grep --help' for more information.");
            return;
        }

        // 是否忽略大小写
        let mut ignore_case = false;
        // 是否显示行号
        let mut line_number = false;
        // 是否反转匹配
        let mut invert_match = false;
        // 是否只显示匹配的行数
        let mut count_only = false;
        // 是否只显示文件名
        let mut files_with_matches = false;
        // 是否显示文件名
        let mut with_filename = false;
        // 是否不显示文件名
        let mut no_filename = false;
        // 模式
        let mut pattern = "";
        // 文件
        let mut files = Vec::new();
        // 索引
        let mut i = 0;

        // 遍历参数
        while i < argv.len() {
            // 获取参数
            let arg = argv[i];
            
            // 如果参数为 --help，显示帮助信息并返回
            if arg == "--help" {
                println!("{}", self.help());
                return;
            } else if arg.starts_with("--") {
                // 处理长选项
                match arg {
                    "--ignore-case" => ignore_case = true,
                    "--line-number" => line_number = true,
                    "--invert-match" => invert_match = true,
                    "--count" => count_only = true,
                    "--files-with-matches" => files_with_matches = true,
                    "--with-filename" => with_filename = true,
                    "--no-filename" => no_filename = true,
                    _ => {
                        if pattern.is_empty() {
                            pattern = arg;
                        } else {
                            files.push(arg);
                        }
                    }
                }
            } else if arg.starts_with('-') && arg.len() > 1 {
                // 处理短选项（包括组合选项，如 -in）
                let chars: Vec<char> = arg.chars().skip(1).collect(); // 跳过 '-'
                for ch in chars {
                    match ch {
                        'i' => ignore_case = true,
                        'n' => line_number = true,
                        'v' => invert_match = true,
                        'c' => count_only = true,
                        'l' => files_with_matches = true,
                        'H' => with_filename = true,
                        'h' => no_filename = true,
                        _ => {
                            println!("grep: invalid option -- '{}'", ch);
                            println!("Try 'grep --help' for more information.");
                            return;
                        }
                    }
                }
            } else {
                // 常规参数（模式或文件名）
                if pattern.is_empty() {
                    pattern = arg;
                } else {
                    files.push(arg);
                }
            }
            i += 1;
        }

        // 如果模式为空，打印错误信息并返回
        if pattern.is_empty() {
            println!("grep: missing pattern");
            return;
        }

        // 如果文件为空，打印错误信息并返回
        if files.is_empty() {
            println!("grep: no files specified");
            return;
        }

        // 是否显示文件名
        let multiple_files = files.len() > 1;
        // 是否显示文件名
        let show_filename = (with_filename || multiple_files) && !no_filename;

        // 遍历文件
        for file_path in files {
            match shell.fs.open(file_path) {
                Ok(fd) => {
                    // 创建一个空数组用于存储文件内容
                    let mut content = Vec::new();
                    // 创建一个缓冲区用于存储文件内容
                    let mut buf = [0u8; 512];
                    
                    // 读取文件内容
                    loop {
                        match shell.fs.read(fd, &mut buf) {
                            Ok(bytes) => {
                                // 如果读取到的字节数为 0，表示已到文件末尾
                                if bytes == 0 {
                                    break;
                                }
                                // 将读取到的字节添加到文件内容中
                                content.extend_from_slice(&buf[..bytes]);
                            }
                            Err(e) => {
                                // 如果读取文件内容失败，打印错误信息并返回
                                println!("grep: {}: {}", file_path, e);
                                break;
                            }
                        }
                    }
                    shell.fs.close(fd).unwrap();

                    // 将文件内容转换为字符串
                    let content_str = str(&content);
                    // 将文件内容按行分割
                    let lines: Vec<&str> = content_str.lines().collect();
                    
                    // 匹配行数
                    let mut match_count = 0;
                    // 是否匹配
                    let mut has_matches = false;

                    // 遍历行
                    for (line_num, line) in lines.iter().enumerate() {
                        // 如果忽略大小写
                        let line_matches = if ignore_case {
                            // 如果忽略大小写，将行转换为小写并检查是否包含模式
                            line.to_lowercase().contains(&pattern.to_lowercase())
                        } else {
                            // 如果不忽略大小写，检查是否包含模式
                            line.contains(pattern)
                        };

                        // 是否打印
                        let should_print = if invert_match { !line_matches } else { line_matches };

                        // 如果需要打印
                        if should_print {
                            // 匹配行数加 1
                            match_count += 1;
                            // 是否匹配
                            has_matches = true;

                            // 如果只显示匹配的行数或只显示文件名
                            if count_only || files_with_matches {
                                // 不打印单个行
                                continue;
                            }

                            // 创建一个空字符串用于存储输出
                            let mut output = String::new();
                            
                            // 如果显示文件名
                            if show_filename {
                                output.push_str(file_path);
                                output.push(':');
                            }
                            
                            // 如果显示行号
                            if line_number {
                                output.push_str(&format!("{}:", line_num + 1));
                            }
                            
                            // 将行添加到输出中
                            output.push_str(line);
                            // 打印输出
                            println!("{}", output);
                        }
                    }

                    // 处理特殊输出模式
                    if count_only {
                        // 如果显示文件名
                        if show_filename {
                            // 打印文件名和匹配行数
                            println!("{}:{}", file_path, match_count);
                        } else {
                            // 打印匹配行数
                            println!("{}", match_count);
                        }
                    } else if files_with_matches && has_matches {
                        // 打印文件名
                        println!("{}", file_path);
                    }
                }
                Err(e) => {
                    // 如果打开文件失败，打印错误信息
                    println!("grep: {}: {}", file_path, e);
                }
            }
        }
    }
} 