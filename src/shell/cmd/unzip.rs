use crate::{shell::Shell, fs::Result};
use super::Cmd;

pub struct Unzip;

impl Unzip {
    fn decompress_data(compressed_data: &[u8], original_size: usize) -> Result<Vec<u8>> {
        if compressed_data.is_empty() {
            return Ok(Vec::new());
        }
        
        // 检查压缩标记
        let compression_flag = compressed_data[0];
        let data_to_process = &compressed_data[1..];
        
        match compression_flag {
            0x00 => {
                // 未压缩数据，直接返回
                Ok(data_to_process.to_vec())
            },
            0x01 => {
                // 已压缩数据，使用RLE解压缩
                let mut decompressed = Vec::with_capacity(original_size);
                let mut i = 0;
                
                while i < data_to_process.len() && decompressed.len() < original_size {
                    if data_to_process[i] == 0xFF {
                        if i + 1 < data_to_process.len() {
                            if data_to_process[i + 1] == 0x00 {
                                // 转义的0xFF字节
                                decompressed.push(0xFF);
                                i += 2;
                            } else {
                                // 压缩格式：0xFF + count + byte
                                if i + 2 < data_to_process.len() {
                                    let count = data_to_process[i + 1];
                                    let byte_value = data_to_process[i + 2];
                                    
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
                        decompressed.push(data_to_process[i]);
                        i += 1;
                    }
                }
                
                Ok(decompressed)
            },
            _ => {
                // 未知压缩格式，尝试使用旧的解压缩方法
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
        }
    }

    // 辅助函数：读取路径字符串
    fn read_path_string(archive_data: &[u8], offset: usize, path_len: usize) -> Result<String> {
        if offset + path_len > archive_data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid archive format: incomplete path"
            ));
        }
        
        String::from_utf8(archive_data[offset..offset + path_len].to_vec())
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid path encoding"
            ))
    }

    // 辅助函数：解析标准格式文件条目
    fn parse_standard_entry(archive_data: &[u8], offset: &mut usize, use_u16: bool) -> Result<(String, Vec<u8>)> {
        // 读取路径长度
        let path_len = if use_u16 {
            if *offset + 2 > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete path length"
                ));
            }
            let len = u16::from_le_bytes([archive_data[*offset], archive_data[*offset + 1]]) as usize;
            *offset += 2;
            len
        } else {
            if *offset + 4 > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete path length"
                ));
            }
            let len = u32::from_le_bytes([
                archive_data[*offset], archive_data[*offset + 1],
                archive_data[*offset + 2], archive_data[*offset + 3]
            ]) as usize;
            *offset += 4;
            len
        };
        
        // 读取路径
        let path = Self::read_path_string(archive_data, *offset, path_len)?;
        *offset += path_len;
        
        // 读取原始大小和压缩大小
        let (original_size, compressed_size) = if use_u16 {
            if *offset + 4 > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete size information"
                ));
            }
            let orig = u16::from_le_bytes([archive_data[*offset], archive_data[*offset + 1]]) as usize;
            let comp = u16::from_le_bytes([archive_data[*offset + 2], archive_data[*offset + 3]]) as usize;
            *offset += 4;
            (orig, comp)
        } else {
            if *offset + 8 > archive_data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid archive format: incomplete size information"
                ));
            }
            let orig = u32::from_le_bytes([
                archive_data[*offset], archive_data[*offset + 1],
                archive_data[*offset + 2], archive_data[*offset + 3]
            ]) as usize;
            let comp = u32::from_le_bytes([
                archive_data[*offset + 4], archive_data[*offset + 5],
                archive_data[*offset + 6], archive_data[*offset + 7]
            ]) as usize;
            *offset += 8;
            (orig, comp)
        };
        
        // 读取压缩数据
        if *offset + compressed_size > archive_data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid archive format: incomplete compressed data"
            ));
        }
        
        let compressed_data = &archive_data[*offset..*offset + compressed_size];
        *offset += compressed_size;
        
        // 解压缩数据
        let decompressed_data = Self::decompress_data(compressed_data, original_size)?;
        
        Ok((path, decompressed_data))
    }

    fn parse_archive(archive_data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
        let mut files = Vec::new();
        let mut offset = 0;
        
        if archive_data.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Empty archive file"
            ));
        }
        
        // 检查格式标记
        let format_flag = archive_data[0];
        offset += 1;
        
        match format_flag {
            0xCC => {
                // 紧凑格式
                if archive_data.len() < 2 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid compact archive format"
                    ));
                }
                
                let file_count = archive_data[1] as usize;
                offset += 1;
                
                // 读取索引表
                let mut file_info = Vec::new();
                for _ in 0..file_count {
                    if offset + 3 > archive_data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid compact archive: incomplete index table"
                        ));
                    }
                    
                    let path_len = archive_data[offset] as usize;
                    let data_len = u16::from_le_bytes([
                        archive_data[offset + 1], archive_data[offset + 2]
                    ]) as usize;
                    
                    file_info.push((path_len, data_len));
                    offset += 3;
                }
                
                // 读取路径
                let mut paths = Vec::new();
                for (path_len, _) in &file_info {
                    if offset + path_len + 1 > archive_data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid compact archive: incomplete path data"
                        ));
                    }
                    
                    let path = Self::read_path_string(archive_data, offset, *path_len)?;
                    paths.push(path);
                    offset += path_len + 1; // +1 for null separator
                }
                
                // 读取文件数据
                for (i, (_, data_len)) in file_info.iter().enumerate() {
                    if offset + data_len > archive_data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid compact archive: incomplete file data"
                        ));
                    }
                    
                    let data = archive_data[offset..offset + data_len].to_vec();
                    files.push((paths[i].clone(), data));
                    offset += data_len;
                }
            },
            0xFF => {
                // 标准格式（新版本，使用u16长度字段）
                if archive_data.len() < 5 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid standard archive format"
                    ));
                }
                
                let file_count = u32::from_le_bytes([
                    archive_data[1], archive_data[2], archive_data[3], archive_data[4]
                ]) as usize;
                offset += 4;
                
                // 解析每个文件
                for _ in 0..file_count {
                    let (path, data) = Self::parse_standard_entry(archive_data, &mut offset, true)?;
                    files.push((path, data));
                }
            },
            _ => {
                // 旧格式兼容性：回退到原始解析方式
                offset = 0; // 重置偏移量
                
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
                
                // 解析每个文件（旧格式，使用u32长度字段）
                for _ in 0..file_count {
                    let (path, data) = Self::parse_standard_entry(archive_data, &mut offset, false)?;
                    files.push((path, data));
                }
            }
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
        "Extract files and directories from archives created with zip command".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.is_empty() {
            println!("Usage: unzip [options] <archive_file> [files...]");
            println!("       unzip -l <archive_file>          # List archive contents");
            println!("       unzip -d <dir> <archive_file>    # Extract to directory");
            return;
        }
        
        let mut list_only = false;
        let mut output_dir = ".";
        let mut archive_file = "";
        let mut extract_files: Vec<&str> = Vec::new();
        let mut i = 0;
        
        // 解析命令行参数
        while i < argv.len() {
            match argv[i] {
                "-l" => {
                    list_only = true;
                    i += 1;
                }
                "-d" => {
                    if i + 1 >= argv.len() {
                        println!("Error: -d option requires a directory argument");
                        return;
                    }
                    output_dir = argv[i + 1];
                    i += 2;
                }
                arg if !arg.starts_with('-') => {
                    if archive_file.is_empty() {
                        archive_file = arg;
                    } else {
                        extract_files.push(arg);
                    }
                    i += 1;
                }
                _ => {
                    println!("Error: Unknown option {}", argv[i]);
                    return;
                }
            }
        }
        
        if archive_file.is_empty() {
            println!("Error: No archive file specified");
            return;
        }
        
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
                
                if list_only {
                    // 列出档案内容
                    println!("Archive: {}", archive_file);
                    println!("  Length      Date    Time    Name");
                    println!("---------  ---------- -----   ----");
                    
                    let mut total_size = 0;
                    for (path, data) in &files {
                        let size = data.len();
                        total_size += size;
                        
                        if path.ends_with('/') {
                            println!("{:>9}  ---------- -----   {}", 0, path);
                        } else {
                            println!("{:>9}  ---------- -----   {}", size, path);
                        }
                    }
                    println!("---------                     -------");
                    println!("{:>9}                     {} files", total_size, files.len());
                    return;
                }
                
                // 过滤要提取的文件
                let files_to_extract = if extract_files.is_empty() {
                    files
                } else {
                    files.into_iter().filter(|(path, _)| {
                        extract_files.iter().any(|pattern| {
                            path.contains(pattern) || path.ends_with(pattern)
                        })
                    }).collect()
                };
                
                if files_to_extract.is_empty() {
                    println!("No matching files found in archive");
                    return;
                }
                
                println!("Extracting {} items from archive...", files_to_extract.len());
                
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
                if let Err(e) = Self::extract_files(shell, output_dir, files_to_extract) {
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

Usage: unzip [options] <archive_file> [files...]

Decompress files and directories that were compressed using the zip command.
Supports both new multi-file archive format and legacy single-file format.

Options:
  -l                  List archive contents without extracting
  -d <directory>      Extract files to specified directory

Arguments:
  <archive_file>      The compressed archive file to extract
  [files...]          Specific files to extract (optional, extracts all if not specified)

Examples:
  unzip data.zip                    # Extract all files to current directory
  unzip -l backup.zip               # List contents of archive
  unzip -d ./restore backup.zip     # Extract to restore directory
  unzip data.zip file1.txt file2.txt # Extract specific files
  unzip -d output data.zip *.txt    # Extract .txt files to output directory
  unzip file.zip                    # Extract single file (legacy format)

The command automatically detects archive format and handles both single-file
and multi-file archives created with the zip command.",
            self.description()
        )
    }
}