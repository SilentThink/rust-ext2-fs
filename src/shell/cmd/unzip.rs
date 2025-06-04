use crate::{shell::Shell, fs::Result, fs::FileType};
use super::Cmd;

pub struct Unzip;

impl Unzip {
    fn decompress_data(compressed_data: &[u8], original_size: usize) -> Result<Vec<u8>> {
        let mut decompressed = Vec::with_capacity(original_size);
        let mut i = 0;
        
        while i < compressed_data.len() && decompressed.len() < original_size {
            if compressed_data[i] == 0xFF {
                if i + 1 < compressed_data.len() {
                    if compressed_data[i + 1] == 0x00 {
                        // 转义的0xFF字节
                        decompressed.push(0xFF);
                        i += 2;
                    } else {
                        // 压缩格式：0xFF + count + byte
                        if i + 2 < compressed_data.len() {
                            let count = compressed_data[i + 1];
                            let byte_value = compressed_data[i + 2];
                            
                            for _ in 0..count {
                                if decompressed.len() < original_size {
                                    decompressed.push(byte_value);
                                }
                            }
                            i += 3;
                        } else {
                            // 数据不完整
                            break;
                        }
                    }
                } else {
                    // 数据不完整
                    break;
                }
            } else {
                // 普通字节
                decompressed.push(compressed_data[i]);
                i += 1;
            }
        }
        
        Ok(decompressed)
    }

    fn parse_archive(archive_data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
        let mut files = Vec::new();
        let mut offset = 0;
        
        // 读取文件数量
        if archive_data.len() < 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid archive format: missing file count"
            ));
        }
        
        let file_count = u32::from_le_bytes([
            archive_data[0], archive_data[1], archive_data[2], archive_data[3]
        ]) as usize;
        offset += 4;
        
        // 解析每个文件
        for _ in 0..file_count {
            // 读取路径长度
            if offset + 4 > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete path length"
                ));
            }
            
            let path_len = u32::from_le_bytes([
                archive_data[offset], archive_data[offset + 1],
                archive_data[offset + 2], archive_data[offset + 3]
            ]) as usize;
            offset += 4;
            
            // 读取路径
            if offset + path_len > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete path"
                ));
            }
            
            let path = String::from_utf8(
                archive_data[offset..offset + path_len].to_vec()
            ).map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid archive format: invalid path encoding"
            ))?;
            offset += path_len;
            
            // 读取原始大小和压缩大小
            if offset + 8 > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete size information"
                ));
            }
            
            let original_size = u32::from_le_bytes([
                archive_data[offset], archive_data[offset + 1],
                archive_data[offset + 2], archive_data[offset + 3]
            ]) as usize;
            offset += 4;
            
            let compressed_size = u32::from_le_bytes([
                archive_data[offset], archive_data[offset + 1],
                archive_data[offset + 2], archive_data[offset + 3]
            ]) as usize;
            offset += 4;
            
            // 读取压缩数据
            if offset + compressed_size > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete compressed data"
                ));
            }
            
            let compressed_data = &archive_data[offset..offset + compressed_size];
            offset += compressed_size;
            
            // 解压缩数据
            let decompressed_data = Self::decompress_data(compressed_data, original_size)?;
            
            files.push((path, decompressed_data));
        }
        
        Ok(files)
    }

    fn create_directory_structure(shell: &mut Shell, output_dir: &str, files: &[(String, Vec<u8>)]) -> Result<()> {
        // 首先创建所有需要的目录
        let mut directories = std::collections::HashSet::new();
        
        for (path, _) in files {
            // 如果路径以 '/' 结尾，说明这是一个目录
            if path.ends_with('/') {
                let dir_path = if output_dir == "." {
                    path.trim_end_matches('/').to_string()
                } else {
                    format!("{}/{}", output_dir, path.trim_end_matches('/'))
                };
                directories.insert(dir_path);
            } else {
                // 对于文件，需要创建其父目录
                if let Some(parent) = path.rfind('/') {
                    let parent_path = &path[..parent];
                    let full_parent_path = if output_dir == "." {
                        parent_path.to_string()
                    } else {
                        format!("{}/{}", output_dir, parent_path)
                    };
                    
                    // 添加所有父目录路径
                    let mut current_path = String::new();
                    for component in full_parent_path.split('/') {
                        if !component.is_empty() {
                            if current_path.is_empty() {
                                current_path = component.to_string();
                            } else {
                                current_path = format!("{}/{}", current_path, component);
                            }
                            directories.insert(current_path.clone());
                        }
                    }
                }
            }
        }
        
        // 按路径长度排序，确保先创建父目录
        let mut sorted_dirs: Vec<_> = directories.into_iter().collect();
        sorted_dirs.sort_by_key(|path| path.matches('/').count());
        
        // 创建目录
        for dir_path in sorted_dirs {
            if let Err(e) = shell.fs.mkdir(&dir_path) {
                // 如果目录已存在，忽略错误
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    println!("Warning: Failed to create directory {}: {}", dir_path, e);
                }
            }
        }
        
        Ok(())
    }

    fn extract_files(shell: &mut Shell, output_dir: &str, files: Vec<(String, Vec<u8>)>) -> Result<()> {
        // 首先创建目录结构
        Self::create_directory_structure(shell, output_dir, &files)?;
        
        // 然后提取文件
        for (path, data) in files {
            // 跳过目录条目（以 '/' 结尾）
            if path.ends_with('/') {
                continue;
            }
            
            let full_path = if output_dir == "." {
                path
            } else {
                format!("{}/{}", output_dir, path)
            };
            
            // 创建文件
            if let Err(e) = shell.fs.create(&full_path) {
                println!("Warning: Failed to create file {}: {}", full_path, e);
                continue;
            }
            
            // 写入文件内容
            let fd = match shell.fs.open(&full_path) {
                Ok(fd) => fd,
                Err(e) => {
                    println!("Warning: Failed to open file {}: {}", full_path, e);
                    continue;
                }
            };
            
            if let Err(e) = shell.fs.write(fd, &data) {
                println!("Warning: Failed to write to file {}: {}", full_path, e);
            }
            
            shell.fs.close(fd).ok();
        }
        
        Ok(())
    }
}

