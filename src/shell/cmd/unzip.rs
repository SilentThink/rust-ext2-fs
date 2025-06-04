use crate::{shell::Shell, fs::Result};
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
}

impl Cmd for Unzip {
    fn description(&self) -> String {
        "Decompress file compressed with zip command".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.len() != 2 {
            println!("Usage: unzip <compressed_file> <output_file>");
            return;
        }

        let compressed_file = argv[0];
        let output_file = argv[1];

        // 读取压缩文件
        let fd_src = match shell.fs.open(compressed_file) {
            Ok(fd) => fd,
            Err(e) => {
                println!("Error opening compressed file {}: {}", compressed_file, e);
                return;
            }
        };

        // 读取原始文件大小（4字节头）
        let mut size_bytes = [0u8; 4];
        match shell.fs.read(fd_src, &mut size_bytes) {
            Ok(bytes_read) => {
                if bytes_read != 4 {
                    println!("Error: Invalid compressed file format (missing header)");
                    shell.fs.close(fd_src).ok();
                    return;
                }
            }
            Err(e) => {
                println!("Error reading header: {}", e);
                shell.fs.close(fd_src).ok();
                return;
            }
        }

        let original_size = u32::from_le_bytes(size_bytes) as usize;

        // 读取压缩数据
        let mut compressed_data = Vec::new();
        loop {
            let mut buf = [0u8; 512];
            match shell.fs.read(fd_src, &mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    compressed_data.extend_from_slice(&buf[..bytes_read]);
                }
                Err(e) => {
                    println!("Error reading compressed data: {}", e);
                    shell.fs.close(fd_src).ok();
                    return;
                }
            }
        }
        shell.fs.close(fd_src).ok();

        // 解压缩数据
        let decompressed_data = match Self::decompress_data(&compressed_data, original_size) {
            Ok(data) => data,
            Err(e) => {
                println!("Error decompressing data: {}", e);
                return;
            }
        };

        // 验证解压缩后的大小
        if decompressed_data.len() != original_size {
            println!(
                "Warning: Decompressed size ({} bytes) doesn't match expected size ({} bytes)",
                decompressed_data.len(),
                original_size
            );
        }

        // 创建输出文件
        if let Err(e) = shell.fs.create(output_file) {
            println!("Error creating output file {}: {}", output_file, e);
            return;
        }

        let fd_dest = match shell.fs.open(output_file) {
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

        println!("File decompressed successfully!");
        println!("Compressed size: {} bytes (including 4-byte header)", compressed_data.len() + 4);
        println!("Decompressed size: {} bytes", decompressed_data.len());
    }

    fn help(&self) -> String {
        format!(
            "{}

Usage: unzip <compressed_file> <output_file>

Decompress a file that was compressed using the zip command.
The compressed file must contain a 4-byte header with the original file size.",
            self.description()
        )
    }
}