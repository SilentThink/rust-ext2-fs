use super::Cmd;
use crate::fs::core::*;
use crate::utils::str;

pub struct Rm;

impl Rm {
    fn rm_recursively(fs: &mut Fs, path: &str) {
        if path == "." || path == ".." {
            println!("{}: Can't delete directory . and ..", path)
        }

        match fs.open(path) {
            Ok(fd) => {
                if let Err(e) = fs.rm(fd) {
                    println!("{}: {}", path, e.to_string())
                }
                return;
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Other => {}
                _ => println!("{}: {}", path, e.to_string()),
            },
        }

        let mut names = Vec::new();

        match fs.path_parse(path) {
            Ok(path_res) => match path_res.dir_entry.iter(fs) {
                Ok(iter) => {
                    for item in iter {
                        if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                            if entry.name == ".".into_array().unwrap()
                                || entry.name == "..".into_array().unwrap()
                            {
                                continue;
                            }
                            names.push(str(&entry.name).to_string());
                        }
                    }
                }
                Err(e) => println!("{}: {}", path, e.to_string()),
            },
            Err(e) => println!("{}: {}", path, e.to_string()),
        }

        for to_delete in names {
            Self::rm_recursively(fs, &format!("{}/{}", path, to_delete));
        }

        if let Err(e) = fs.rmdir(path) {
            println!("{}: {}", path, e);
        }
    }
}

impl Cmd for Rm {
    fn description(&self) -> String {
        "Delete file".to_string()
    }

    fn run(&self, crate::shell::Shell { fs, .. }: &mut crate::shell::Shell, argv: &[&str]) {
        let dir = fs.pwd();

        if argv.len() == 0 {
            return;
        }

        // 检查是否有-r选项（递归删除）
        let mut recursive = false;
        // 检查是否有-L选项（跟随链接）
        let mut follow_links = false;
        let mut args = Vec::new();
        
        for &arg in argv {
            if arg == "-L" {
                follow_links = true;
            } else if arg == "-r" {
                recursive = true;
            } else if arg == "-h" {
                println!("{}", self.help());
                return;
            } else {
                args.push(arg);
            }
        }

        // 如果没有指定文件参数，显示帮助
        if args.is_empty() {
            println!("Usage: rm [-r] [-L] <file>...");
            return;
        }

        // 处理递归删除
        if recursive {
            for arg in args {
                Self::rm_recursively(fs, &format!("{}/{}", dir, arg));
            }
            return;
        }

        // 处理普通删除
        for arg in args {
            // 首先尝试检查是否为符号链接
            if !follow_links {
                // 尝试删除符号链接本身
                match fs.path_parse_with_options(arg, false) {
                    Ok(path_res) => {
                        let symlink_type: u8 = FileType::Symlink.into();
                        if path_res.dir_entry.file_type == symlink_type {
                            // 是符号链接，删除链接本身
                            if let Err(e) = fs.rm_symlink(arg) {
                                println!("{}: {}", arg, e);
                            }
                            continue;
                        }
                    },
                    Err(_) => {}
                }
            }
            
            // 不是符号链接或者指定了-L选项，使用常规删除流程
            let mut err = None;
            match fs.open(arg) {
                Ok(fd) => {
                    if let Err(e) = fs.rm(fd) {
                        println!("{}", e.to_string())
                    }
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::Other => {
                        if let Err(e) = fs.rmdir(arg) {
                            err = Some(e.to_string())
                        }
                    }
                    _ => err = Some(e.to_string()),
                },
            }
            if let Some(err) = err {
                println!("{}/{}: {}", dir, arg, err);
            }
        }
    }

    fn help(&self) -> String {
        self.description()
            + r#"
 -h show this message
 -r delete files recursively
 -L follow symbolic links (delete the target instead of the link)"#
    }
}
