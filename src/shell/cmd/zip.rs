use crate::{shell::Shell, fs::Result, fs::{FileType, DirEntryIterItem, Item}};
use super::Cmd;

pub struct Zip;

impl Zip {
    fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
        // 简单的RLE (Run-Length Encoding) 压缩算法
        let mut compressed = Vec::new();
        
        if data.is_empty() {
            return Ok(compressed);
        }
        
        let mut i = 0;
        while i < data.len() {
            let current_byte = data[i];
            let mut count = 1u8;
            
            // 计算连续相同字节的数量
            while i + (count as usize) < data.len() && 
                  data[i + (count as usize)] == current_byte && 
                  count < 255 {
                count += 1;
            }
            
            // 如果连续字节数大于3，使用压缩格式
            if count >= 3 {
                compressed.push(0xFF); // 压缩标记
                compressed.push(count);
                compressed.push(current_byte);
            } else {
                // 否则直接存储原始数据
                for _ in 0..count {
                    // 如果原始数据是0xFF，需要转义
                    if current_byte == 0xFF {
                        compressed.push(0xFF);
                        compressed.push(0x00); // 转义标记
                    }
                    compressed.push(current_byte);
                }
            }
            
            i += count as usize;
        }
        
        Ok(compressed)
    }

    fn collect_files_recursive(shell: &mut Shell, path: &str, base_path: &str) -> Result<Vec<(String, Vec<u8>)>> {
        let mut files = Vec::new();
        
        // 获取路径信息
        let path_info = shell.fs.path_parse(path)?;
        
        // 检查是否为目录
        if matches!(path_info.dir_entry.file_type.into(), FileType::Dir) {
            // 收集目录中的所有条目信息
            let mut entries = Vec::new();
            for item in path_info.dir_entry.iter(&shell.fs)? {
                if let DirEntryIterItem::Using(Item { entry, .. }) = item {
                    let entry_name = crate::fs::utils::str(&entry.name);
                    
                    // 跳过 "." 和 ".." 目录
                    if entry_name == "." || entry_name == ".." {
                        continue;
                    }
                    
                    entries.push((entry_name.to_string(), entry.file_type.into()));
                }
            }
            
            // 处理收集到的条目
            for (entry_name, file_type) in entries {
                let full_path = if path == "." {
                    entry_name.clone()
                } else {
                    format!("{}/{}", path, entry_name)
                };
                
                let relative_path = if base_path == "." {
                    full_path.clone()
                } else {
                    full_path.strip_prefix(&format!("{}/", base_path))
                        .unwrap_or(&full_path)
                        .to_string()
                };
                
                // 递归处理子目录或文件
                match file_type {
                    FileType::Dir => {
                        // 添加目录标记
                        files.push((format!("{}/", relative_path), Vec::new()));
                        // 递归处理子目录
                        let mut sub_files = Self::collect_files_recursive(shell, &full_path, base_path)?;
                        files.append(&mut sub_files);
                    }
                    FileType::File => {
                         // 读取文件内容
                         let fd = shell.fs.open(&full_path)?;
                         let mut file_data = Vec::new();
                         loop {
                             let mut buf = [0u8; 512];
                             match shell.fs.read(fd, &mut buf) {
                                 Ok(bytes_read) => {
                                     if bytes_read == 0 {
                                         break;
                                     }
                                     file_data.extend_from_slice(&buf[..bytes_read]);
                                 }
                                 Err(e) => {
                                     shell.fs.close(fd).ok();
                                     return Err(e);
                                 }
                             }
                         }
                         shell.fs.close(fd).ok();
                         files.push((relative_path, file_data));
                     }
                    FileType::Symlink => {
                        // 处理符号链接（暂时跳过）
                        println!("Warning: Skipping symlink {}", relative_path);
                    }
                }
            }
        } else {
            // 处理单个文件
            let relative_path = if base_path == "." {
                path.to_string()
            } else {
                path.strip_prefix(&format!("{}/", base_path))
                    .unwrap_or(path)
                    .to_string()
            };
            
            let fd = shell.fs.open(path)?;
            let mut file_data = Vec::new();
            loop {
                let mut buf = [0u8; 512];
                match shell.fs.read(fd, &mut buf) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        }
                        file_data.extend_from_slice(&buf[..bytes_read]);
                    }
                    Err(e) => {
                        shell.fs.close(fd).ok();
                        return Err(e);
                    }
                }
            }
            shell.fs.close(fd).ok();
            files.push((relative_path, file_data));
        }
        
        Ok(files)
    }

    fn create_archive(files: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>> {
        let mut archive = Vec::new();
        
        // 写入文件数量
        let file_count = files.len() as u32;
        archive.extend_from_slice(&file_count.to_le_bytes());
        
        // 写入文件信息和数据
        for (path, data) in files {
            // 写入路径长度和路径
            let path_bytes = path.as_bytes();
            let path_len = path_bytes.len() as u32;
            archive.extend_from_slice(&path_len.to_le_bytes());
            archive.extend_from_slice(path_bytes);
            
            // 压缩文件数据
            let compressed_data = Self::compress_data(&data)?;
            
            // 写入原始大小和压缩后大小
            let original_size = data.len() as u32;
            let compressed_size = compressed_data.len() as u32;
            archive.extend_from_slice(&original_size.to_le_bytes());
            archive.extend_from_slice(&compressed_size.to_le_bytes());
            
            // 写入压缩数据
            archive.extend_from_slice(&compressed_data);
        }
        
        Ok(archive)
    }
}

