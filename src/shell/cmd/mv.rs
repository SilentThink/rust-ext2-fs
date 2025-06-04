use crate::fs::*;
use super::*;

pub struct Mv;

impl Mv {
    fn mv(fs: &mut Fs, src: &str, dest: &str, recursive: bool) -> Result<()> {
        // 获取源文件/目录的信息
        let src_path = fs.path_parse(src)?;
        let src_type: FileType = src_path.dir_entry.file_type.into();
        let is_dir = matches!(src_type, FileType::Dir);
        
        // 如果是目录但没有指定递归移动，则报错
        if is_dir && !recursive {
            println!("{}: 是一个目录，需要使用 -r 选项", src);
            return Err(Error::new(ErrorKind::InvalidInput, "不能移动目录，使用 -r 选项"));
        }
        
        // 检查源路径和目标路径是否相同
        if src == dest {
            println!("源路径和目标路径相同，无需移动");
            return Ok(());
        }
        
        // 处理目标路径，如果目标是目录，则构造完整的目标路径
        let final_dest = Self::resolve_dest_path(fs, src, dest)?;
        
        // 检查是否试图将目录移动到自己的子目录中
        if is_dir && final_dest.starts_with(&format!("{}/", src)) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "不能将目录移动到自己的子目录中"
            ));
        }
        
        // 先复制文件/目录
        if let Err(e) = Self::cp(fs, src, &final_dest, recursive) {
            return Err(e);
        }
        
        // 复制成功后删除源文件/目录
        if let Err(e) = Self::rm(fs, src, recursive) {
            // 如果删除失败，尝试清理已复制的目标文件
            println!("移动失败，正在清理目标文件...");
            let _ = Self::rm(fs, &final_dest, recursive);
            return Err(e);
        }
        
        Ok(())
    }
    
    // 解析目标路径，处理目标是目录的情况
    fn resolve_dest_path(fs: &mut Fs, src: &str, dest: &str) -> Result<String> {
        // 如果目标以 / 结尾，说明是要移动到目录内
        if dest.ends_with('/') {
            let dir_path = &dest[..dest.len()-1]; // 去掉末尾的 /
            
            // 检查目录是否存在
            if let Ok(path_res) = fs.path_parse(dir_path) {
                if matches!(FileType::from(path_res.dir_entry.file_type), FileType::Dir) {
                    // 提取源文件名
                    let src_filename = src.rsplit('/').next().unwrap_or(src);
                    return Ok(format!("{}/{}", dir_path, src_filename));
                }
            }
            return Err(Error::new(ErrorKind::NotFound, "目标目录不存在"));
        }
        
        // 检查目标是否是已存在的目录
        if let Ok(path_res) = fs.path_parse(dest) {
            if matches!(FileType::from(path_res.dir_entry.file_type), FileType::Dir) {
                // 提取源文件名
                let src_filename = src.rsplit('/').next().unwrap_or(src);
                return Ok(format!("{}/{}", dest, src_filename));
            }
        }
        
        // 目标不是目录，直接返回
        Ok(dest.to_string())
    }
    
    fn cp(fs: &mut Fs, src: &str, dest: &str, recursive: bool) -> Result<()> {
        // 获取源文件/目录的信息
        let src_path = fs.path_parse(src)?;
        let src_type: FileType = src_path.dir_entry.file_type.into();
        let is_dir = matches!(src_type, FileType::Dir);
        
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
                    return Err(e);
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
    
    fn rm(fs: &mut Fs, path: &str, recursive: bool) -> Result<()> {
        if path == "." || path == ".." {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "无法删除当前目录或父目录"
            ));
        }

        // 如果是递归删除
        if recursive {
            Self::rm_recursively(fs, path);
            return Ok(());
        }

        // 尝试删除文件
        match fs.open(path) {
            Ok(fd) => {
                if let Err(e) = fs.rm(fd) {
                    return Err(e);
                }
                return Ok(());
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Other => {
                    // 可能是目录，尝试删除空目录
                    if let Err(e) = fs.rmdir(path) {
                        return Err(e);
                    }
                    return Ok(());
                }
                _ => return Err(e),
            },
        }
    }
    
    fn rm_recursively(fs: &mut Fs, path: &str) {
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
                            names.push(utils::str(&entry.name).to_string());
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

impl Cmd for Mv {
    fn description(&self) -> String {
        "移动文件或目录".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        // 检查参数
        if argv.is_empty() {
            println!("用法: mv [-r] 源文件/目录 目标文件/目录");
            return;
        }
        
        let mut recursive = false;
        let mut src = "";
        let mut dest = "";
        
        // 解析参数
        if argv[0] == "-r" {
            recursive = true;
            if argv.len() < 3 {
                println!("用法: mv -r 源目录 目标目录");
                return;
            }
            src = argv[1];
            dest = argv[2];
        } else {
            if argv.len() < 2 {
                println!("用法: mv 源文件 目标文件");
                return;
            }
            src = argv[0];
            dest = argv[1];
        }
        
        if let Err(e) = Self::mv(&mut shell.fs, src, dest, recursive) {
            println!("移动失败: {}", e)
        } else {
            println!("成功移动 {} 到 {}", src, dest);
        }
    }

    fn help(&self) -> String {
        self.description() + "\n用法: mv [-r] 源文件/目录 目标文件/目录\n选项:\n  -r  递归移动目录及其内容"
    }
} 