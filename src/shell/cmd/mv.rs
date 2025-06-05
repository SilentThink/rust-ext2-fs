use crate::fs::*;
use super::*;

pub struct Mv;

impl Mv {
    fn mv(fs: &mut Fs, src: &str, dest: &str) -> Result<()> {
        // 获取源文件/目录的信息
        let src_path = fs.path_parse(src)?;
        let src_type: FileType = src_path.dir_entry.file_type.into();
        let is_src_dir = matches!(src_type, FileType::Dir);
        
        // 检查目标路径是否已存在
        let dest_exists = fs.path_parse(dest).is_ok();
        let dest_is_dir = if dest_exists {
            let dest_path = fs.path_parse(dest)?;
            matches!(dest_path.dir_entry.file_type.into(), FileType::Dir)
        } else {
            false
        };
        
        // 确定最终的目标路径
        let final_dest = if dest_exists && dest_is_dir {
            // 如果目标是现有目录，将源文件/目录移动到该目录下
            let src_name = utils::str(&src_path.dir_entry.name);
            if dest == "/" {
                format!("/{}", src_name)
            } else {
                format!("{}/{}", dest, src_name)
            }
        } else {
            // 否则使用指定的目标路径（重命名或移动到新位置）
            dest.to_string()
        };
        
        // 检查目标路径是否与源路径相同
        if src == final_dest {
            return Ok(()); // 源和目标相同，无需操作
        }
        
        // 检查是否尝试将目录移动到其子目录中
        if is_src_dir && final_dest.starts_with(&format!("{}/", src)) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Cannot move directory into itself"
            ));
        }
        
        // 执行移动操作：先复制，后删除
        Self::copy_recursive(fs, src, &final_dest, is_src_dir)?;
        Self::remove_recursive(fs, src, is_src_dir)?;
        
        Ok(())
    }
    
    fn copy_recursive(fs: &mut Fs, src: &str, dest: &str, is_dir: bool) -> Result<()> {
        if is_dir {
            // 复制目录
            // 创建目标目录
            if let Err(e) = fs.mkdir(dest) {
                return Err(e);
            }
            
            // 获取源目录中的所有项
            let src_path = fs.path_parse(src)?;
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
                        
                        let item_is_dir = matches!(entry.file_type.into(), FileType::Dir);
                        copy_tasks.push((src_item_path, dest_item_path, item_is_dir));
                    }
                }
            }
            
            // 递归复制所有项目
            for (src_path, dest_path, item_is_dir) in copy_tasks {
                Self::copy_recursive(fs, &src_path, &dest_path, item_is_dir)?;
            }
        } else {
            // 复制文件
            if let Err(e) = fs.create(dest) {
                return Err(e);
            }

            let fd_src = fs.open(src)?;
            let fd_dest = fs.open(dest)?;

            loop {
                let mut buf = [0u8; 512];
                let bytes_read = fs.read(fd_src, &mut buf)?;
                if bytes_read == 0 {
                    break;
                }
                fs.write(fd_dest, &buf[0..bytes_read])?;
            }

            fs.close(fd_src)?;
            fs.close(fd_dest)?;
        }
        
        Ok(())
    }
    
    fn remove_recursive(fs: &mut Fs, path: &str, is_dir: bool) -> Result<()> {
        if is_dir {
            // 递归删除目录
            let path_res = fs.path_parse(path)?;
            let mut items_to_delete = Vec::new();
            
            if let Ok(entries) = path_res.dir_entry.iter(fs) {
                for entry in entries {
                    if let DirEntryIterItem::Using(Item { entry, .. }) = entry {
                        let filename = utils::str(&entry.name).to_string();
                        
                        // 跳过 . 和 .. 目录
                        if filename == "." || filename == ".." {
                            continue;
                        }
                        
                        let item_path = if path == "/" {
                            format!("/{}", filename)
                        } else {
                            format!("{}/{}", path, filename)
                        };
                        
                        let item_is_dir = matches!(entry.file_type.into(), FileType::Dir);
                        items_to_delete.push((item_path, item_is_dir));
                    }
                }
            }
            
            // 先删除所有子项
            for (item_path, item_is_dir) in items_to_delete {
                Self::remove_recursive(fs, &item_path, item_is_dir)?;
            }
            
            // 最后删除目录本身
            fs.rmdir(path)?;
        } else {
            // 删除文件
            let fd = fs.open(path)?;
            fs.rm(fd)?;
        }
        
        Ok(())
    }
}

impl Cmd for Mv {
    fn description(&self) -> String {
        "Move/rename files and directories".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.len() < 2 {
            println!("Usage: mv <source> <destination>");
            println!("       mv <source1> <source2> ... <destination_directory>");
            return;
        }
        
        if argv.len() == 2 {
            // 单个源文件/目录
            let src = argv[0];
            let dest = argv[1];
            
            if let Err(e) = Self::mv(&mut shell.fs, src, dest) {
                println!("mv: {}", e);
            }
        } else {
            // 多个源文件/目录，目标必须是目录
            let sources = &argv[..argv.len()-1];
            let dest_dir = argv[argv.len()-1];
            
            // 检查目标是否为目录
            match shell.fs.path_parse(dest_dir) {
                Ok(dest_path) => {
                    if !matches!(dest_path.dir_entry.file_type.into(), FileType::Dir) {
                        println!("mv: target '{}' is not a directory", dest_dir);
                        return;
                    }
                }
                Err(_) => {
                    println!("mv: target directory '{}' does not exist", dest_dir);
                    return;
                }
            }
            
            // 移动每个源文件/目录到目标目录
            for src in sources {
                if let Err(e) = Self::mv(&mut shell.fs, src, dest_dir) {
                    println!("mv: {}: {}", src, e);
                }
            }
        }
    }

    fn help(&self) -> String {
        format!("{}

Usage: mv <source> <destination>
       mv <source1> <source2> ... <destination_directory>

Move or rename files and directories.

Examples:
  mv file1.txt file2.txt              # Rename file1.txt to file2.txt
  mv file.txt /path/to/directory/     # Move file.txt to directory
  mv dir1 dir2                        # Rename directory dir1 to dir2
  mv file1 file2 file3 target_dir/    # Move multiple files to target_dir

The command automatically handles:
- Renaming files and directories
- Moving files/directories between different directories
- Moving multiple files to a target directory
- Recursive moving of directories and their contents

Note: Moving a directory into itself is not allowed.", 
            self.description())
    }
} 