impl Cmd for Zip {
    fn description(&self) -> String {
        "Compress multiple files and directories into a single archive using RLE compression".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.len() < 2 {
            println!("Usage: zip <archive_file> <file1> [file2] [file3] ...");
            println!("       zip -r <archive_file> <directory1> [directory2] ...");
            return;
        }

        let mut recursive = false;
        let mut start_idx = 0;
        
        // 检查是否有 -r 选项
        if argv[0] == "-r" {
            if argv.len() < 3 {
                println!("Usage: zip -r <archive_file> <directory1> [directory2] ...");
                return;
            }
            recursive = true;
            start_idx = 1;
        }
        
        let compressed_file = argv[start_idx];
        let source_paths = &argv[start_idx + 1..];
        
        if source_paths.is_empty() {
            println!("Error: No source files or directories specified");
            return;
        }

        // 收集所有文件
        let mut all_files = Vec::new();
        
        for source_path in source_paths {
            // 检查源路径是否存在
            if let Err(e) = shell.fs.path_parse(source_path) {
                println!("Error accessing source path {}: {}", source_path, e);
                return;
            }
            
            // 检查是否为目录
            let path_info = match shell.fs.path_parse(source_path) {
                Ok(info) => info,
                Err(e) => {
                    println!("Error parsing path {}: {}", source_path, e);
                    return;
                }
            };
            
            let is_directory = matches!(path_info.dir_entry.file_type.into(), FileType::Dir);
            
            // 如果是目录但没有 -r 选项，跳过
            if is_directory && !recursive {
                println!("zip: {}: is a directory (use -r to include directories)", source_path);
                continue;
            }
            
            // 收集文件（递归或非递归）
            let files = if is_directory {
                match Self::collect_files_recursive(shell, source_path, ".") {
                    Ok(files) => files,
                    Err(e) => {
                        println!("Error collecting files from {}: {}", source_path, e);
                        return;
                    }
                }
            } else {
                // 单个文件
                match Self::collect_files_recursive(shell, source_path, ".") {
                    Ok(files) => files,
                    Err(e) => {
                        println!("Error collecting file {}: {}", source_path, e);
                        return;
                    }
                }
            };
            
            all_files.extend(files);
        }
        
        let files = all_files;

        if files.is_empty() {
            println!("No files to compress");
            return;
        }

        // 创建压缩档案
        let archive_data = match Self::create_archive(files) {
            Ok(data) => data,
            Err(e) => {
                println!("Error creating archive: {}", e);
                return;
            }
        };

        // 创建压缩文件
        if let Err(e) = shell.fs.create(compressed_file) {
            println!("Error creating compressed file {}: {}", compressed_file, e);
            return;
        }

        let fd_dest = match shell.fs.open(compressed_file) {
            Ok(fd) => fd,
            Err(e) => {
                println!("Error opening compressed file {}: {}", compressed_file, e);
                return;
            }
        };

        // 写入压缩档案数据
        if let Err(e) = shell.fs.write(fd_dest, &archive_data) {
            println!("Error writing archive data: {}", e);
            shell.fs.close(fd_dest).ok();
            return;
        }

        shell.fs.close(fd_dest).ok();

        // 计算统计信息
        let file_count = u32::from_le_bytes([
            archive_data[0], archive_data[1], archive_data[2], archive_data[3]
        ]);
        
        println!("Archive created successfully!");
        println!("Files/directories compressed: {}", file_count);
        println!("Archive size: {} bytes", archive_data.len());
    }

    fn help(&self) -> String {
        format!(
            "{}

Usage: zip <archive_file> <file1> [file2] [file3] ...
       zip -r <archive_file> <directory1> [directory2] ...

Compress files and directories using simple RLE (Run-Length Encoding) compression.

Options:
  -r    Recursively compress directories and their contents

Arguments:
  <archive_file>    Name of the compressed archive file to create
  <file1> ...       Files to compress
  <directory1> ...  Directories to compress (requires -r option)

Examples:
  zip backup.zip file1.txt file2.txt        # Compress multiple files
  zip -r project.zip src/ docs/              # Compress directories recursively
  zip data.zip *.txt                         # Compress all .txt files

The compressed file contains a multi-file archive format with file paths and compressed data.",
            self.description()
        )
    }
}