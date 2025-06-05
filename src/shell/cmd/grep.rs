use crate::{utils::str, shell::Shell};
use super::Cmd;

pub struct Grep;

impl Cmd for Grep {
    fn description(&self) -> String {
        "Search for patterns in files".into()
    }

    fn help(&self) -> String {
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

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.is_empty() {
            println!("grep: missing pattern");
            println!("Try 'grep --help' for more information.");
            return;
        }

        let mut ignore_case = false;
        let mut line_number = false;
        let mut invert_match = false;
        let mut count_only = false;
        let mut files_with_matches = false;
        let mut with_filename = false;
        let mut no_filename = false;
        let mut pattern = "";
        let mut files = Vec::new();
        let mut i = 0;

        // Parse arguments
        while i < argv.len() {
            let arg = argv[i];
            
            if arg == "--help" {
                println!("{}", self.help());
                return;
            } else if arg.starts_with("--") {
                // Handle long options
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
                // Handle short options (including combined ones like -in)
                let chars: Vec<char> = arg.chars().skip(1).collect(); // Skip the '-'
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
                // Regular argument (pattern or filename)
                if pattern.is_empty() {
                    pattern = arg;
                } else {
                    files.push(arg);
                }
            }
            i += 1;
        }

        if pattern.is_empty() {
            println!("grep: missing pattern");
            return;
        }

        // If no files specified, we would read from stdin, but for simplicity we'll require files
        if files.is_empty() {
            println!("grep: no files specified");
            return;
        }

        let multiple_files = files.len() > 1;
        let show_filename = (with_filename || multiple_files) && !no_filename;

        for file_path in files {
            match shell.fs.open(file_path) {
                Ok(fd) => {
                    let mut content = Vec::new();
                    let mut buf = [0u8; 512];
                    
                    // Read entire file content
                    loop {
                        match shell.fs.read(fd, &mut buf) {
                            Ok(bytes) => {
                                if bytes == 0 {
                                    break;
                                }
                                content.extend_from_slice(&buf[..bytes]);
                            }
                            Err(e) => {
                                println!("grep: {}: {}", file_path, e);
                                break;
                            }
                        }
                    }
                    shell.fs.close(fd).unwrap();

                    // Convert to string and process lines
                    let content_str = str(&content);
                    let lines: Vec<&str> = content_str.lines().collect();
                    
                    let mut match_count = 0;
                    let mut has_matches = false;

                    for (line_num, line) in lines.iter().enumerate() {
                        let line_matches = if ignore_case {
                            line.to_lowercase().contains(&pattern.to_lowercase())
                        } else {
                            line.contains(pattern)
                        };

                        let should_print = if invert_match { !line_matches } else { line_matches };

                        if should_print {
                            match_count += 1;
                            has_matches = true;

                            if count_only || files_with_matches {
                                // Don't print individual lines for these options
                                continue;
                            }

                            let mut output = String::new();
                            
                            if show_filename {
                                output.push_str(file_path);
                                output.push(':');
                            }
                            
                            if line_number {
                                output.push_str(&format!("{}:", line_num + 1));
                            }
                            
                            output.push_str(line);
                            println!("{}", output);
                        }
                    }

                    // Handle special output modes
                    if count_only {
                        if show_filename {
                            println!("{}:{}", file_path, match_count);
                        } else {
                            println!("{}", match_count);
                        }
                    } else if files_with_matches && has_matches {
                        println!("{}", file_path);
                    }
                }
                Err(e) => {
                    println!("grep: {}: {}", file_path, e);
                }
            }
        }
    }
} 