use crate::fs::*;
use super::*;

pub struct Cp;

impl Cp {
    fn cp(fs: &mut Fs, src: &str, dest: &str, recursive: bool) -> Result<()> {
        // 获取源文件/目录的信息
        let src_path = fs.path_parse(src)?;
        let src_type: FileType = src_path.dir_entry.file_type.into();
        let is_dir = matches!(src_type, FileType::Dir);
        
        // 如果是目录但没有指定递归复制，则报错
        if is_dir && !recursive {
            println!("{}: 是一个目录，需要使用 -r 选项", src);
            return Err(Error::new(ErrorKind::InvalidInput, "不能复制目录，使用 -r 选项"));
        }
        
        // 如果源是目录且指定了递归复制
        if is_dir && recursive {
            // 创建目标目录
            if let Err(e) = fs.mkdir(dest) {
                println!("无法创建目录 {}: {}", dest, e);
                return Err(e);
            }
            
            // 获取源目录中的所有项
            let mut copy_tasks = Vec::new();
            
            if let Ok(entries) = src_path.dir_entry.iter(fs) {
                for entry in entries {
                    if let DirEntryIterItem::Using(Item { entry, .. }) = entry {
                        let filename = utils::str(&entry.name).to_string();
                        
                        // 跳过 . 和 .. 目录
                        if filename == "." || filename == ".." {
                            continue;
                        }
                        
                        // 构建源路径和目标路径
                        let src_item_path = if src == "/" {
                            format!("/{}", filename)
                        } else {
                            format!("{}/{}", src, filename)
                        };
                        
                        let dest_item_path = if dest == "/" {
                            format!("/{}", filename)
                        } else {
                            format!("{}/{}", dest, filename)
                        };
                        
                        // 收集需要复制的项
                        copy_tasks.push((src_item_path, dest_item_path));
                    }
                }
            }
            
            // 在不可变借用结束后执行复制操作
            for (src_path, dest_path) in copy_tasks {
                if let Err(e) = Self::cp(fs, &src_path, &dest_path, recursive) {
                    println!("复制 {} 到 {} 失败: {}", src_path, dest_path, e);
                }
            }
            
            return Ok(());
        }
        
        // 复制文件
        if let Err(e) = fs.create(dest) {
            println!("{}: {}", dest, e);
            return Err(e);
        };

        let fd_src = fs.open(src)?;
        let fd_dest = fs.open(dest)?;

        loop {
            let mut buf = [0u8; 512]; // 使用更大的缓冲区提高效率
            let bytes_read = fs.read(fd_src, &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            fs.write(fd_dest, &buf[0..bytes_read])?;
        }

        fs.close(fd_src)?;
        fs.close(fd_dest)?;
        Ok(())
    }
}

impl Cmd for Cp {
    fn description(&self) -> String {
        "复制文件或目录".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        // 检查参数
        if argv.is_empty() {
            println!("用法: cp [-r] 源文件/目录 目标文件/目录");
            return;
        }
        
        let mut recursive = false;
        let src;
        let dest;
        
        // 解析参数
        if argv[0] == "-r" {
            recursive = true;
            if argv.len() < 3 {
                println!("用法: cp -r 源目录 目标目录");
                return;
            }
            src = argv[1];
            dest = argv[2];
        } else {
            if argv.len() < 2 {
                println!("用法: cp 源文件 目标文件");
                return;
            }
            src = argv[0];
            dest = argv[1];
        }
        
        if let Err(e) = Self::cp(&mut shell.fs, src, dest, recursive) {
            println!("复制失败: {}", e)
        }
    }

    fn help(&self) -> String {
        self.description() + "\n用法: cp [-r] 源文件/目录 目标文件/目录"
    }
}