impl Cmd for Unzip {
    fn description(&self) -> String {
        "Decompress files and directories compressed with zip command".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.len() < 1 || argv.len() > 2 {
            println!("Usage: unzip <archive_file> [output_directory]");
            return;
        }
        
        let archive_file = argv[0];
        let output_dir = if argv.len() == 2 {
            argv[1]
        } else {
            "."
        };
        
        // 读取压缩文件
        let fd_src = match shell.fs.open(archive_file) {
            Ok(fd) => fd,
            Err(e) => {
                println!("Error opening archive file {}: {}", archive_file, e);
                return;
            }
        };
        
        // 读取所有数据
        let mut archive_data = Vec::new();
        loop {
            let mut buf = [0u8; 512];
            match shell.fs.read(fd_src, &mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    archive_data.extend_from_slice(&buf[..bytes_read]);
                }
                Err(e) => {
                    println!("Error reading archive data: {}", e);
                    shell.fs.close(fd_src).ok();
                    return;
                }
            }
        }
        shell.fs.close(fd_src).ok();
        
        // 尝试解析新格式的档案
        match Self::parse_archive(&archive_data) {
            Ok(files) => {
                // 新格式：包含多个文件和目录的档案
                println!("Extracting {} items from archive...", files.len());
                
                // 如果输出目录不是当前目录且不存在，则创建它
                if output_dir != "." {
                    if let Err(e) = shell.fs.mkdir(output_dir) {
                        if e.kind() != std::io::ErrorKind::AlreadyExists {
                            println!("Error creating output directory {}: {}", output_dir, e);
                            return;
                        }
                    }
                }
                
                // 提取文件和目录
                if let Err(e) = Self::extract_files(shell, output_dir, files) {
                    println!("Error extracting files: {}", e);
                    return;
                }
                
                println!("Successfully extracted archive to '{}'", output_dir);
            },
            Err(_) => {
                // 尝试旧格式：单个文件压缩
                if archive_data.len() < 4 {
                    println!("Error: Invalid archive file format");
                    return;
                }
                
                // 读取原始文件大小（前4字节）
                let original_size = u32::from_le_bytes([
                    archive_data[0], archive_data[1], 
                    archive_data[2], archive_data[3]
                ]) as usize;
                
                // 解压缩数据
                let decompressed_data = match Self::decompress_data(&archive_data[4..], original_size) {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Error decompressing data: {}", e);
                        return;
                    }
                };
                
                // 验证解压后的大小
                if decompressed_data.len() != original_size {
                    println!(
                        "Warning: Decompressed size ({} bytes) doesn't match expected size ({} bytes)",
                        decompressed_data.len(),
                        original_size
                    );
                }
                
                // 对于旧格式，如果没有指定输出文件名，使用默认名称
                let output_file = if argv.len() == 2 {
                    argv[1].to_string()
                } else {
                    // 移除.zip扩展名作为输出文件名
                    if archive_file.ends_with(".zip") {
                        archive_file[..archive_file.len()-4].to_string()
                    } else {
                        format!("{}.out", archive_file)
                    }
                };
                
                // 创建输出文件
                if let Err(e) = shell.fs.create(&output_file) {
                    println!("Error creating output file {}: {}", output_file, e);
                    return;
                }
                
                let fd_dest = match shell.fs.open(&output_file) {
                    Ok(fd) => fd,
                    Err(e) => {
                        println!("Error opening output file {}: {}", output_file, e);
                        return;
                    }
                };
                
                // 写入解压缩数据
                if let Err(e) = shell.fs.write(fd_dest, &decompressed_data) {
                    println!("Error writing decompressed data: {}", e);
                    shell.fs.close(fd_dest).ok();
                    return;
                }
                
                shell.fs.close(fd_dest).ok();
                
                println!("Successfully decompressed '{}' to '{}' ({} bytes)", 
                        archive_file, output_file, decompressed_data.len());
            }
        }
    }

    fn help(&self) -> String {
        format!(
            "{}

Usage: unzip <archive_file> [output_directory]

Decompress files and directories that were compressed using the zip command.
Supports both new multi-file archive format and legacy single-file format.

Arguments:
  <archive_file>      The compressed archive file to extract
  [output_directory]  Directory to extract to (optional, defaults to current directory)

Examples:
  unzip data.zip              # Extract to current directory
  unzip backup.zip ./restore  # Extract to restore directory
  unzip file.zip output.txt   # Extract single file (legacy format)",
            self.description()
        )
    }
}