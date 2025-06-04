use crate::{shell::Shell, fs::Result};
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
}

impl Cmd for Zip {
    fn description(&self) -> String {
        "Compress file using simple RLE compression".into()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        if argv.len() != 2 {
            println!("Usage: zip <source_file> <compressed_file>");
            return;
        }

        let source_file = argv[0];
        let compressed_file = argv[1];

        // 读取源文件内容
        let fd_src = match shell.fs.open(source_file) {
            Ok(fd) => fd,
            Err(e) => {
                println!("Error opening source file {}: {}", source_file, e);
                return;
            }
        };

        let mut file_data = Vec::new();
        loop {
            let mut buf = [0u8; 512];
            match shell.fs.read(fd_src, &mut buf) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    file_data.extend_from_slice(&buf[..bytes_read]);
                }
                Err(e) => {
                    println!("Error reading file {}: {}", source_file, e);
                    shell.fs.close(fd_src).ok();
                    return;
                }
            }
        }
        shell.fs.close(fd_src).ok();

        // 压缩数据
        let compressed_data = match Self::compress_data(&file_data) {
            Ok(data) => data,
            Err(e) => {
                println!("Error compressing data: {}", e);
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

        // 写入压缩头信息（原始文件大小）
        let original_size = file_data.len() as u32;
        let size_bytes = original_size.to_le_bytes();
        if let Err(e) = shell.fs.write(fd_dest, &size_bytes) {
            println!("Error writing header: {}", e);
            shell.fs.close(fd_dest).ok();
            return;
        }

        // 写入压缩数据
        if let Err(e) = shell.fs.write(fd_dest, &compressed_data) {
            println!("Error writing compressed data: {}", e);
            shell.fs.close(fd_dest).ok();
            return;
        }

        shell.fs.close(fd_dest).ok();

        let compression_ratio = if file_data.len() > 0 {
            (compressed_data.len() + 4) as f64 / file_data.len() as f64 * 100.0
        } else {
            100.0
        };

        println!("File compressed successfully!");
        println!("Original size: {} bytes", file_data.len());
        println!("Compressed size: {} bytes (including 4-byte header)", compressed_data.len() + 4);
        println!("Compression ratio: {:.2}%", compression_ratio);
    }

    fn help(&self) -> String {
        format!(
            "{}

Usage: zip <source_file> <compressed_file>

Compress a file using simple RLE (Run-Length Encoding) compression.
The compressed file includes a 4-byte header containing the original file size.",
            self.description()
        )
    }
}